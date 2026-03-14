package dev.paw.android.runtime

import android.util.Log
import com.google.firebase.messaging.FirebaseMessaging
import kotlinx.coroutines.suspendCancellableCoroutine
import uniffi.paw_core.PushPlatform
import uniffi.paw_core.PushRegistrationState
import uniffi.paw_core.PushRegistrationStatus
import kotlin.coroutines.resume
import kotlin.coroutines.resumeWithException

class FirebasePushRegistrar(
    private val apiClient: PawApiClient,
) {
    @Volatile
    private var state = PushRegistrationState(
        status = PushRegistrationStatus.UNREGISTERED,
        token = null,
        platform = PushPlatform.FCM,
        lastError = null,
        lastUpdatedMs = System.currentTimeMillis(),
    )

    fun currentState(): PushRegistrationState = state.copy()

    suspend fun register(accessToken: String?): PushRegistrationState {
        if (accessToken.isNullOrBlank()) {
            return updateState(
                status = PushRegistrationStatus.UNREGISTERED,
                token = null,
                error = "Missing access token for FCM registration",
            )
        }

        return try {
            apiClient.setAccessToken(accessToken)
            val token = awaitFirebaseToken()
            apiClient.registerPush(token)
            updateState(
                status = PushRegistrationStatus.REGISTERED,
                token = token,
                error = null,
            )
        } catch (error: Throwable) {
            Log.w(TAG, "Unable to register FCM token", error)
            updateState(
                status = PushRegistrationStatus.FAILED,
                token = null,
                error = error.message ?: error::class.simpleName ?: "push-registration-failed",
            )
        }
    }

    suspend fun unregister(accessToken: String?): PushRegistrationState {
        return try {
            if (!accessToken.isNullOrBlank()) {
                apiClient.setAccessToken(accessToken)
                apiClient.unregisterPush()
            }
            updateState(
                status = PushRegistrationStatus.UNREGISTERED,
                token = null,
                error = null,
            )
        } catch (error: Throwable) {
            Log.w(TAG, "Unable to unregister FCM token", error)
            updateState(
                status = PushRegistrationStatus.FAILED,
                token = state.token,
                error = error.message ?: error::class.simpleName ?: "push-unregister-failed",
            )
        }
    }

    private suspend fun awaitFirebaseToken(): String = suspendCancellableCoroutine { continuation ->
        FirebaseMessaging.getInstance().token
            .addOnSuccessListener { token -> continuation.resume(token) }
            .addOnFailureListener { error -> continuation.resumeWithException(error) }
    }

    private fun updateState(
        status: PushRegistrationStatus,
        token: String?,
        error: String?,
    ): PushRegistrationState {
        state = PushRegistrationState(
            status = status,
            token = token,
            platform = PushPlatform.FCM,
            lastError = error,
            lastUpdatedMs = System.currentTimeMillis(),
        )
        return currentState()
    }

    companion object {
        private const val TAG = "FirebasePushRegistrar"
    }
}
