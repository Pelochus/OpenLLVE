use std::slice;

use crate::filters::{EwmaFilter, FrameBlendFilter};
use crate::metrics::BenchmarkMetrics;
use crate::strategies::{InferenceStrategy, LlieStrategy, LlveTemporalStrategy};

pub enum OpenLlveStrategyEnum {
    Llie(LlieStrategy),
    Temporal(LlveTemporalStrategy),
}

/// Creates a new LLIE strategy handle.
///
/// # Safety
/// The returned handle must be owned by a single thread: stateful strategies
/// must not be shared across threads. Free it with `openllve_strategy_free`.
#[unsafe(no_mangle)]
pub extern "C" fn openllve_strategy_new_llie() -> *mut OpenLlveStrategyEnum {
    match LlieStrategy::new() {
        Ok(strategy) => Box::into_raw(Box::new(OpenLlveStrategyEnum::Llie(strategy))),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Creates a new temporal strategy handle.
///
/// # Safety
/// The returned handle must be owned by a single thread: stateful strategies
/// must not be shared across threads. Free it with `openllve_strategy_free`.
#[unsafe(no_mangle)]
pub extern "C" fn openllve_strategy_new_temporal() -> *mut OpenLlveStrategyEnum {
    let strategy = LlveTemporalStrategy::new();
    Box::into_raw(Box::new(OpenLlveStrategyEnum::Temporal(strategy)))
}

/// Frees a strategy handle.
///
/// # Safety
/// `strategy` must be a non-null handle returned by one of the
/// `openllve_strategy_new_*` constructors, not yet freed, and not in use.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn openllve_strategy_free(strategy: *mut OpenLlveStrategyEnum) {
    if !strategy.is_null() {
        let _ = unsafe { Box::from_raw(strategy) };
    }
}

/// Processes a frame with the given strategy.
///
/// # Safety
/// `strategy` must be a valid, non-null strategy handle owned by the calling
/// thread. `input_data` must be non-null, aligned, and valid for reads of at
/// least `input_len` f32 elements; `output_data` must be non-null, aligned,
/// and valid for writes of at least `output_len` f32 elements.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn openllve_strategy_process(
    strategy: *mut OpenLlveStrategyEnum,
    input_data: *const f32,
    input_len: usize,
    output_data: *mut f32,
    output_len: usize,
) -> bool {
    if strategy.is_null() || input_data.is_null() || output_data.is_null() {
        return false;
    }

    unsafe {
        let strategy = &mut *strategy;
        let input = slice::from_raw_parts(input_data, input_len);
        let output = slice::from_raw_parts_mut(output_data, output_len);

        if output_len < input_len {
            return false;
        }

        let processed = match strategy {
            OpenLlveStrategyEnum::Llie(s) => s.process(input),
            OpenLlveStrategyEnum::Temporal(s) => s.process(input),
        };

        match processed {
            Ok(res) => {
                let copy_len = res.len().min(output_len);
                output[..copy_len].copy_from_slice(&res[..copy_len]);
                true
            }
            Err(_) => false,
        }
    }
}

/// Creates a new EWMA filter handle.
///
/// # Safety
/// The returned handle must be owned by a single thread and freed with
/// `openllve_ewma_filter_free`.
#[unsafe(no_mangle)]
pub extern "C" fn openllve_ewma_filter_new(alpha: f32) -> *mut EwmaFilter {
    match EwmaFilter::new(alpha) {
        Ok(filter) => Box::into_raw(Box::new(filter)),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Applies the EWMA filter to a frame.
///
/// # Safety
/// `filter` must be a valid, non-null handle returned by
/// `openllve_ewma_filter_new` and owned by the calling thread. `input_data`
/// must be non-null, aligned, and valid for reads of at least `input_len` f32
/// elements; `output_data` must be non-null, aligned, and valid for writes of
/// at least `output_len` f32 elements.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn openllve_ewma_filter_apply(
    filter: *mut EwmaFilter,
    input_data: *const f32,
    input_len: usize,
    output_data: *mut f32,
    output_len: usize,
) -> bool {
    if filter.is_null() || input_data.is_null() || output_data.is_null() {
        return false;
    }

    unsafe {
        let filter = &mut *filter;
        let input = slice::from_raw_parts(input_data, input_len);
        let output = slice::from_raw_parts_mut(output_data, output_len);

        if output_len < input_len {
            return false;
        }

        let result = filter.apply(input);
        let copy_len = result.len().min(output_len);
        output[..copy_len].copy_from_slice(&result[..copy_len]);
        true
    }
}

/// Frees an EWMA filter handle.
///
/// # Safety
/// `filter` must be a non-null handle returned by `openllve_ewma_filter_new`,
/// not yet freed, and not in use.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn openllve_ewma_filter_free(filter: *mut EwmaFilter) {
    if !filter.is_null() {
        let _ = unsafe { Box::from_raw(filter) };
    }
}

