use super::{Pipeline, TemporalMode};
use crate::error::Result;
use crate::filters::{EwmaFilter, FrameBlendFilter};
use crate::frame::{Frame, FrameRef};
use std::path::Path;

/// Pipeline A: LLIE (static frame enhancer) with optional temporal toppings.
///
/// With the `model` feature enabled, this pipeline runs the Zero-DCE model
/// (`ModelRunner`, see `docs/dev/architecture/ARCHITECTURE.md` §11): frame in → enhanced frame out. Without the
/// feature (or without a model), `process` is an identity stub so the core
/// still builds and tests without the LiteRT runtime present.
#[derive(Default, Debug)]
pub struct LliePipeline {
    ewma: Option<EwmaFilter>,
    blend: Option<FrameBlendFilter>,
    #[cfg(feature = "model")]
    model: Option<crate::model::ModelRunner>,
}

impl LliePipeline {
    pub fn new() -> Result<Self> {
        Ok(Self {
            ewma: None,
            blend: None,
            #[cfg(feature = "model")]
            model: None,
        })
    }

    /// Attaches the Zero-DCE model (feature `model`).
    ///
    /// # Errors
    /// Returns [`crate::error::CoreError::InferenceFailure`] if the `model`
    /// feature is not enabled, or the model/LiteRT library cannot be loaded.
    pub fn with_model(self, model_path: &Path, num_threads: u32) -> Result<Self> {
        #[cfg(feature = "model")]
        {
            let model = crate::model::ModelRunner::new(model_path, num_threads)?;
            Ok(Self {
                ewma: self.ewma,
                blend: self.blend,
                model: Some(model),
            })
        }
        #[cfg(not(feature = "model"))]
        {
            let _ = (model_path, num_threads);
            Err(crate::error::CoreError::InferenceFailure(
                "the `model` cargo feature is not enabled; rebuild with --features model".to_string(),
            ))
        }
    }

    pub fn with_ewma(mut self, alpha: f32) -> Result<Self> {
        self.ewma = Some(EwmaFilter::new(alpha)?);
        Ok(self)
    }

    pub fn with_blend(mut self, beta: f32) -> Result<Self> {
        self.blend = Some(FrameBlendFilter::new(beta)?);
        Ok(self)
    }
}

impl Pipeline for LliePipeline {
    fn process(&mut self, input: &FrameRef, output: &mut Frame) -> Result<()> {
        // 1. Enhancement: the model if present, else an identity stub.
        //    (P1.2: the model maps RGB [0,1] in -> enhanced RGB [0,1] out.)
        #[cfg(feature = "model")]
        if let Some(model) = &mut self.model {
            model.run_frame(input, output)?;
        } else {
            output.copy_from(input)?;
        }
        #[cfg(not(feature = "model"))]
        {
            output.copy_from(input)?;
        }

        // 2. EWMA temporal smoothing of the enhanced frame (in place; no
        //    per-frame allocation).
        if let Some(ewma) = &mut self.ewma {
            ewma.apply_in_place(output)?;
        }

        // 3. Blend the *current* raw frame with the enhanced frame (in place):
        //    output = beta * enhanced + (1 - beta) * raw
        if let Some(blend) = &self.blend {
            blend.apply_in_place(input, output)?;
        }

        Ok(())
    }

    fn reset(&mut self) {
        if let Some(ewma) = &mut self.ewma {
            ewma.reset();
        }
    }

    fn mode(&self) -> TemporalMode {
        TemporalMode::Stateless
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::CoreError;

    fn frame_ref(data: &[f32], w: u32, h: u32, c: u32) -> FrameRef<'_> {
        FrameRef::new(w, h, c, (w * c) as usize * 4, data).unwrap()
    }

    fn frame_mut(data: &mut [f32], w: u32, h: u32, c: u32) -> Frame<'_> {
        Frame::new(w, h, c, (w * c) as usize * 4, data).unwrap()
    }

    #[test]
    fn test_llie_pipeline() {
        let mut pipeline = LliePipeline::new().unwrap();
        let input = vec![1.0, 2.0, 3.0];
        let mut output = vec![0.0; 3];
        pipeline
            .process(
                &frame_ref(&input, 3, 1, 1),
                &mut frame_mut(&mut output, 3, 1, 1),
            )
            .unwrap();
        assert_eq!(output, input);
        assert_eq!(pipeline.mode(), TemporalMode::Stateless);
    }

    #[test]
    fn test_llie_pipeline_with_ewma() {
        let mut pipeline = LliePipeline::new().unwrap().with_ewma(0.5).unwrap();
        let input = vec![0.0, 10.0];
        let mut out1 = vec![0.0; 2];
        pipeline
            .process(
                &frame_ref(&input, 2, 1, 1),
                &mut frame_mut(&mut out1, 2, 1, 1),
            )
            .unwrap();
        assert_eq!(out1, input);

        let input2 = vec![10.0, 0.0];
        let mut out2 = vec![0.0; 2];
        pipeline
            .process(
                &frame_ref(&input2, 2, 1, 1),
                &mut frame_mut(&mut out2, 2, 1, 1),
            )
            .unwrap();
        assert_eq!(out2, vec![5.0, 5.0]);
    }

    #[test]
    fn test_llie_pipeline_with_blend_uses_current_frame() {
        // Without a model the enhanced frame equals the raw frame, so a blend
        // with the current frame leaves the output equal to the input.
        let mut pipeline = LliePipeline::new().unwrap().with_blend(0.5).unwrap();
        let input = vec![1.0, 2.0, 3.0];
        let mut output = vec![0.0; 3];
        pipeline
            .process(
                &frame_ref(&input, 3, 1, 1),
                &mut frame_mut(&mut output, 3, 1, 1),
            )
            .unwrap();
        assert_eq!(output, input);
        // A second frame must not leak state from the first frame.
        let input2 = vec![4.0, 5.0, 6.0];
        let mut output2 = vec![0.0; 3];
        pipeline
            .process(
                &frame_ref(&input2, 3, 1, 1),
                &mut frame_mut(&mut output2, 3, 1, 1),
            )
            .unwrap();
        assert_eq!(output2, input2);
    }

    #[test]
    fn test_llie_pipeline_output_dim_mismatch() {
        let mut pipeline = LliePipeline::new().unwrap();
        let input = vec![1.0, 2.0, 3.0];
        let mut output = vec![0.0; 6];
        let err = pipeline
            .process(
                &frame_ref(&input, 3, 1, 1),
                &mut frame_mut(&mut output, 3, 2, 1),
            )
            .unwrap_err();
        assert!(matches!(err, CoreError::BufferDimensionMismatch { .. }));
    }

    #[cfg(not(feature = "model"))]
    #[test]
    fn test_llie_with_model_requires_feature() {
        let pipeline = LliePipeline::new().unwrap();
        let err = pipeline
            .with_model(std::path::Path::new("nonexistent.tflite"), 1)
            .unwrap_err();
        assert!(matches!(err, CoreError::InferenceFailure(_)));
    }
}
