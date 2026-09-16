package openllve.android

import android.app.Application
import openllve.android.data.SettingsRepository
import openllve.android.domain.EnhancementEngine
import openllve.android.engine.AndroidLiteRtEngine

/**
 * Application-scoped wiring.
 *
 * Deliberately lightweight (no DI framework): the two long-lived components
 * (the enhancement engine and the settings repository) are created once and
 * handed to the Activity/ViewModels. The engine is the seam that will later be
 * swapped for a Rust-backed implementation without touching the UI.
 */
class OpenLLVEApp : Application() {

    lateinit var enhancementEngine: EnhancementEngine
    lateinit var settingsRepository: SettingsRepository

    override fun onCreate() {
        super.onCreate()
        // Temporary Android LiteRT implementation. Replace with a Rust-backed
        // engine (via the C FFI) in a later phase; the UI depends only on
        // EnhancementEngine, so this is the only line that changes.
        enhancementEngine = AndroidLiteRtEngine()
        settingsRepository = SettingsRepository(this)
    }
}
