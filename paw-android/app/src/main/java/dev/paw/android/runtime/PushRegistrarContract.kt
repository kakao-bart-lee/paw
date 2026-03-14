package dev.paw.android.runtime

import uniffi.paw_core.PushRegistrationState

/**
 * Platform-independent contract for push notification registration.
 * Implementations handle platform-specific push services (FCM, APNs).
 */
interface PushRegistrarContract {
    suspend fun register(accessToken: String?): PushRegistrationState
    suspend fun unregister(accessToken: String?): PushRegistrationState
    fun currentState(): PushRegistrationState
}
