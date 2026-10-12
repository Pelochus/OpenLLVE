//! JNI exports for the Kotlin app layer.
//!
//! Each export is an instance method of `openllve.android.core.CoreBridge`, so
//! the symbols defined here (`Java_openllve_android_core_CoreBridge_<method>`)
//! are what Kotlin's `@JNI external fun` declarations resolve against. Rust
//! names are `snake_case`; `jni_mangle` converts them to Kotlin `lowerCamelCase`.
//!
//! Handles are JVM `jlong` (Kotlin `Long`, never `ULong`), frames are JVM
//! `jfloatarray` (Kotlin `FloatArray`), and strides are bytes per row. Core-level
//! failures return an [`OpenLlveError`] code; JNI-level failures (a bad array
//! reference, a panic) throw a `java.lang.RuntimeException` into Kotlin.

use jni::EnvUnowned;
use jni::{
    elements::ReleaseMode,
    errors::{Result, ThrowRuntimeExAndDefault},
    objects::{JFloatArray, JObject, JString},
    sys::{jdouble, jfloat, jint, jlong},
};
use jni_macros::jni_mangle;
use std::os::raw::c_char;

use openllve_core::ffi;
use openllve_core::ffi::OpenLlvePipeline;
use openllve_core::{BenchmarkMetrics, EwmaFilter, FrameBlendFilter};

#[jni_mangle("openllve.android.core.CoreBridge")]
pub extern "system" fn abi_version<'local>(_env: EnvUnowned<'local>, _this: JObject<'local>) -> jint {
    ffi::openllve_abi_version() as jint
}

/// Handle-only exports take no `Env` upgrade: they move plain `jlong` values and
/// never touch JNI handles, so the JNI handle context is not needed.
#[jni_mangle("openllve.android.core.CoreBridge")]
pub extern "system" fn pipeline_new_llie<'local>(_env: EnvUnowned<'local>, _this: JObject<'local>) -> jlong {
    ffi::openllve_pipeline_new_llie() as jlong
}

#[jni_mangle("openllve.android.core.CoreBridge")]
pub extern "system" fn pipeline_new_temporal<'local>(_env: EnvUnowned<'local>, _this: JObject<'local>) -> jlong {
    ffi::openllve_pipeline_new_temporal() as jlong
}

#[jni_mangle("openllve.android.core.CoreBridge")]
pub extern "system" fn pipeline_new_llie_with_model<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _this: JObject<'local>,
    model_path: JString<'local>,
    num_threads: jint,
) -> jlong {
    unowned_env
        .with_env(|env| -> Result<jlong> {
            // The terminator has to stay alive for the duration of the call, so the
            // string is built here rather than in a helper.
            let path = model_path.try_to_string(env)? + "\0";
            Ok(unsafe {
                ffi::openllve_pipeline_new_llie_with_model(path.as_ptr() as *const c_char, num_threads) as jlong
            })
        })
        .resolve::<ThrowRuntimeExAndDefault>()
}

#[jni_mangle("openllve.android.core.CoreBridge")]
pub extern "system" fn pipeline_free<'local>(_env: EnvUnowned<'local>, _this: JObject<'local>, pipeline: jlong) {
    unsafe { ffi::openllve_pipeline_free(pipeline as *mut OpenLlvePipeline) };
}

#[jni_mangle("openllve.android.core.CoreBridge")]
pub extern "system" fn process_frame<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _this: JObject<'local>,
    pipeline: jlong,
    in_width: jint,
    in_height: jint,
    in_channels: jint,
    in_stride: jint,
    in_data: JFloatArray<'local>,
    out_width: jint,
    out_height: jint,
    out_channels: jint,
    out_stride: jint,
    out_data: JFloatArray<'local>,
) -> jint {
    unowned_env
        .with_env(|env| -> Result<jint> {
            let input = unsafe { in_data.get_elements(env, ReleaseMode::NoCopyBack) }?;
            let output = unsafe { out_data.get_elements(env, ReleaseMode::CopyBack) }?;

            let in_violation =
                crate::frames::frame_violation(in_width, in_height, in_channels, in_stride, &[input.len()]);
            let out_violation = crate::frames::frame_violation(
                out_width,
                out_height,
                out_channels,
                out_stride,
                &[output.len()],
            );
            if in_violation != 0 {
                return Ok(in_violation);
            }
            if out_violation != 0 {
                return Ok(out_violation);
            }

            Ok(unsafe {
                ffi::openllve_process_frame(
                    pipeline as *mut OpenLlvePipeline,
                    in_width as u32,
                    in_height as u32,
                    in_channels as u32,
                    in_stride as usize,
                    input.as_ptr(),
                    out_width as u32,
                    out_height as u32,
                    out_channels as u32,
                    out_stride as usize,
                    output.as_ptr(),
                )
            })
        })
        .resolve::<ThrowRuntimeExAndDefault>()
}

