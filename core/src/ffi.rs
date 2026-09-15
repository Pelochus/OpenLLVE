//! C ABI surface for the OpenLLVE core.
//!
//! All functions are `extern "C"` and exported with `#[unsafe(no_mangle)]`.
//! Handles are opaque pointers owned by the caller; each handle must be owned
//! by a single thread and freed with its matching `*_free` function.
//! Functions that can fail return an [`OpenLlveError`] code as `i32`
//! (`0` = success).

use std::slice;

use crate::error::CoreError;
use crate::filters::{EwmaFilter, FrameBlendFilter};
use crate::metrics::BenchmarkMetrics;
use crate::strategies::{InferenceStrategy, LlieStrategy, LlveTemporalStrategy};

/// C ABI version. Bump when the ABI changes in a breaking way.
pub const OPENLLVE_ABI_VERSION: u32 = 1;

/// Error codes returned by the C ABI. `0` means success.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpenLlveError {
    Ok = 0,
    NullPointer = 1,
    InvalidParameter = 2,
    BufferDimensionMismatch = 3,
    Internal = 4,
}

impl OpenLlveError {
    /// The code as returned by the C ABI.
    pub fn as_i32(self) -> i32 {
        self as i32
    }
}

/// Maps a core error to an ABI error code.
fn core_error_to_abi(error: &CoreError) -> OpenLlveError {
    match error {
        CoreError::InvalidParameter(_) => OpenLlveError::InvalidParameter,
        CoreError::BufferDimensionMismatch { .. } => OpenLlveError::BufferDimensionMismatch,
        CoreError::InferenceFailure(_) | CoreError::HardwareError(_) => OpenLlveError::Internal,
    }
}

/// Strategy handle: LLIE (static frame enhancer) or temporal (sequence model).
///
/// The name matches the opaque `OpenLlveStrategy` typedef in
/// `include/openllve_core.h`.
pub enum OpenLlveStrategy {
    Llie(LlieStrategy),
    Temporal(LlveTemporalStrategy),
}

/// Returns the C ABI version.
#[unsafe(no_mangle)]
pub extern "C" fn openllve_abi_version() -> u32 {
    OPENLLVE_ABI_VERSION
}

/// Creates a new LLIE strategy handle.
///
/// Returns null if the strategy cannot be created.
///
/// # Safety
/// The returned handle must be owned by a single thread: stateful strategies
/// must not be shared across threads. Free it with `openllve_strategy_free`.
#[unsafe(no_mangle)]
pub extern "C" fn openllve_strategy_new_llie() -> *mut OpenLlveStrategy {
    match LlieStrategy::new() {
        Ok(strategy) => Box::into_raw(Box::new(OpenLlveStrategy::Llie(strategy))),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Creates a new temporal strategy handle.
///
/// Returns null if the strategy cannot be created.
///
/// # Safety
/// The returned handle must be owned by a single thread: stateful strategies
/// must not be shared across threads. Free it with `openllve_strategy_free`.
#[unsafe(no_mangle)]
pub extern "C" fn openllve_strategy_new_temporal() -> *mut OpenLlveStrategy {
    let strategy = LlveTemporalStrategy::new();
    Box::into_raw(Box::new(OpenLlveStrategy::Temporal(strategy)))
}

/// Frees a strategy handle.
///
/// # Safety
/// `strategy` must be a non-null handle returned by one of the
/// `openllve_strategy_new_*` constructors, not yet freed, and not in use.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn openllve_strategy_free(strategy: *mut OpenLlveStrategy) {
    if !strategy.is_null() {
        let _ = unsafe { Box::from_raw(strategy) };
    }
}

/// Processes a frame with the given strategy.
///
/// Returns `0` on success, otherwise an [`OpenLlveError`] code. Only the
/// first `input_len` elements of `output_data` are written.
///
/// # Safety
/// `strategy` must be a valid, non-null strategy handle owned by the calling
/// thread. `input_data` must be non-null, aligned, and valid for reads of at
/// least `input_len` f32 elements; `output_data` must be non-null, aligned,
/// and valid for writes of at least `input_len` f32 elements (`output_len`
/// must be >= `input_len`).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn openllve_strategy_process(
    strategy: *mut OpenLlveStrategy,
    input_data: *const f32,
    input_len: usize,
    output_data: *mut f32,
    output_len: usize,
) -> i32 {
    if strategy.is_null() || input_data.is_null() || output_data.is_null() {
        return OpenLlveError::NullPointer.as_i32();
    }
    if output_len < input_len {
        return OpenLlveError::InvalidParameter.as_i32();
    }

    unsafe {
        let strategy = &mut *strategy;
        let input = slice::from_raw_parts(input_data, input_len);
        let output = slice::from_raw_parts_mut(output_data, output_len);

        let processed = match strategy {
            OpenLlveStrategy::Llie(s) => s.process(input),
            OpenLlveStrategy::Temporal(s) => s.process(input),
        };

        match processed {
            Ok(res) => {
                output[..input_len].copy_from_slice(&res);
                0
            }
            Err(e) => core_error_to_abi(&e).as_i32(),
        }
    }
}

