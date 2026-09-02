use openllve_core::{
    BenchmarkMetrics, InferenceStrategy, LlieStrategy, LlveTemporalStrategy, NativeFrameHandle,
};

#[test]
fn test_integration_pipeline_llie() {
    let mut strategy = LlieStrategy::new().unwrap();
    let mut metrics = BenchmarkMetrics::new();

    let frame_data = vec![1.0, 2.0, 3.0, 4.0];

    for i in 0..10 {
        let start = std::time::Instant::now();
        let res = strategy.process(&frame_data).unwrap();
        let elapsed = start.elapsed().as_secs_f32() * 1000.0;

        if i >= 3 {
            metrics.record(elapsed);
        }
        assert_eq!(res.len(), 4);
    }

    assert_eq!(metrics.count(), 7);
    assert!(metrics.average_latency() >= 0.0);
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
    let handle = NativeFrameHandle::new(4, 4, 16, data.as_mut_ptr(), false);
    assert_eq!(handle.width(), 4);
    assert_eq!(handle.height(), 4);
    assert_eq!(handle.stride(), 16);
    assert!(!handle.is_hardware_buffer());
}
