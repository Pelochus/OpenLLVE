package openllve.shared.log

/**
 * One log entry. [epochMillis] is UTC epoch milliseconds; display formatting
 * (local time zone) is the platform's job, since common Kotlin has no
 * timezone API.
 */
data class LogEntry(
    val epochMillis: Long,
    val level: LogLevel,
    val tag: String,
    val message: String,
)
