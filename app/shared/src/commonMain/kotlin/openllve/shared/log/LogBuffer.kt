package openllve.shared.log

/**
 * Thread-safe in-memory ring buffer of [LogEntry]s (RAM only; cleared on
 * process restart). Shared between the Android and (future) iOS hosts; each
 * platform mirrors entries to its native logging (logcat / os.log) and
 * formats timestamps for display.
 */
class LogBuffer(
    private val capacity: Int = 1000,
) {
    private val entries = ArrayDeque<LogEntry>()
    private val lock = LogLock()

    fun record(entry: LogEntry) {
        lock.withLock {
            entries.addLast(entry)
            while (entries.size > capacity) {
                entries.removeFirst()
            }
        }
    }

    /** Snapshot, oldest first. */
    fun snapshot(): List<LogEntry> = lock.withLock { entries.toList() }

    fun clear() = lock.withLock { entries.clear() }

    val size: Int
        get() = lock.withLock { entries.size }
}
