package openllve.android

import android.app.Application
import openllve.android.data.SettingsRepository
import openllve.android.engine.AndroidLiteRtEngine
import openllve.shared.domain.EnhancementEngine

/**
 * Application-scoped wiring: the engine and settings repository are created
 * once and handed to the Activity/ViewModels. The engine is the seam that
 * will later be swapped for a Rust-backed implementation.
 */
class OpenLLVEApp : Application() {
    lateinit var enhancementEngine: EnhancementEngine
    lateinit var settingsRepository: SettingsRepository

    override fun onCreate() {
        super.onCreate()
        // Temporary LiteRT implementation; a Rust-backed engine replaces it later.
        enhancementEngine = AndroidLiteRtEngine(this)
        settingsRepository = SettingsRepository(this)
    }
}
