package openllve.android.core

import openllve.android.log.AppLog
import openllve.shared.domain.EnhancementSettings

/**
 * The app-facing temporal toppings, run by the Rust core through [CoreBridge].
 *
 * A `CoreToppings` instance owns the native filter handles for one media
 * session: build it after the engine is configured, call [apply] per frame, and
 * [close] once when the session ends. Filters keep state across frames (EWMA
 * remembers the previous filtered frame), which is why they cannot be created
 * per frame.
 *
 * A core error never propagates: the frame is returned unsmoothed and the code
 * is logged once, so a broken library degrades the picture instead of ending
 * the run. A library whose ABI disagrees with this contract disables both
 * toppings at session start (the committed `libbridge.so` can otherwise drift
 * silently out of step with `core/src/ffi.rs`).
 */
class CoreToppings(
    private val settings: EnhancementSettings,
    private val width: Int,
    private val height: Int,
) {
    companion object {
        private const val CHANNELS = 3

        // EWMA_ALPHA weights the CURRENT frame (core/src/filters/ewma.rs:
        // out = alpha*current + (1-alpha)*previous); 0.8 keeps most of the
        // current frame so ghosting stays invisible on video while still
        // damping frame-to-frame shimmer. BLEND_BETA weights the
        // AI-enhanced frame — 0.9 keeps the picture mostly model output. P1.8
        // promotes both to `EnhancementSettings`; core/tests/filters_properties.rs
        // pins this pair in a golden-frame hash.
        private const val EWMA_ALPHA = 0.8f
        private const val BLEND_BETA = 0.9f

        // Bytes per row for a tightly packed RGB `FloatArray` (4 bytes per element).
        private const val STRIDE_SCALE = CHANNELS * 4

        // Must match core/src/ffi.rs OPENLLVE_ABI_VERSION; scripts/build-bridge.sh
        // rebuilds the library, this constant is what catches a stale one.
        private const val CORE_ABI_VERSION = 3
    }

    private var coreLoaded = false
    private var coreUnavailable = false
    private var ewmaCodeLogged = false
    private var blendCodeLogged = false

    private val ewmaHandle = newEwmaHandle()
    private val blendHandle = newBlendHandle()

    /** JNI bindings are instance members of [CoreBridge], so one is held per session. */
    private val bridge = CoreBridge()
    private val blended = FloatArray(width * height * CHANNELS)
    private val smoothed = FloatArray(width * height * CHANNELS)

    /**
     * Runs the enabled toppings over one frame.
     *
     * @param raw the model's input frame, row-major RGB floats in `[0, 1]`
     * @param enhanced the model's output frame, same layout
     * @return the topped frame — the same buffer every call when a topping runs,
     * so the caller must consume it before the next call
     */
    fun apply(
        raw: FloatArray,
        enhanced: FloatArray,
    ): FloatArray {
        var result = enhanced
        if (blendHandle != 0L) {
            val blendCode =
                bridge.blendFilterApply(
                    blendHandle,
                    width,
                    height,
                    CHANNELS,
                    width * STRIDE_SCALE,
                    raw,
                    result,
                    blended,
                )
            if (blendCode != 0) {
                logCoreFailure("blend", blendCode)
            } else {
                result = blended
            }
        }
        if (ewmaHandle != 0L) {
            val ewmaCode =
                bridge.ewmaFilterApply(
                    ewmaHandle,
                    width,
                    height,
                    CHANNELS,
                    width * STRIDE_SCALE,
                    result,
                    smoothed,
                )
            if (ewmaCode != 0) {
                logCoreFailure("ewma", ewmaCode)
            } else {
                result = smoothed
            }
        }
        return result
    }

    /** Releases the native handles. Safe to call more than once. */
    fun close() {
        if (ewmaHandle != 0L) bridge.ewmaFilterFree(ewmaHandle)
        if (blendHandle != 0L) bridge.blendFilterFree(blendHandle)
    }

    private fun newEwmaHandle(): Long {
        if (!settings.ewmaEnabled) return 0L
        if (!coreReady()) return 0L
        return bridge.ewmaFilterNew(EWMA_ALPHA)
    }

    private fun newBlendHandle(): Long {
        if (!settings.flickerReductionEnabled) return 0L
        if (!coreReady()) return 0L
        return bridge.blendFilterNew(BLEND_BETA)
    }

    /**
     * Loads `libbridge.so` once per session and probes its ABI. JNI symbols
     * resolve lazily at first call, so without the probe a wrong library would
     * only fail deep inside a frame; `abiVersion()` is the startup check.
     *
     * @return false when the library speaks an unexpected ABI, in which case no
     * handles are created and frames pass through untouched
     */
    private fun coreReady(): Boolean {
        if (!coreLoaded) {
            coreLoaded = true
            // scripts/build-bridge.sh renames Cargo's artifact (libopenllve_core_bridge.so)
            // to libbridge.so in jniLibs, which is the name the JVM resolves.
            System.loadLibrary("bridge")
            val abi = bridge.abiVersion()
            if (abi != CORE_ABI_VERSION) {
                AppLog.error(
                    "Core: bridge ABI $abi != expected $CORE_ABI_VERSION, " +
                        "toppings disabled",
                    "Core",
                )
                coreUnavailable = true
            }
        }
        return !coreUnavailable
    }

    /** Logs a core rejection once per filter, not once per frame. */
    private fun logCoreFailure(
        filter: String,
        code: Int,
    ) {
        if (filter == "ewma") {
            if (ewmaCodeLogged) return
            ewmaCodeLogged = true
        } else {
            if (blendCodeLogged) return
            blendCodeLogged = true
        }
        AppLog.error(
            "Core: $filter filter rejected the frame (code $code), " +
                "frame passed through unsmoothed",
            "Core",
        )
    }
}
