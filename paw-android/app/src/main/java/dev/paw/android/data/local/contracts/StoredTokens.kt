package dev.paw.android.data.local.contracts

/**
 * Value object holding an access/refresh token pair.
 */
data class StoredTokens(
    val accessToken: String,
    val refreshToken: String,
)

/**
 * Platform-independent contract for secure token storage.
 * Implementations handle encryption/decryption details.
 */
interface TokenVault {
    fun read(): StoredTokens?
    fun write(tokens: StoredTokens)
    fun clear()
}
