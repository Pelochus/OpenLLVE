package openllve.shared.domain

/**
 * Platform-neutral settings persistence contract: the single seam between
 * the shared [EnhancementSettings] domain type and a host's storage
 * (Jetpack DataStore on Android, UserDefaults/Core Data on iOS).
 *
 * Implementations own the storage format; consumers (ViewModels, and later
 * benchmark tooling) depend only on this interface.
 */
interface SettingsStore {
    /** Loads the persisted [EnhancementSettings] (defaults if none stored). */
    suspend fun load(): EnhancementSettings

    /** Persists [settings], replacing the previous values. */
    suspend fun save(settings: EnhancementSettings)
}
