package openllve.shared.log

/**
 * Minimal cross-platform mutual exclusion for [LogBuffer].
 *
 * Android actual: `java.util.concurrent.locks.ReentrantLock`; iOS actual:
 * `platform.Foundation.NSLock`.
 */
internal expect class LogLock() {
    fun <T> withLock(block: () -> T): T
}
