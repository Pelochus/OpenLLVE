package openllve.android.data

import android.content.Context
import androidx.datastore.core.DataStore
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.core.floatPreferencesKey
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.datastore.preferences.preferencesDataStore
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import openllve.shared.domain.ComputeTarget
import openllve.shared.domain.EnhancementSettings
import openllve.shared.domain.SettingsStore

/**
 * DataStore-backed settings. The delegate's `getValue` needs a [Context]
 * this-reference, so the store is obtained via this extension property.
 */
val Context.settingsDataStore: DataStore<Preferences> by preferencesDataStore(name = "openllve_settings")

/**
 * Persists user preferences with Jetpack DataStore (key/value, no database)
 * and implements the platform-neutral [SettingsStore] contract from the
 * shared layer.
 */
class SettingsRepository(
    context: Context,
) : SettingsStore {
    private val dataStore: DataStore<Preferences> = context.settingsDataStore

    private val computeTargetKey = stringPreferencesKey("compute_target")
    private val ewmaKey = booleanPreferencesKey("ewma_enabled")
    private val flickerKey = booleanPreferencesKey("flicker_reduction_enabled")
    private val ewmaAlphaKey = floatPreferencesKey("ewma_alpha")
    private val blendBetaKey = floatPreferencesKey("blend_beta")

    /** Emits the persisted [EnhancementSettings] whenever they change. */
    fun observeSettings(): Flow<EnhancementSettings> =
        dataStore.data.map { prefs ->
            EnhancementSettings(
                computeTarget = parseTarget(prefs[computeTargetKey]),
                ewmaEnabled = prefs[ewmaKey] ?: false,
                flickerReductionEnabled = prefs[flickerKey] ?: false,
                ewmaAlpha =
                    (prefs[ewmaAlphaKey] ?: EnhancementSettings().ewmaAlpha)
                        .coerceIn(EWMA_ALPHA_MIN..1f),
                blendBeta =
                    (prefs[blendBetaKey] ?: EnhancementSettings().blendBeta)
                        .coerceIn(0f..1f),
            )
        }

    override suspend fun load(): EnhancementSettings = observeSettings().first()

    override suspend fun save(settings: EnhancementSettings) {
        dataStore.edit { prefs ->
            prefs[computeTargetKey] = settings.computeTarget.name
            prefs[ewmaKey] = settings.ewmaEnabled
            prefs[flickerKey] = settings.flickerReductionEnabled
            prefs[ewmaAlphaKey] = settings.ewmaAlpha.coerceIn(EWMA_ALPHA_MIN..1f)
            prefs[blendBetaKey] = settings.blendBeta.coerceIn(0f..1f)
        }
    }

    private fun parseTarget(name: String?): ComputeTarget =
        runCatching { ComputeTarget.valueOf(name ?: ComputeTarget.CPU.name) }
            .getOrDefault(ComputeTarget.CPU)

    private companion object {
        private const val EWMA_ALPHA_MIN = 0.25f
    }
}