/// Creates a new frame blend filter handle.
///
/// # Safety
/// The returned handle must be owned by a single thread and freed with
/// `openllve_blend_filter_free`.
#[unsafe(no_mangle)]
pub extern "C" fn openllve_blend_filter_new(beta: f32) -> *mut FrameBlendFilter {
    match FrameBlendFilter::new(beta) {
        Ok(filter) => Box::into_raw(Box::new(filter)),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Blends a raw frame with an enhanced frame.
///
/// # Safety
/// `filter` must be a valid, non-null handle returned by
/// `openllve_blend_filter_new` and owned by the calling thread. `raw_data`
/// and `enhanced_data` must be non-null, aligned, and valid for reads of at
/// least `raw_len`/`enhanced_len` f32 elements; `output_data` must be
/// non-null, aligned, and valid for writes of at least `output_len` f32
/// elements.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn openllve_blend_filter_apply(
    filter: *const FrameBlendFilter,
    raw_data: *const f32,
    raw_len: usize,
    enhanced_data: *const f32,
    enhanced_len: usize,
    output_data: *mut f32,
    output_len: usize,
) -> bool {
    if filter.is_null() || raw_data.is_null() || enhanced_data.is_null() || output_data.is_null() {
        return false;
    }

    unsafe {
        let filter = &*filter;
        let raw = slice::from_raw_parts(raw_data, raw_len);
        let enhanced = slice::from_raw_parts(enhanced_data, enhanced_len);
        let output = slice::from_raw_parts_mut(output_data, output_len);

        if raw_len != enhanced_len || output_len < raw_len {
            return false;
        }

        match filter.apply(raw, enhanced) {
            Ok(result) => {
                output[..result.len()].copy_from_slice(&result);
                true
            }
            Err(_) => false,
        }
    }
}

/// Frees a frame blend filter handle.
///
/// # Safety
/// `filter` must be a non-null handle returned by `openllve_blend_filter_new`,
/// not yet freed, and not in use.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn openllve_blend_filter_free(filter: *mut FrameBlendFilter) {
    if !filter.is_null() {
        let _ = unsafe { Box::from_raw(filter) };
    }
}

/// Creates a new benchmark metrics handle.
///
/// # Safety
/// The returned handle must be owned by a single thread and freed with
/// `openllve_metrics_free`.
#[unsafe(no_mangle)]
pub extern "C" fn openllve_metrics_new() -> *mut BenchmarkMetrics {
    Box::into_raw(Box::new(BenchmarkMetrics::new()))
}

/// Records a latency sample in the metrics handle.
///
/// # Safety
/// `metrics` must be a valid, non-null handle returned by
/// `openllve_metrics_new` and owned by the calling thread.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn openllve_metrics_record(metrics: *mut BenchmarkMetrics, latency_ms: f32) {
    if !metrics.is_null() {
        unsafe { (*metrics).record(latency_ms) };
    }
}

/// Returns the average latency in milliseconds.
///
/// # Safety
/// `metrics` must be a valid, non-null handle returned by
/// `openllve_metrics_new` and owned by the calling thread.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn openllve_metrics_get_average(metrics: *const BenchmarkMetrics) -> f32 {
    if metrics.is_null() {
        0.0
    } else {
        unsafe { (*metrics).average_latency() }
    }
}

/// Returns the frames-per-second rate derived from recorded latencies.
///
/// # Safety
/// `metrics` must be a valid, non-null handle returned by
/// `openllve_metrics_new` and owned by the calling thread.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn openllve_metrics_get_fps(metrics: *const BenchmarkMetrics) -> f32 {
    if metrics.is_null() {
        0.0
    } else {
        unsafe { (*metrics).fps() }
    }
}

/// Returns the 99th percentile latency in milliseconds.
///
/// # Safety
/// `metrics` must be a valid, non-null handle returned by
/// `openllve_metrics_new` and owned by the calling thread.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn openllve_metrics_get_p99(metrics: *const BenchmarkMetrics) -> f32 {
    if metrics.is_null() {
        0.0
    } else {
        unsafe { (*metrics).percentile_p99() }
    }
}

/// Resets the metrics handle to an empty state.
///
/// # Safety
/// `metrics` must be a valid, non-null handle returned by
/// `openllve_metrics_new` and owned by the calling thread.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn openllve_metrics_reset(metrics: *mut BenchmarkMetrics) {
    if !metrics.is_null() {
        unsafe { (*metrics).reset() };
    }
}

/// Frees a metrics handle.
///
/// # Safety
/// `metrics` must be a non-null handle returned by `openllve_metrics_new`,
/// not yet freed, and not in use.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn openllve_metrics_free(metrics: *mut BenchmarkMetrics) {
    if !metrics.is_null() {
        let _ = unsafe { Box::from_raw(metrics) };
    }
}
