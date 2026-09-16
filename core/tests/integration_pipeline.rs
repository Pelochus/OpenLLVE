use openllve_core::{
    BenchmarkMetrics, Frame, FrameRef, LliePipeline, LlveTemporalPipeline, NativeFrameHandle, Pipeline,
};

#[cfg(feature = "model")]
fn model_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../external/models/zero-dce-int8.tflite")
}

#[cfg(feature = "model")]
fn tflite_available() -> bool {
    // Probe the library without constructing a runner: load_default succeeds
    // only when libtensorflowlite_c is reachable.
    tflite_c::TfLiteLibrary::load_default().is_ok()
}

#[cfg(feature = "model")]
#[test]
fn test_integration_pipeline_llie_with_model() {
    if !tflite_available() {
        eprintln!("skipping: libtensorflowlite_c not found (set OPENLLVE_TFLITE_LIB)");
        return;
    }
    let mut pipeline = match LliePipeline::new().unwrap().with_model(&model_path(), 1) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("skipping: model load failed: {e}");
            return;
        }
    };

    let w = 256u32;
    let h = 256u32;
    let c = 3u32;
    let stride = (w * c) as usize * 4;
    let in_data: Vec<f32> = (0..(w * h * c) as usize).map(|i| ((i % 251) as f32) / 250.0).collect();
    let input = FrameRef::new(w, h, c, stride, &in_data).unwrap();
    let mut out_data = vec![0.0f32; (w * h * c) as usize];
    let mut output = Frame::new(w, h, c, stride, &mut out_data).unwrap();

    pipeline.process(&input, &mut output).unwrap();

    // Enhanced frame must be in [0,1], finite, and actually changed.
    assert!(
        output
            .as_slice()
            .iter()
            .all(|v| v.is_finite() && (0.0..=1.0).contains(v))
    );
    let changed = (0..output.as_slice().len())
        .filter(|&idx| (output.as_slice()[idx] - in_data[idx]).abs() > 1e-4)
        .count();
    assert!(
        changed > (w * h * c) as usize / 10,
        "output too close to input: {changed}"
    );
}

#[test]
fn test_integration_pipeline_llie() {
    let mut pipeline = LliePipeline::new().unwrap();
    let mut metrics = BenchmarkMetrics::with_warmup(3);

    let w = 4u32;
    let h = 4u32;
    let c = 3u32;
    let stride = (w * c) as usize * 4;
    let in_data: Vec<f32> = (0..(w * h * c) as usize).map(|i| i as f32 * 0.001).collect();
    let input = FrameRef::new(w, h, c, stride, &in_data).unwrap();
    let mut out_data = vec![0.0f32; (w * h * c) as usize];
    let mut output = Frame::new(w, h, c, stride, &mut out_data).unwrap();

    for _ in 0..10 {
        let start = std::time::Instant::now();
        pipeline.process(&input, &mut output).unwrap();
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        metrics.record(elapsed);
        assert_eq!(output.pixel_count(), (w * h * c) as usize);
    }

    // 10 frames with a 3-frame warm-up budget -> 7 counted samples.
    assert_eq!(metrics.count(), 7);
    assert_eq!(metrics.warmup_count(), 3);
    assert!(metrics.average_latency() >= 0.0);
    assert!(metrics.median_latency() >= 0.0);
    assert!(metrics.fps() >= 0.0);
}

#[test]
fn test_integration_pipeline_temporal() {
    let mut pipeline = LlveTemporalPipeline::new();
    let in_data = vec![0.5; 100];
    let input = FrameRef::new(10, 10, 1, 40, &in_data).unwrap();
    let mut out_data = vec![0.0; 100];
    let mut output = Frame::new(10, 10, 1, 40, &mut out_data).unwrap();

    pipeline.process(&input, &mut output).unwrap();
    assert_eq!(output.as_slice(), &in_data);
    pipeline.reset();
}

#[test]
fn test_native_frame_handle() {
    let mut data = vec![0u8; 64];
    let handle = NativeFrameHandle::new(4, 4, 16, data.as_mut_ptr(), false).unwrap();
    assert_eq!(handle.width(), 4);
    assert_eq!(handle.height(), 4);
    assert_eq!(handle.stride(), 16);
    assert!(!handle.is_hardware_buffer());
}