#[jni_mangle("openllve.android.core.CoreBridge")]
pub extern "system" fn ewma_filter_new<'local>(
    _env: EnvUnowned<'local>,
    _this: JObject<'local>,
    alpha: jfloat,
) -> jlong {
    ffi::openllve_ewma_filter_new(alpha) as jlong
}

#[jni_mangle("openllve.android.core.CoreBridge")]
pub extern "system" fn ewma_filter_apply<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _this: JObject<'local>,
    filter: jlong,
    width: jint,
    height: jint,
    channels: jint,
    stride: jint,
    in_data: JFloatArray<'local>,
    out_data: JFloatArray<'local>,
) -> jint {
    unowned_env
        .with_env(|env| -> Result<jint> {
            let input = unsafe { in_data.get_elements(env, ReleaseMode::NoCopyBack) }?;
            let output = unsafe { out_data.get_elements(env, ReleaseMode::CopyBack) }?;
            let violation = crate::frames::frame_violation(
                width,
                height,
                channels,
                stride,
                &[input.len(), output.len()],
            );
            if violation != 0 {
                return Ok(violation);
            }

            Ok(unsafe {
                ffi::openllve_ewma_filter_apply(
                    filter as *mut EwmaFilter,
                    width as u32,
                    height as u32,
                    channels as u32,
                    stride as usize,
                    input.as_ptr(),
                    output.as_ptr(),
                )
            })
        })
        .resolve::<ThrowRuntimeExAndDefault>()
}

#[jni_mangle("openllve.android.core.CoreBridge")]
pub extern "system" fn ewma_filter_free<'local>(_env: EnvUnowned<'local>, _this: JObject<'local>, filter: jlong) {
    unsafe { ffi::openllve_ewma_filter_free(filter as *mut EwmaFilter) };
}

#[jni_mangle("openllve.android.core.CoreBridge")]
pub extern "system" fn blend_filter_new<'local>(
    _env: EnvUnowned<'local>,
    _this: JObject<'local>,
    beta: jfloat,
) -> jlong {
    ffi::openllve_blend_filter_new(beta) as jlong
}

#[jni_mangle("openllve.android.core.CoreBridge")]
pub extern "system" fn blend_filter_apply<'local>(
    mut unowned_env: EnvUnowned<'local>,
    _this: JObject<'local>,
    filter: jlong,
    width: jint,
    height: jint,
    channels: jint,
    stride: jint,
    raw_data: JFloatArray<'local>,
    enhanced_data: JFloatArray<'local>,
    out_data: JFloatArray<'local>,
) -> jint {
    unowned_env
        .with_env(|env| -> Result<jint> {
            let raw = unsafe { raw_data.get_elements(env, ReleaseMode::NoCopyBack) }?;
            let enhanced = unsafe { enhanced_data.get_elements(env, ReleaseMode::NoCopyBack) }?;
            let output = unsafe { out_data.get_elements(env, ReleaseMode::CopyBack) }?;
            let violation = crate::frames::frame_violation(
                width,
                height,
                channels,
                stride,
                &[raw.len(), enhanced.len(), output.len()],
            );
            if violation != 0 {
                return Ok(violation);
            }

            Ok(unsafe {
                ffi::openllve_blend_filter_apply(
                    filter as *mut FrameBlendFilter,
                    width as u32,
                    height as u32,
                    channels as u32,
                    stride as usize,
                    raw.as_ptr(),
                    enhanced.as_ptr(),
                    output.as_ptr(),
                )
            })
        })
        .resolve::<ThrowRuntimeExAndDefault>()
}