/// Creates a new EWMA filter handle.
///
/// Returns null if `alpha` is invalid.
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
/// Returns `0` on success, otherwise an [`OpenLlveError`] code. Only the
/// first `input_len` elements of `output_data` are written.
///
/// # Safety
/// `filter` must be a valid, non-null handle returned by
/// `openllve_ewma_filter_new` and owned by the calling thread. `input_data`
/// must be non-null, aligned, and valid for reads of at least `input_len` f32
/// elements; `output_data` must be non-null, aligned, and valid for writes of
/// at least `input_len` f32 elements (`output_len` must be >= `input_len`).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn openllve_ewma_filter_apply(
    filter: *mut EwmaFilter,
    input_data: *const f32,
    input_len: usize,
    output_data: *mut f32,
    output_len: usize,
) -> i32 {
    if filter.is_null() || input_data.is_null() || output_data.is_null() {
        return OpenLlveError::NullPointer.as_i32();
    }
    if output_len < input_len {
        return OpenLlveError::InvalidParameter.as_i32();
    }

    unsafe {
        let filter = &mut *filter;
        let input = slice::from_raw_parts(input_data, input_len);
        let output = slice::from_raw_parts_mut(output_data, output_len);

        let result = filter.apply(input);
        output[..input_len].copy_from_slice(&result);
        0
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
/// Returns null if `beta` is invalid.
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
/// Returns `0` on success, otherwise an [`OpenLlveError`] code.
///
/// # Safety
/// `filter` must be a valid, non-null handle returned by
/// `openllve_blend_filter_new` and owned by the calling thread. `raw_data`
/// and `enhanced_data` must be non-null, aligned, and valid for reads of at
/// least `raw_len`/`enhanced_len` f32 elements; `output_data` must be
/// non-null, aligned, and valid for writes of at least `raw_len` f32
/// elements (`raw_len` must equal `enhanced_len`, and `output_len` must be >=
/// `raw_len`).
#[unsafe(no_mangle)]
pub unsafe extern "C" fn openllve_blend_filter_apply(
    filter: *const FrameBlendFilter,
    raw_data: *const f32,
    raw_len: usize,
    enhanced_data: *const f32,
    enhanced_len: usize,
    output_data: *mut f32,
    output_len: usize,
) -> i32 {
    if filter.is_null() || raw_data.is_null() || enhanced_data.is_null() || output_data.is_null() {
        return OpenLlveError::NullPointer.as_i32();
    }
    if raw_len != enhanced_len || output_len < raw_len {
        return OpenLlveError::InvalidParameter.as_i32();
    }

    unsafe {
        let filter = &*filter;
        let raw = slice::from_raw_parts(raw_data, raw_len);
        let enhanced = slice::from_raw_parts(enhanced_data, enhanced_len);
        let output = slice::from_raw_parts_mut(output_data, output_len);

        match filter.apply(raw, enhanced) {
            Ok(result) => {
                output[..raw_len].copy_from_slice(&result);
                0
            }
            Err(e) => core_error_to_abi(&e).as_i32(),
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

/// Creates a benchmark metrics handle that excludes the first
/// `warmup_frames` recorded samples from all statistics.
///
/// # Safety
/// The returned handle must be owned by a single thread and freed with
/// `openllve_metrics_free`.
#[unsafe(no_mangle)]
pub extern "C" fn openllve_metrics_new_with_warmup(warmup_frames: usize) -> *mut BenchmarkMetrics {
    Box::into_raw(Box::new(BenchmarkMetrics::with_warmup(warmup_frames)))
}

/// Records a latency sample in milliseconds.
///
/// # Safety
/// `metrics` must be a valid, non-null handle returned by
/// `openllve_metrics_new` or `openllve_metrics_new_with_warmup` and owned by
/// the calling thread.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn openllve_metrics_record(metrics: *mut BenchmarkMetrics, latency_ms: f64) {
    if !metrics.is_null() {
        unsafe { (*metrics).record(latency_ms) };
    }
}

/// Returns the average latency in milliseconds.
///
/// # Safety
/// `metrics` must be a valid, non-null metrics handle owned by the calling
/// thread.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn openllve_metrics_get_average(metrics: *const BenchmarkMetrics) -> f64 {
    if metrics.is_null() {
        0.0
    } else {
        unsafe { (*metrics).average_latency() }
    }
}

/// Returns the median latency in milliseconds.
///
/// # Safety
/// `metrics` must be a valid, non-null metrics handle owned by the calling
/// thread.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn openllve_metrics_get_median(metrics: *const BenchmarkMetrics) -> f64 {
    if metrics.is_null() {
        0.0
    } else {
        unsafe { (*metrics).median_latency() }
    }
}

/// Returns the frames-per-second rate derived from recorded latencies.
///
/// # Safety
/// `metrics` must be a valid, non-null metrics handle owned by the calling
/// thread.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn openllve_metrics_get_fps(metrics: *const BenchmarkMetrics) -> f64 {
    if metrics.is_null() {
        0.0
    } else {
        unsafe { (*metrics).fps() }
    }
}

/// Returns the 99th percentile latency in milliseconds.
///
/// # Safety
/// `metrics` must be a valid, non-null metrics handle owned by the calling
/// thread.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn openllve_metrics_get_p99(metrics: *const BenchmarkMetrics) -> f64 {
    if metrics.is_null() {
        0.0
    } else {
        unsafe { (*metrics).percentile_p99() }
    }
}

