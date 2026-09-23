//! Feature-gated LiteRT model runner (see `docs/dev/architecture/ARCHITECTURE.md` §11).
//!
//! Enabled with `cargo build --features model`. It loads a `.tflite` model via
//! `tflite-c-rs`, which dynamically opens `libtensorflowlite_c` at runtime
//! (no build-time link). The library path is taken from the
//! `OPENLLVE_TFLITE_LIB` environment variable when set, otherwise the
//! default library search path is used.
//!
//! The runner targets the Zero-DCE model (`zero-dce-int8.tflite`), which has a
//! **fixed** 256×256 input shape (see `external/models/README.md`): input
//! `(1, 256, 256, 4)` (RGB in `[0, 1]` + brightness channel) → output
//! `(1, 256, 256, 24)` (8 iterations × 3 RGB curve parameters). Frames larger
//! than 256×256 are tiled into overlapping patches and reassembled, mirroring
//! the upstream `network.py`.

use std::path::Path;

use tflite_c::{Interpreter, InterpreterOptions, Model, TfLiteLibrary, TfLiteType};

use crate::error::{CoreError, Result};

/// Patch size the Zero-DCE model was trained on.
pub const MODEL_PATCH_SIZE: u32 = 256;
/// Overlap (pixels) between adjacent patches during tiling.
pub const MODEL_TILE_OVERLAP: u32 = 16;

const EXPECTED_INPUT_DIMS: [i32; 4] = [1, 256, 256, 4];
const EXPECTED_OUTPUT_DIMS: [i32; 4] = [1, 256, 256, 24];

/// Runs the Zero-DCE model on 256×256 patches.
///
/// Owns a LiteRT interpreter and reusable scratch buffers so steady-state
/// frames allocate nothing. The interpreter is not thread-safe: one runner is
/// owned by one thread (the FFI documents this for strategy handles).
pub struct ModelRunner {
    interpreter: Interpreter,
    /// Reusable `(256, 256, 4)` model-input scratch.
    input_scratch: Vec<f32>,
    /// Reusable `(256, 256, 24)` model-output scratch.
    output_scratch: Vec<f32>,
    /// Reusable full-frame `(h_pad, w_pad, 24)` tiling accumulator.
    accum: Vec<f32>,
}

impl std::fmt::Debug for ModelRunner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ModelRunner")
            .field("input_scratch", &self.input_scratch.len())
            .field("output_scratch", &self.output_scratch.len())
            .field("accum", &self.accum.len())
            .finish()
    }
}

impl ModelRunner {
    /// Loads `model_path` and prepares an interpreter with `num_threads`
    /// threads.
    ///
    /// # Errors
    /// Returns [`CoreError::InferenceFailure`] if the LiteRT library cannot be
    /// loaded, the model file cannot be parsed, or the model's input/output
    /// shapes do not match the expected `(1, 256, 256, 4)` /
    /// `(1, 256, 256, 24)`.
    pub fn new(model_path: &Path, num_threads: u32) -> Result<Self> {
        if num_threads == 0 {
            return Err(CoreError::InvalidParameter(
                "num_threads must be >= 1".to_string(),
            ));
        }

        let lib = match std::env::var_os("OPENLLVE_TFLITE_LIB") {
            Some(path) => TfLiteLibrary::load_from_path(path)
                .map_err(|e| CoreError::InferenceFailure(format!("failed to load LiteRT library: {e}")))?,
            None => TfLiteLibrary::load_default()
                .map_err(|e| CoreError::InferenceFailure(format!("failed to load LiteRT library: {e}")))?,
        };

        let model = Model::from_file(model_path, lib.clone()).map_err(|e| {
            CoreError::InferenceFailure(format!(
                "failed to load model {}: {e}",
                model_path.display()
            ))
        })?;

        let mut options = InterpreterOptions::new(lib);
        options.num_threads(num_threads as i32);
        let interpreter = Interpreter::new(model, options)
            .map_err(|e| CoreError::InferenceFailure(format!("failed to create interpreter: {e}")))?;

        let in_dims = interpreter
            .input(0)
            .map_err(|e| CoreError::InferenceFailure(e.to_string()))?
            .dims();
        let out_dims = interpreter
            .output(0)
            .map_err(|e| CoreError::InferenceFailure(e.to_string()))?
            .dims();
        if in_dims != EXPECTED_INPUT_DIMS {
            return Err(CoreError::InferenceFailure(format!(
                "model input dims {in_dims:?} do not match expected {EXPECTED_INPUT_DIMS:?}"
            )));
        }
        if out_dims != EXPECTED_OUTPUT_DIMS {
            return Err(CoreError::InferenceFailure(format!(
                "model output dims {out_dims:?} do not match expected {EXPECTED_OUTPUT_DIMS:?}"
            )));
        }

        let patch = MODEL_PATCH_SIZE;
        Ok(Self {
            interpreter,
            input_scratch: vec![0.0f32; (patch * patch * 4) as usize],
            output_scratch: vec![0.0f32; (patch * patch * 24) as usize],
            accum: Vec::new(),
        })
    }

