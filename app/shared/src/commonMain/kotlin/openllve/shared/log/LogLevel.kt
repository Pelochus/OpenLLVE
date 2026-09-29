package openllve.shared.log

/**
 * Log severity levels shared between the Android and (future) iOS hosts.
 */
enum class LogLevel(val char: Char) {
    DEBUG('D'),
    INFO('I'),
    WARN('W'),
    ERROR('E'),
}
