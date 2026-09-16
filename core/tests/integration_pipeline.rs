use openllve_core::{
    BenchmarkMetrics, Frame, FrameRef, InferenceStrategy, LlieStrategy, LlveTemporalStrategy, NativeFrameHandle,
};

#[test]
fn test_integration_pipeline_llie() {
    let mut strategy = LlieStrategy::new().unwrap();
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
        strategy.process(&input, &mut output).unwrap();
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
    let mut strategy = LlveTemporalStrategy::new();
    let in_data = vec![0.5; 100];
    let input = FrameRef::new(10, 10, 1, 40, &in_data).unwrap();
    let mut out_data = vec![0.0; 100];
    let mut output = Frame::new(10, 10, 1, 40, &mut out_data).unwrap();

    strategy.process(&input, &mut output).unwrap();
    assert_eq!(output.as_slice(), &in_data);
    strategy.reset();
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