#[jni_mangle("openllve.android.core.CoreBridge")]
pub extern "system" fn blend_filter_free<'local>(_env: EnvUnowned<'local>, _this: JObject<'local>, filter: jlong) {
    unsafe { ffi::openllve_blend_filter_free(filter as *mut FrameBlendFilter) };
}

#[jni_mangle("openllve.android.core.CoreBridge")]
pub extern "system" fn metrics_new<'local>(_env: EnvUnowned<'local>, _this: JObject<'local>) -> jlong {
    ffi::openllve_metrics_new() as jlong
}

#[jni_mangle("openllve.android.core.CoreBridge")]
pub extern "system" fn metrics_new_with_warmup<'local>(
    _env: EnvUnowned<'local>,
    _this: JObject<'local>,
    warmup_frames: jint,
) -> jlong {
    // A negative warm-up budget is a caller error: hand back the null handle the
    // C ABI uses for construction failures rather than an absurd budget.
    if warmup_frames < 0 {
        return 0;
    }
    ffi::openllve_metrics_new_with_warmup(warmup_frames as usize) as jlong
}

#[jni_mangle("openllve.android.core.CoreBridge")]
pub extern "system" fn metrics_record<'local>(
    _env: EnvUnowned<'local>,
    _this: JObject<'local>,
    metrics: jlong,
    latency_ms: jdouble,
) {
    unsafe { ffi::openllve_metrics_record(metrics as *mut BenchmarkMetrics, latency_ms) };
}

#[jni_mangle("openllve.android.core.CoreBridge")]
pub extern "system" fn metrics_average<'local>(
    _env: EnvUnowned<'local>,
    _this: JObject<'local>,
    metrics: jlong,
) -> jdouble {
    unsafe { ffi::openllve_metrics_get_average(metrics as *const BenchmarkMetrics) }
}

#[jni_mangle("openllve.android.core.CoreBridge")]
pub extern "system" fn metrics_median<'local>(
    _env: EnvUnowned<'local>,
    _this: JObject<'local>,
    metrics: jlong,
) -> jdouble {
    unsafe { ffi::openllve_metrics_get_median(metrics as *const BenchmarkMetrics) }
}

#[jni_mangle("openllve.android.core.CoreBridge")]
pub extern "system" fn metrics_fps<'local>(
    _env: EnvUnowned<'local>,
    _this: JObject<'local>,
    metrics: jlong,
) -> jdouble {
    unsafe { ffi::openllve_metrics_get_fps(metrics as *const BenchmarkMetrics) }
}

#[jni_mangle("openllve.android.core.CoreBridge")]
pub extern "system" fn metrics_p99<'local>(
    _env: EnvUnowned<'local>,
    _this: JObject<'local>,
    metrics: jlong,
) -> jdouble {
    unsafe { ffi::openllve_metrics_get_p99(metrics as *const BenchmarkMetrics) }
}

#[jni_mangle("openllve.android.core.CoreBridge")]
pub extern "system" fn metrics_count<'local>(_env: EnvUnowned<'local>, _this: JObject<'local>, metrics: jlong) -> jint {
    unsafe { ffi::openllve_metrics_get_count(metrics as *const BenchmarkMetrics) as jint }
}

#[jni_mangle("openllve.android.core.CoreBridge")]
pub extern "system" fn metrics_reset<'local>(_env: EnvUnowned<'local>, _this: JObject<'local>, metrics: jlong) {
    unsafe { ffi::openllve_metrics_reset(metrics as *mut BenchmarkMetrics) };
}

#[jni_mangle("openllve.android.core.CoreBridge")]
pub extern "system" fn metrics_free<'local>(_env: EnvUnowned<'local>, _this: JObject<'local>, metrics: jlong) {
    unsafe { ffi::openllve_metrics_free(metrics as *mut BenchmarkMetrics) };
}
