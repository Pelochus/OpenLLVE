package openllve.android.log

import android.util.Log
import java.time.Instant
import java.time.ZoneId
import java.time.format.DateTimeFormatter
import kotlin.time.Clock
import openllve.shared.log.LogBuffer
import openllve.shared.log.LogEntry
import openllve.shared.log.LogLevel

/**
 * Android-facing log facade over the shared [LogBuffer] (KMP common layer,
 * reused by the future iOS host with its own mirroring/formatting).
 *
 * - Records entries into the in-memory ring buffer (last [MAX_ENTRIES];
 *   RAM only, cleared on app restart — saving to a file from the settings
 *   screen keeps a copy).
 * - Mirrors every entry to logcat under [LOGCAT_TAG] (`adb logcat`).
 * - Formats timestamps in the device's local time zone for display.
 */
object AppLog {
    const val MAX_ENTRIES = 1000
    const val LOGCAT_TAG = "OpenLLVE"

    private val buffer = LogBuffer(MAX_ENTRIES)
    private val timeFormat = DateTimeFormatter.ofPattern("HH:mm:ss.SSS").withZone(ZoneId.systemDefault())

    fun debug(message: String, tag: String = "App") = record(LogLevel.DEBUG, tag, message)
    fun info(message: String, tag: String = "App") = record(LogLevel.INFO, tag, message)
    fun warn(message: String, tag: String = "App") = record(LogLevel.WARN, tag, message)
    fun error(message: String, tag: String = "App") = record(LogLevel.ERROR, tag, message)

    /** Snapshot of the buffer as display lines, oldest first. */
    fun snapshot(): List<String> = buffer.snapshot().map { formatLine(it) }

    val count: Int
        get() = buffer.size

    fun clear() = buffer.clear()

    private fun record(level: LogLevel, tag: String, message: String) {
        val entry = LogEntry(Clock.System.now().toEpochMilliseconds(), level, tag, message)
        buffer.record(entry)
        val line = formatLine(entry)
        when (level) {
            LogLevel.DEBUG -> Log.d(LOGCAT_TAG, line)
            LogLevel.INFO -> Log.i(LOGCAT_TAG, line)
            LogLevel.WARN -> Log.w(LOGCAT_TAG, line)
            LogLevel.ERROR -> Log.e(LOGCAT_TAG, line)
        }
    }

    private fun formatLine(entry: LogEntry): String =
        "${timeFormat.format(Instant.ofEpochMilli(entry.epochMillis))} [${entry.level.char}] ${entry.tag}: ${entry.message}"
}
