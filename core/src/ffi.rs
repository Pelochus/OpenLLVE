//! C ABI surface for the OpenLLVE core.
//!
//! All functions are `extern "C"` and exported with `#[unsafe(no_mangle)]`.
//! Handles are opaque pointers owned by the caller; each handle must be owned
//! by a single thread and freed with its matching `*_free` function.
//! Functions that can fail return an [`OpenLlveError`] code as `i32`
//! (`0` = success).
//!
//! Frames are passed as explicit dimensions plus raw pointers:
//! `width`, `height`, `channels`, `stride` (bytes per row), and the data
//! pointer. Dimension mismatches are checked and reported as error codes.

use std::slice;

use crate::error::CoreError;
use crate::filters::{EwmaFilter, FrameBlendFilter};
use crate::frame::{Frame, FrameRef};
use crate::metrics::BenchmarkMetrics;
use crate::pipelines::{LliePipeline, LlveTemporalPipeline, Pipeline};

/// C ABI version. Bump when the ABI changes in a breaking way.
pub const OPENLLVE_ABI_VERSION: u32 = 3;

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

/// Pipeline handle: LLIE (static frame enhancer) or temporal (sequence model).
///
/// The name matches the opaque `OpenLlvePipeline` typedef in
/// `include/openllve_core.h`.
pub enum OpenLlvePipeline {
    Llie(LliePipeline),
    Temporal(LlveTemporalPipeline),
}

/// Returns the C ABI version.
#[unsafe(no_mangle)]
pub extern "C" fn openllve_abi_version() -> u32 {
    OPENLLVE_ABI_VERSION
}

/// Creates a new LLIE pipeline handle.
///
/// Returns null if the pipeline cannot be created.
///
/// # Safety
/// The returned handle must be owned by a single thread: stateful pipelines
/// must not be shared across threads. Free it with `openllve_pipeline_free`.
#[unsafe(no_mangle)]
pub extern "C" fn openllve_pipeline_new_llie() -> *mut OpenLlvePipeline {
    match LliePipeline::new() {
        Ok(pipeline) => Box::into_raw(Box::new(OpenLlvePipeline::Llie(pipeline))),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Creates a new LLIE pipeline handle with the Zero-DCE model loaded.
///
/// The model is loaded from `model_path` (a NUL-terminated UTF-8 C string)
/// and run with `num_threads` threads. Returns null if `model_path` is null,
/// `num_threads` is not positive, the `model` cargo feature is not enabled,
/// or the LiteRT library / model file cannot be loaded.
///
/// # Safety
/// `model_path` must be a valid, NUL-terminated C string. The returned handle
/// must be owned by a single thread and freed with `openllve_pipeline_free`.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn openllve_pipeline_new_llie_with_model(
    model_path: *const std::os::raw::c_char,
    num_threads: std::os::raw::c_int,
) -> *mut OpenLlvePipeline {
    use std::ffi::CStr;
    use std::path::Path;

    if model_path.is_null() || num_threads <= 0 {
        return std::ptr::null_mut();
    }
    let path = match unsafe { CStr::from_ptr(model_path) }.to_str() {
        Ok(s) => Path::new(s),
        Err(_) => return std::ptr::null_mut(),
    };
    match LliePipeline::new().and_then(|s| s.with_model(path, num_threads as u32)) {
        Ok(pipeline) => Box::into_raw(Box::new(OpenLlvePipeline::Llie(pipeline))),
        Err(_) => std::ptr::null_mut(),
    }
}

/// Creates a new temporal pipeline handle.
///
/// Returns null if the pipeline cannot be created.
///
/// # Safety
/// The returned handle must be owned by a single thread: stateful pipelines
/// must not be shared across threads. Free it with `openllve_pipeline_free`.
#[unsafe(no_mangle)]
pub extern "C" fn openllve_pipeline_new_temporal() -> *mut OpenLlvePipeline {
    let pipeline = LlveTemporalPipeline::new();
    Box::into_raw(Box::new(OpenLlvePipeline::Temporal(pipeline)))
}

/// Frees a pipeline handle.
///
/// # Safety
/// `pipeline` must be a non-null handle returned by one of the
/// `openllve_pipeline_new_*` constructors, not yet freed, and not in use.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn openllve_pipeline_free(pipeline: *mut OpenLlvePipeline) {
    if !pipeline.is_null() {
        let _ = unsafe { Box::from_raw(pipeline) };
    }
}

