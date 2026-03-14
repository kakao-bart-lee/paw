@file:Suppress("unused")
package dev.paw.android.runtime

/**
 * Backward-compatibility delegate. New code should import from domain.model directly.
 */
fun normalizePhone(input: String): String = dev.paw.android.domain.model.normalizePhone(input)
