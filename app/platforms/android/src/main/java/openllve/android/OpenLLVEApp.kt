package openllve.android

import android.app.Application
import com.example.openllve.BuildConfig
import openllve.android.data.SettingsRepository
import openllve.android.engine.AndroidLiteRtEngine
import openllve.android.log.AppLog
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
        AppLog.info("App initialized (version ${BuildConfig.VERSION_NAME})", "App")
        AppLog.info("Engine: AndroidLiteRtEngine (LiteRT 2.2.0, model ${enhancementEngine.modelName})", "App")
    }
}