    /// Runs the model on a full frame, writing the enhanced RGB frame.
    ///
    /// `input` must be `(width, height, 3)` with RGB in `[0, 1]`; `output`
    /// must have the same dimensions. The 4th (brightness) input channel is
    /// derived as the frame's global mean clamped to `0.5` — a faithful
    /// simplification of the upstream `_prepare_image` (its 4×4 bilinear
    /// downscale + blend collapses to the global mean for practical images).
    ///
    /// Frames larger than 256×256 are tiled into 256×256 patches with a 16 px
    /// overlap (reflect padding at edges, linear-ramp reassembly), matching
    /// the upstream `network.py`.
    ///
    /// # Errors
    /// Returns [`CoreError::InvalidParameter`] if the input is not 3-channel,
    /// [`CoreError::BufferDimensionMismatch`] if the output dimensions differ,
    /// or [`CoreError::InferenceFailure`] if inference fails.
    pub fn run_frame(&mut self, input: &crate::frame::FrameRef, output: &mut crate::frame::Frame) -> Result<()> {
        let (w, h, c) = input.dims();
        if c != 3 {
            return Err(CoreError::InvalidParameter(format!(
                "model input must have 3 channels (RGB), got {c}"
            )));
        }
        if output.dims() != (w, h, 3) {
            return Err(CoreError::BufferDimensionMismatch {
                expected: (w * h * 3) as usize,
                actual: output.pixel_count(),
            });
        }

        let (stride, num_w, num_h, w_pad, h_pad) = Self::patch_info(w, h);
        let brightness = Self::brightness_channel(input);

        let total = (h_pad * w_pad * 24) as usize;
        self.accum.resize(total, 0.0);

        for i in 0..num_h {
            for j in 0..num_w {
                self.prepare_patch(input, j * stride, i * stride, brightness);
                self.run_model()?;
                self.accumulate(i, j, num_h, num_w, w_pad);
            }
        }

        self.apply_curves(input, output, w_pad);
        Ok(())
    }

    /// Computes the patch grid for a `(w, h)` frame.
    ///
    /// Returns `(stride, num_w, num_h, w_pad, h_pad)`.
    fn patch_info(w: u32, h: u32) -> (u32, u32, u32, u32, u32) {
        let patch = MODEL_PATCH_SIZE;
        let overlap = MODEL_TILE_OVERLAP;
        let stride = patch - overlap;
        let num_w = if w <= patch {
            1
        } else {
            (w - patch).div_ceil(stride) + 1
        };
        let num_h = if h <= patch {
            1
        } else {
            (h - patch).div_ceil(stride) + 1
        };
        let w_pad = (num_w - 1) * stride + patch;
        let h_pad = (num_h - 1) * stride + patch;
        (stride, num_w, num_h, w_pad, h_pad)
    }

