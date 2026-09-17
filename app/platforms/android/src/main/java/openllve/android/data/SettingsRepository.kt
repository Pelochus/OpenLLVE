package openllve.android.data

import android.content.Context
import androidx.datastore.core.DataStore
import androidx.datastore.preferences.core.booleanPreferencesKey
import androidx.datastore.preferences.core.stringPreferencesKey
import androidx.datastore.preferences.core.Preferences
import androidx.datastore.preferences.core.edit
import androidx.datastore.preferences.preferencesDataStore
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.map
import openllve.android.domain.ComputeTarget
import openllve.android.domain.EnhancementSettings

/**
 * DataStore-backed settings. The [Context.settingsDataStore] extension property
 * is the canonical way to obtain the [DataStore] (the delegate's `getValue`
 * requires a [Context] as its this-reference, so it cannot be used as a plain
 * class property).
 */
val Context.settingsDataStore: DataStore<Preferences> by preferencesDataStore(name = "openllve_settings")

/**
 * Persists lightweight user preferences with Jetpack DataStore (Preferences).
 *
 * This is the modern, KMP-friendly persistence mechanism recommended for
 * simple key/value settings. No database is needed for this slice.
 */
class SettingsRepository(context: Context) {
    private val dataStore: DataStore<Preferences> = context.settingsDataStore

    private val computeTargetKey = stringPreferencesKey("compute_target")
    private val ewmaKey = booleanPreferencesKey("ewma_enabled")
    private val flickerKey = booleanPreferencesKey("flicker_reduction_enabled")

    /** Emits the persisted [EnhancementSettings] whenever they change. */
    fun observeSettings(): Flow<EnhancementSettings> = dataStore.data.map { prefs ->
        EnhancementSettings(
            computeTarget = parseTarget(prefs[computeTargetKey]),
            ewmaEnabled = prefs[ewmaKey] ?: false,
            flickerReductionEnabled = prefs[flickerKey] ?: false
        )
    }

    suspend fun updateSettings(settings: EnhancementSettings) {
        dataStore.edit { prefs ->
            prefs[computeTargetKey] = settings.computeTarget.name
            prefs[ewmaKey] = settings.ewmaEnabled
            prefs[flickerKey] = settings.flickerReductionEnabled
        }
    }

    private fun parseTarget(name: String?): ComputeTarget =
        runCatching { ComputeTarget.valueOf(name ?: ComputeTarget.CPU.name) }
            .getOrDefault(ComputeTarget.CPU)
}