/// Returns the number of non-warm-up samples recorded.
///
/// # Safety
/// `metrics` must be a valid, non-null metrics handle owned by the calling
/// thread.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn openllve_metrics_get_count(metrics: *const BenchmarkMetrics) -> usize {
    if metrics.is_null() {
        0
    } else {
        unsafe { (*metrics).count() }
    }
}

/// Resets the metrics handle to an empty state (including the warm-up budget).
///
/// # Safety
/// `metrics` must be a valid, non-null metrics handle owned by the calling
/// thread.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn openllve_metrics_reset(metrics: *mut BenchmarkMetrics) {
    if !metrics.is_null() {
        unsafe { (*metrics).reset() };
    }
}

/// Frees a metrics handle.
///
/// # Safety
/// `metrics` must be a non-null handle returned by `openllve_metrics_new` or
/// `openllve_metrics_new_with_warmup`, not yet freed, and not in use.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn openllve_metrics_free(metrics: *mut BenchmarkMetrics) {
    if !metrics.is_null() {
        let _ = unsafe { Box::from_raw(metrics) };
    }
}

#[cfg(test)]
mod ffi_tests {
    use super::*;

    #[test]
    fn test_abi_version() {
        assert_eq!(openllve_abi_version(), OPENLLVE_ABI_VERSION);
    }

    #[test]
    fn test_strategy_process_roundtrip() {
        let handle = openllve_strategy_new_llie();
        assert!(!handle.is_null());

        let input = vec![1.0f32, 2.0, 3.0];
        let mut output = vec![0.0f32; 3];
        let rc = unsafe {
            openllve_strategy_process(
                handle,
                input.as_ptr(),
                input.len(),
                output.as_mut_ptr(),
                output.len(),
            )
        };
        assert_eq!(rc, OpenLlveError::Ok.as_i32());
        assert_eq!(output, input);

        unsafe { openllve_strategy_free(handle) };
    }

    #[test]
    fn test_strategy_process_null_handle() {
        let input = vec![1.0f32];
        let mut output = vec![0.0f32; 1];
        let rc = unsafe {
            openllve_strategy_process(
                std::ptr::null_mut(),
                input.as_ptr(),
                input.len(),
                output.as_mut_ptr(),
                output.len(),
            )
        };
        assert_eq!(rc, OpenLlveError::NullPointer.as_i32());
    }

    #[test]
    fn test_strategy_process_output_too_small() {
        let handle = openllve_strategy_new_temporal();
        assert!(!handle.is_null());

        let input = vec![1.0f32, 2.0];
        let mut output = vec![0.0f32; 1];
        let rc = unsafe {
            openllve_strategy_process(
                handle,
                input.as_ptr(),
                input.len(),
                output.as_mut_ptr(),
                output.len(),
            )
        };
        assert_eq!(rc, OpenLlveError::InvalidParameter.as_i32());

        unsafe { openllve_strategy_free(handle) };
    }

    #[test]
    fn test_blend_filter_roundtrip() {
        let filter = openllve_blend_filter_new(0.5);
        assert!(!filter.is_null());

        let raw = vec![0.0f32, 10.0];
        let enhanced = vec![10.0f32, 0.0];
        let mut output = vec![0.0f32; 2];
        let rc = unsafe {
            openllve_blend_filter_apply(
                filter,
                raw.as_ptr(),
                raw.len(),
                enhanced.as_ptr(),
                enhanced.len(),
                output.as_mut_ptr(),
                output.len(),
            )
        };
        assert_eq!(rc, OpenLlveError::Ok.as_i32());
        // 0.5 * enhanced + 0.5 * raw
        assert!((output[0] - 5.0).abs() < 1e-4);
        assert!((output[1] - 5.0).abs() < 1e-4);

        unsafe { openllve_blend_filter_free(filter) };
    }

    #[test]
    fn test_metrics_roundtrip_with_warmup() {
        let metrics = openllve_metrics_new_with_warmup(1);
        unsafe { openllve_metrics_record(metrics, 100.0) }; // warm-up
        unsafe { openllve_metrics_record(metrics, 10.0) };
        unsafe { openllve_metrics_record(metrics, 20.0) };

        assert_eq!(unsafe { openllve_metrics_get_count(metrics) }, 2);
        assert_eq!(unsafe { openllve_metrics_get_average(metrics) }, 15.0);
        assert_eq!(unsafe { openllve_metrics_get_median(metrics) }, 15.0);
        assert_eq!(unsafe { openllve_metrics_get_p99(metrics) }, 20.0);

        unsafe { openllve_metrics_free(metrics) };
    }
}