    /// The brightness guidance channel: global RGB mean clamped to `0.5`.
    fn brightness_channel(input: &crate::frame::FrameRef) -> f32 {
        let (w, h, c) = input.dims();
        let mut sum = 0.0f32;
        for y in 0..h {
            for x in 0..w {
                for ch in 0..c {
                    sum += input.pixel(x, y, ch);
                }
            }
        }
        let mean = sum / (w * h * c) as f32;
        mean.min(0.5)
    }

    /// Fills `input_scratch` with the 256×256 patch at `(x0, y0)`, using
    /// reflect padding beyond the frame edges.
    fn prepare_patch(&mut self, input: &crate::frame::FrameRef, x0: u32, y0: u32, brightness: f32) {
        let (w, h) = (input.width(), input.height());
        let patch = MODEL_PATCH_SIZE as usize;
        let scratch = &mut self.input_scratch;
        for py in 0..patch {
            let sy = Self::reflect_coord(h, y0 + py as u32);
            for px in 0..patch {
                let sx = Self::reflect_coord(w, x0 + px as u32);
                let base = (py * patch + px) * 4;
                scratch[base] = input.pixel(sx, sy, 0);
                scratch[base + 1] = input.pixel(sx, sy, 1);
                scratch[base + 2] = input.pixel(sx, sy, 2);
                scratch[base + 3] = brightness;
            }
        }
    }

    /// Reflects a coordinate beyond `dim` (numpy `mode='reflect'`): the edge
    /// pixel is not repeated. `dim <= 1` maps everything to `0`.
    fn reflect_coord(dim: u32, pos: u32) -> u32 {
        if dim <= 1 {
            return 0;
        }
        let period = 2 * (dim - 1);
        let p = pos % period;
        if p < dim { p } else { period - p }
    }

    /// Writes `input_scratch` into the model input, invokes, and dequantizes
    /// the output into `output_scratch`.
    fn run_model(&mut self) -> Result<()> {
        {
            let mut in_tensor = self
                .interpreter
                .input_mut(0)
                .map_err(|e| CoreError::InferenceFailure(e.to_string()))?;
            let data = in_tensor
                .data_mut()
                .map_err(|e| CoreError::InferenceFailure(e.to_string()))?;
            data.copy_from_slice(bytemuck::cast_slice(&self.input_scratch));
        }
        self.interpreter
            .invoke()
            .map_err(|e| CoreError::InferenceFailure(format!("model invoke failed: {e}")))?;

        let out = self
            .interpreter
            .output(0)
            .map_err(|e| CoreError::InferenceFailure(e.to_string()))?;
        match out.dtype() {
            TfLiteType::Float32 => {
                let slice = out
                    .as_slice_f32()
                    .map_err(|e| CoreError::InferenceFailure(e.to_string()))?;
                self.output_scratch.copy_from_slice(slice);
            }
            TfLiteType::Int8 => {
                let raw = out
                    .as_slice_i8()
                    .map_err(|e| CoreError::InferenceFailure(e.to_string()))?;
                let q = out.quantization();
                let zp = q.zero_point as f32;
                let scale = q.scale;
                for (dst, &q) in self.output_scratch.iter_mut().zip(raw.iter()) {
                    *dst = (q as f32 - zp) * scale;
                }
            }
            actual => {
                return Err(CoreError::InferenceFailure(format!(
                    "unsupported model output dtype {actual:?}"
                )));
            }
        }
        Ok(())
    }

