package dev.paw.android.runtime

import org.junit.Assert.assertEquals
import org.junit.Test

class PhoneNormalizerTest {

    @Test
    fun `TC-PHONE-01 blank input returns empty string`() {
        assertEquals("", normalizePhone(""))
        assertEquals("", normalizePhone("  "))
    }

    @Test
    fun `TC-PHONE-02 already prefixed with plus is returned as-is`() {
        assertEquals("+821012345678", normalizePhone("+821012345678"))
    }

    @Test
    fun `TC-PHONE-03 starts with 82 gets plus prefix`() {
        assertEquals("+821012345678", normalizePhone("821012345678"))
    }

    @Test
    fun `TC-PHONE-04 starts with 0 replaces leading zero with plus82`() {
        assertEquals("+821012345678", normalizePhone("01012345678"))
    }

    @Test
    fun `TC-PHONE-05 pure digits get plus82 prefix`() {
        assertEquals("+821012345678", normalizePhone("1012345678"))
    }
}
