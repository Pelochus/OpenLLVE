package openllve.shared.log

import java.util.concurrent.locks.ReentrantLock

internal actual class LogLock {
    private val lock = ReentrantLock()

    actual fun <T> withLock(block: () -> T): T {
        lock.lock()
        try {
            return block()
        } finally {
            lock.unlock()
        }
    }
}