    /// Accumulates the current patch output into `accum` with linear-ramp
    /// overlap weights (matching upstream `network.py`).
    fn accumulate(&mut self, i: u32, j: u32, num_h: u32, num_w: u32, w_pad: u32) {
        const PATCH: u32 = MODEL_PATCH_SIZE;
        const OVERLAP: u32 = MODEL_TILE_OVERLAP;
        let denom = (OVERLAP - 1) as f32;
        let stride = PATCH - OVERLAP;
        let y0 = i * stride;
        let x0 = j * stride;
        let patch_out = &self.output_scratch;
        let accum = &mut self.accum;
        for py in 0..PATCH {
            let wy = if i != 0 && py < OVERLAP {
                (py as f32) / denom
            } else {
                1.0
            };
            let wy = if i != num_h - 1 && py >= PATCH - OVERLAP {
                wy * ((PATCH - 1 - py) as f32 / denom)
            } else {
                wy
            };
            let row_base = ((y0 + py) * w_pad) as usize;
            for px in 0..PATCH {
                let wx = if j != 0 && px < OVERLAP {
                    (px as f32) / denom
                } else {
                    1.0
                };
                let wx = if j != num_w - 1 && px >= PATCH - OVERLAP {
                    wx * ((PATCH - 1 - px) as f32 / denom)
                } else {
                    wx
                };
                let weight = wx * wy;
                let dst = row_base + (x0 + px) as usize;
                let src = ((py * PATCH + px) * 24) as usize;
                for c in 0..24 {
                    accum[dst * 24 + c] += patch_out[src + c] * weight;
                }
            }
        }
    }

