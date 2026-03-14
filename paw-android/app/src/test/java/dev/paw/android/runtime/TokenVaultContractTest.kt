package dev.paw.android.runtime

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNotNull
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test

/**
 * In-memory implementation of TokenVault for JVM unit testing.
 * No Android or Keystore dependencies.
 */
private class InMemoryTokenVault : TokenVault {
    private var stored: StoredTokens? = null

    override fun read(): StoredTokens? = stored?.let { StoredTokens(it.accessToken, it.refreshToken) }

    override fun write(tokens: StoredTokens) {
        stored = StoredTokens(tokens.accessToken, tokens.refreshToken)
    }

    override fun clear() {
        stored = null
    }
}

/**
 * Tests the TokenVault contract using an in-memory implementation.
 * Covers TC-TOKEN-01 through TC-TOKEN-03.
 */
class TokenVaultContractTest {

    private lateinit var vault: TokenVault

    @Before
    fun setUp() {
        vault = InMemoryTokenVault()
    }

    @Test
    fun `TC-TOKEN-01 round-trip write read and clear`() {
        val tokens = StoredTokens(accessToken = "access-abc", refreshToken = "refresh-xyz")

        vault.write(tokens)

        val retrieved = vault.read()
        assertNotNull(retrieved)
        assertEquals("access-abc", retrieved!!.accessToken)
        assertEquals("refresh-xyz", retrieved.refreshToken)

        vault.clear()

        assertNull(vault.read())
    }

    @Test
    fun `TC-TOKEN-02 overwrite replaces previous tokens`() {
        val original = StoredTokens(accessToken = "old-access", refreshToken = "old-refresh")
        val updated = StoredTokens(accessToken = "new-access", refreshToken = "new-refresh")

        vault.write(original)
        vault.write(updated)

        val retrieved = vault.read()
        assertNotNull(retrieved)
        assertEquals("new-access", retrieved!!.accessToken)
        assertEquals("new-refresh", retrieved.refreshToken)
    }

    @Test
    fun `TC-TOKEN-03 read from empty vault returns null`() {
        assertNull(vault.read())
    }
}