/// Processes a frame with the given pipeline, writing the result into the
/// caller-owned output buffer.
///
/// Input and output frames are described independently (the model may change
/// the channel count, e.g. Zero-DCE maps 4 channels to 24).
///
/// Returns `0` on success, otherwise an [`OpenLlveError`] code.
///
/// # Safety
/// `pipeline` must be a valid, non-null pipeline handle owned by the calling
/// thread. `in_data` must be non-null, aligned, and valid for reads of at
/// least `in_stride * in_height` bytes; `out_data` must be non-null, aligned,
/// and valid for writes of at least `out_stride * out_height` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn openllve_process_frame(
    pipeline: *mut OpenLlvePipeline,
    in_width: u32,
    in_height: u32,
    in_channels: u32,
    in_stride: usize,
    in_data: *const f32,
    out_width: u32,
    out_height: u32,
    out_channels: u32,
    out_stride: usize,
    out_data: *mut f32,
) -> i32 {
    if pipeline.is_null() || in_data.is_null() || out_data.is_null() {
        return OpenLlveError::NullPointer.as_i32();
    }

    unsafe {
        let pipeline = &mut *pipeline;
        let input = FrameRef::new(
            in_width,
            in_height,
            in_channels,
            in_stride,
            slice::from_raw_parts(in_data, in_stride * in_height as usize / 4),
        );
        let output = Frame::new(
            out_width,
            out_height,
            out_channels,
            out_stride,
            slice::from_raw_parts_mut(out_data, out_stride * out_height as usize / 4),
        );

        match (input, output) {
            (Ok(input), Ok(mut output)) => {
                let result = match pipeline {
                    OpenLlvePipeline::Llie(s) => s.process(&input, &mut output),
                    OpenLlvePipeline::Temporal(s) => s.process(&input, &mut output),
                };
                match result {
                    Ok(()) => 0,
                    Err(e) => core_error_to_abi(&e).as_i32(),
                }
            }
            (Err(e), _) | (_, Err(e)) => core_error_to_abi(&e).as_i32(),
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

/// Applies the EWMA filter to a frame, writing into the caller-owned output
/// buffer. The output frame must have the same dimensions as the input.
///
/// Returns `0` on success, otherwise an [`OpenLlveError`] code.
///
/// # Safety
/// `filter` must be a valid, non-null handle returned by
/// `openllve_ewma_filter_new` and owned by the calling thread. `input_data`
/// must be non-null, aligned, and valid for reads of at least `stride *
/// height` bytes; `output_data` must be non-null, aligned, and valid for
/// writes of at least `stride * height` bytes.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn openllve_ewma_filter_apply(
    filter: *mut EwmaFilter,
    width: u32,
    height: u32,
    channels: u32,
    stride: usize,
    input_data: *const f32,
    output_data: *mut f32,
) -> i32 {
    if filter.is_null() || input_data.is_null() || output_data.is_null() {
        return OpenLlveError::NullPointer.as_i32();
    }

    unsafe {
        let filter = &mut *filter;
        let input = FrameRef::new(
            width,
            height,
            channels,
            stride,
            slice::from_raw_parts(input_data, stride * height as usize / 4),
        );
        let output = Frame::new(
            width,
            height,
            channels,
            stride,
            slice::from_raw_parts_mut(output_data, stride * height as usize / 4),
        );

        match (input, output) {
            (Ok(input), Ok(mut output)) => match filter.apply(&input, &mut output) {
                Ok(()) => 0,
                Err(e) => core_error_to_abi(&e).as_i32(),
            },
            (Err(e), _) | (_, Err(e)) => core_error_to_abi(&e).as_i32(),
        }
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

/// Blends a raw frame with an enhanced frame, writing into the caller-owned
/// output buffer. All three frames must share the same dimensions.
///
/// Returns `0` on success, otherwise an [`OpenLlveError`] code.
///
/// # Safety
/// `filter` must be a valid, non-null handle returned by
/// `openllve_blend_filter_new` and owned by the calling thread. `raw_data`,
/// `enhanced_data`, and `output_data` must be non-null, aligned, and valid
/// for reads/writes of at least `stride * height` bytes respectively.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn openllve_blend_filter_apply(
    filter: *const FrameBlendFilter,
    width: u32,
    height: u32,
    channels: u32,
    stride: usize,
    raw_data: *const f32,
    enhanced_data: *const f32,
    output_data: *mut f32,
) -> i32 {
    if filter.is_null() || raw_data.is_null() || enhanced_data.is_null() || output_data.is_null() {
        return OpenLlveError::NullPointer.as_i32();
    }

    unsafe {
        let filter = &*filter;
        let len = stride * height as usize / 4;
        let raw = FrameRef::new(
            width,
            height,
            channels,
            stride,
            slice::from_raw_parts(raw_data, len),
        );
        let enhanced = FrameRef::new(
            width,
            height,
            channels,
            stride,
            slice::from_raw_parts(enhanced_data, len),
        );
        let output = Frame::new(
            width,
            height,
            channels,
            stride,
            slice::from_raw_parts_mut(output_data, len),
        );

        match (raw, enhanced, output) {
            (Ok(raw), Ok(enhanced), Ok(mut output)) => match filter.apply(&raw, &enhanced, &mut output) {
                Ok(()) => 0,
                Err(e) => core_error_to_abi(&e).as_i32(),
            },
            (Err(e), _, _) | (_, Err(e), _) | (_, _, Err(e)) => core_error_to_abi(&e).as_i32(),
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

    const W: u32 = 4;
    const H: u32 = 2;
    const C: u32 = 3;
    const STRIDE: usize = (W * C) as usize * 4;
    const N: usize = (W * H * C) as usize;

    #[test]
    fn test_abi_version() {
        assert_eq!(openllve_abi_version(), OPENLLVE_ABI_VERSION);
    }

    #[test]
    fn test_process_frame_roundtrip() {
        let handle = openllve_pipeline_new_llie();
        assert!(!handle.is_null());

        let input: Vec<f32> = (1..=N).map(|i| i as f32).collect();
        let mut output = vec![0.0f32; N];
        let rc = unsafe {
            openllve_process_frame(
                handle,
                W,
                H,
                C,
                STRIDE,
                input.as_ptr(),
                W,
                H,
                C,
                STRIDE,
                output.as_mut_ptr(),
            )
        };
        assert_eq!(rc, OpenLlveError::Ok.as_i32());
        assert_eq!(output, input);

        unsafe { openllve_pipeline_free(handle) };
    }

    #[test]
    fn test_process_frame_null_handle() {
        let input = vec![1.0f32; N];
        let mut output = vec![0.0f32; N];
        let rc = unsafe {
            openllve_process_frame(
                std::ptr::null_mut(),
                W,
                H,
                C,
                STRIDE,
                input.as_ptr(),
                W,
                H,
                C,
                STRIDE,
                output.as_mut_ptr(),
            )
        };
        assert_eq!(rc, OpenLlveError::NullPointer.as_i32());
    }

    #[test]
    fn test_process_frame_output_dim_mismatch() {
        let handle = openllve_pipeline_new_temporal();
        assert!(!handle.is_null());

        let input = vec![1.0f32; N];
        // Output claims 2x2x3 but the strategy requires the same dims as input.
        let mut output = vec![0.0f32; N];
        let rc = unsafe {
            openllve_process_frame(
                handle,
                W,
                H,
                C,
                STRIDE,
                input.as_ptr(),
                2,
                2,
                C,
                (2 * C) as usize * 4,
                output.as_mut_ptr(),
            )
        };
        assert_eq!(rc, OpenLlveError::BufferDimensionMismatch.as_i32());

        unsafe { openllve_pipeline_free(handle) };
    }

    #[test]
    fn test_process_frame_stride_too_small() {
        let handle = openllve_pipeline_new_llie();
        assert!(!handle.is_null());

        let input = vec![1.0f32; N];
        let mut output = vec![0.0f32; N];
        // Stride smaller than width*channels*4 is an invalid parameter.
        let rc = unsafe {
            openllve_process_frame(
                handle,
                W,
                H,
                C,
                12,
                input.as_ptr(),
                W,
                H,
                C,
                STRIDE,
                output.as_mut_ptr(),
            )
        };
        assert_eq!(rc, OpenLlveError::InvalidParameter.as_i32());

        unsafe { openllve_pipeline_free(handle) };
    }

    #[test]
    fn test_ewma_filter_roundtrip() {
        let filter = openllve_ewma_filter_new(0.5);
        assert!(!filter.is_null());

        let input1 = vec![0.0f32; N];
        let mut out1 = vec![0.0f32; N];
        let rc = unsafe { openllve_ewma_filter_apply(filter, W, H, C, STRIDE, input1.as_ptr(), out1.as_mut_ptr()) };
        assert_eq!(rc, OpenLlveError::Ok.as_i32());
        assert_eq!(out1, input1);

        let input2 = vec![10.0f32; N];
        let mut out2 = vec![0.0f32; N];
        let rc = unsafe { openllve_ewma_filter_apply(filter, W, H, C, STRIDE, input2.as_ptr(), out2.as_mut_ptr()) };
        assert_eq!(rc, OpenLlveError::Ok.as_i32());
        assert!(out2.iter().all(|v| (v - 5.0).abs() < 1e-4));

        unsafe { openllve_ewma_filter_free(filter) };
    }

    #[test]
    fn test_blend_filter_roundtrip() {
        let filter = openllve_blend_filter_new(0.5);
        assert!(!filter.is_null());

        let raw = vec![0.0f32; N];
        let enhanced = vec![10.0f32; N];
        let mut output = vec![0.0f32; N];
        let rc = unsafe {
            openllve_blend_filter_apply(
                filter,
                W,
                H,
                C,
                STRIDE,
                raw.as_ptr(),
                enhanced.as_ptr(),
                output.as_mut_ptr(),
            )
        };
        assert_eq!(rc, OpenLlveError::Ok.as_i32());
        // 0.5 * enhanced + 0.5 * raw
        assert!(output.iter().all(|v| (v - 5.0).abs() < 1e-4));

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