    /// Applies the 8 learned curves per pixel, writing enhanced RGB to
    /// `output`.
    ///
    /// `image = image + r_k * (image^2 - image)` for k = 0..8, where `r_k` is
    /// channels `[3k, 3k+1, 3k+2]` of the reassembled 24-channel output; the
    /// result is clamped to `[0, 1]` (matching upstream `_finish_image`).
    fn apply_curves(&self, input: &crate::frame::FrameRef, output: &mut crate::frame::Frame, w_pad: u32) {
        let (w, h) = (input.width(), input.height());
        for y in 0..h {
            for x in 0..w {
                let base = ((y * w_pad + x) * 24) as usize;
                let curves = &self.accum[base..base + 24];
                let mut r = input.pixel(x, y, 0);
                let mut g = input.pixel(x, y, 1);
                let mut b = input.pixel(x, y, 2);
                for k in 0..8 {
                    r += curves[3 * k] * (r * r - r);
                    g += curves[3 * k + 1] * (g * g - g);
                    b += curves[3 * k + 2] * (b * b - b);
                }
                *output.pixel_mut(x, y, 0) = r.clamp(0.0, 1.0);
                *output.pixel_mut(x, y, 1) = g.clamp(0.0, 1.0);
                *output.pixel_mut(x, y, 2) = b.clamp(0.0, 1.0);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::frame::{Frame, FrameRef};

    /// Path to the committed model, relative to the crate root.
    fn model_path() -> std::path::PathBuf {
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../external/models/zero-dce-int8.tflite")
    }

    /// Loads a runner, or `None` when the LiteRT library / model is
    /// unavailable (the test then skips gracefully).
    fn try_runner() -> Option<ModelRunner> {
        if TfLiteLibrary::load_default().is_err() {
            eprintln!("skipping: libtensorflowlite_c not found (set OPENLLVE_TFLITE_LIB)");
            return None;
        }
        match ModelRunner::new(&model_path(), 1) {
            Ok(r) => Some(r),
            Err(e) => {
                eprintln!("skipping: model load failed: {e}");
                None
            }
        }
    }

    fn make_input(w: u32, h: u32) -> Vec<f32> {
        // Deterministic gradient in [0, 1].
        (0..(w * h * 3) as usize).map(|i| ((i % 251) as f32) / 250.0).collect()
    }

    #[test]
    fn test_patch_info_single_patch() {
        let (stride, num_w, num_h, w_pad, h_pad) = ModelRunner::patch_info(256, 256);
        assert_eq!(stride, 240);
        assert_eq!(num_w, 1);
        assert_eq!(num_h, 1);
        assert_eq!(w_pad, 256);
        assert_eq!(h_pad, 256);
    }

    #[test]
    fn test_patch_info_small_frame() {
        let (_, num_w, num_h, w_pad, h_pad) = ModelRunner::patch_info(100, 60);
        assert_eq!(num_w, 1);
        assert_eq!(num_h, 1);
        assert_eq!(w_pad, 256);
        assert_eq!(h_pad, 256);
    }

    #[test]
    fn test_patch_info_larger_frame() {
        // 400x320: num_w = (400-256+239)/240+1 = 385/240+1 = 1+1 = 2
        //            num_h = (320-256+239)/240+1 = 303/240+1 = 1+1 = 2
        let (_, num_w, num_h, w_pad, h_pad) = ModelRunner::patch_info(400, 320);
        assert_eq!(num_w, 2);
        assert_eq!(num_h, 2);
        assert_eq!(w_pad, 240 + 256);
        assert_eq!(h_pad, 240 + 256);
    }

    #[test]
    fn test_reflect_coord() {
        assert_eq!(ModelRunner::reflect_coord(4, 0), 0);
        assert_eq!(ModelRunner::reflect_coord(4, 3), 3);
        assert_eq!(ModelRunner::reflect_coord(4, 4), 2);
        assert_eq!(ModelRunner::reflect_coord(4, 5), 1);
        assert_eq!(ModelRunner::reflect_coord(4, 6), 0);
        assert_eq!(ModelRunner::reflect_coord(1, 5), 0);
    }

    #[test]
    fn test_brightness_channel() {
        // 2x2x3 all 0.4 -> mean 0.4, clamped to 0.4.
        let data = vec![0.4f32; 12];
        let frame = FrameRef::new(2, 2, 3, 24, &data).unwrap();
        assert!((ModelRunner::brightness_channel(&frame) - 0.4).abs() < 1e-6);
        // mean > 0.5 clamps to 0.5.
        let data = vec![0.9f32; 12];
        let frame = FrameRef::new(2, 2, 3, 24, &data).unwrap();
        assert!((ModelRunner::brightness_channel(&frame) - 0.5).abs() < 1e-6);
    }

    #[test]
    fn test_model_runner_loads_and_runs_patch() {
        let Some(mut runner) = try_runner() else {
            return;
        };

        let input = make_input(256, 256);
        let mut output = vec![0.0f32; (256 * 256 * 3) as usize];
        let in_frame = FrameRef::new(256, 256, 3, (256 * 3) as usize * 4, &input).unwrap();
        let mut out_frame = Frame::new(256, 256, 3, (256 * 3) as usize * 4, &mut output).unwrap();

        runner.run_frame(&in_frame, &mut out_frame).unwrap();

        // Output must be finite and within [0, 1].
        assert!(output.iter().all(|v| v.is_finite() && (0.0..=1.0).contains(v)));
        // The enhancement must actually change the frame (not be identity).
        let changed = (0..output.len())
            .filter(|&idx| (output[idx] - input[idx]).abs() > 1e-4)
            .count();
        assert!(
            changed > (256 * 256 * 3) as usize / 10,
            "output too close to input: {changed}"
        );
    }

    #[test]
    fn test_model_runner_tiled_frame() {
        let Some(mut runner) = try_runner() else {
            return;
        };

        // 400x320 exercises the tiling path (2x2 patches).
        let w = 400u32;
        let h = 320u32;
        let input = make_input(w, h);
        let mut output = vec![0.0f32; (w * h * 3) as usize];
        let in_frame = FrameRef::new(w, h, 3, (w * 3) as usize * 4, &input).unwrap();
        let mut out_frame = Frame::new(w, h, 3, (w * 3) as usize * 4, &mut output).unwrap();

        runner.run_frame(&in_frame, &mut out_frame).unwrap();
        assert!(output.iter().all(|v| v.is_finite() && (0.0..=1.0).contains(v)));
    }

    #[test]
    fn test_model_runner_rejects_non_rgb_input() {
        let Some(mut runner) = try_runner() else {
            return;
        };
        let input = vec![0.5f32; 8];
        let mut output = vec![0.0f32; 8];
        let in_frame = FrameRef::new(2, 2, 1, 4, &input).unwrap();
        let mut out_frame = Frame::new(2, 2, 3, 12, &mut output).unwrap();
        let err = runner.run_frame(&in_frame, &mut out_frame).unwrap_err();
        assert!(matches!(err, CoreError::InvalidParameter(_)));
    }
}
