package dev.paw.android.domain.model

/**
 * Normalizes phone number input to E.164-like format with Korean country code (+82).
 *
 * Rules:
 * - Blank input -> empty string
 * - Already starts with "+" -> returned as-is (trimmed)
 * - Starts with "82" -> prepend "+"
 * - Starts with "0" -> replace leading 0 with "+82"
 * - Otherwise -> prepend "+82"
 */
fun normalizePhone(input: String): String {
    val trimmed = input.trim()
    if (trimmed.isBlank()) return ""
    if (trimmed.startsWith("+")) return trimmed

    val digits = trimmed.filter(Char::isDigit)
    if (digits.isBlank()) return ""

    return when {
        digits.startsWith("82") -> "+$digits"
        digits.startsWith("0") -> "+82${digits.drop(1)}"
        else -> "+82$digits"
    }
}
