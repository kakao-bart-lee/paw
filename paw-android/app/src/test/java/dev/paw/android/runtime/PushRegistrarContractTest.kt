package dev.paw.android.runtime

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Before
import org.junit.Test
import uniffi.paw_core.PushPlatform
import uniffi.paw_core.PushRegistrationState
import uniffi.paw_core.PushRegistrationStatus

/**
 * In-memory implementation of PushRegistrarContract for JVM unit testing.
 * Simulates register/unregister without Firebase dependencies.
 */
private class InMemoryPushRegistrar(
    private val platform: PushPlatform,
) : PushRegistrarContract {
    private var state = PushRegistrationState(
        status = PushRegistrationStatus.UNREGISTERED,
        token = null,
        platform = platform,
        lastError = null,
        lastUpdatedMs = 0L,
    )

    override suspend fun register(accessToken: String?): PushRegistrationState {
        if (accessToken.isNullOrBlank()) {
            state = state.copy(
                status = PushRegistrationStatus.FAILED,
                lastError = "Missing access token",
            )
            return currentState()
        }
        state = PushRegistrationState(
            status = PushRegistrationStatus.REGISTERED,
            token = "fake-push-token-$accessToken",
            platform = platform,
            lastError = null,
            lastUpdatedMs = System.currentTimeMillis(),
        )
        return currentState()
    }

    override suspend fun unregister(accessToken: String?): PushRegistrationState {
        state = PushRegistrationState(
            status = PushRegistrationStatus.UNREGISTERED,
            token = null,
            platform = platform,
            lastError = null,
            lastUpdatedMs = System.currentTimeMillis(),
        )
        return currentState()
    }

    override fun currentState(): PushRegistrationState = state.copy()
}

/**
 * Tests the PushRegistrarContract using an in-memory implementation.
 * Covers TC-PUSH-01 through TC-PUSH-03.
 */
class PushRegistrarContractTest {

    private lateinit var registrar: PushRegistrarContract

    @Before
    fun setUp() {
        registrar = InMemoryPushRegistrar(PushPlatform.FCM)
    }

    @Test
    fun `TC-PUSH-01 initial state is unregistered`() {
        val state = registrar.currentState()
        assertEquals(PushRegistrationStatus.UNREGISTERED, state.status)
        assertNull(state.token)
    }

    @Test
    fun `TC-PUSH-02 register then unregister cycle`() {
        kotlinx.coroutines.runBlocking {
            val registered = registrar.register("valid-access-token")
            assertEquals(PushRegistrationStatus.REGISTERED, registered.status)
            assertEquals("fake-push-token-valid-access-token", registered.token)

            val afterRegister = registrar.currentState()
            assertEquals(PushRegistrationStatus.REGISTERED, afterRegister.status)

            val unregistered = registrar.unregister("valid-access-token")
            assertEquals(PushRegistrationStatus.UNREGISTERED, unregistered.status)
            assertNull(unregistered.token)

            val afterUnregister = registrar.currentState()
            assertEquals(PushRegistrationStatus.UNREGISTERED, afterUnregister.status)
        }
    }

    @Test
    fun `TC-PUSH-03 platform tag is fcm for Android`() {
        val state = registrar.currentState()
        assertEquals(PushPlatform.FCM, state.platform)
    }

    @Test
    fun `TC-PUSH-03 platform tag preserved after registration`() {
        kotlinx.coroutines.runBlocking {
            registrar.register("token")
            assertEquals(PushPlatform.FCM, registrar.currentState().platform)

            registrar.unregister(null)
            assertEquals(PushPlatform.FCM, registrar.currentState().platform)
        }
    }
}
