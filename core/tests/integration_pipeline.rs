use openllve_core::{BenchmarkMetrics, InferenceStrategy, LlieStrategy, LlveTemporalStrategy, NativeFrameHandle};

#[test]
fn test_integration_pipeline_llie() {
    let mut strategy = LlieStrategy::new().unwrap();
    let mut metrics = BenchmarkMetrics::with_warmup(3);

    let frame_data = vec![1.0, 2.0, 3.0, 4.0];

    for _ in 0..10 {
        let start = std::time::Instant::now();
        let res = strategy.process(&frame_data).unwrap();
        let elapsed = start.elapsed().as_secs_f64() * 1000.0;

        metrics.record(elapsed);
        assert_eq!(res.len(), 4);
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
    let frame_data = vec![0.5; 100];

    let res = strategy.process(&frame_data).unwrap();
    assert_eq!(res, frame_data);
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
