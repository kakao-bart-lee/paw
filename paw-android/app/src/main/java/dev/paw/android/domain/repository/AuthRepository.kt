package dev.paw.android.domain.repository

import org.json.JSONObject
import uniffi.paw_core.DeviceKeyMaterial
import uniffi.paw_core.PushRegistrationState

/**
 * Repository interface for all authentication-related operations.
 * Coordinates between API client, token vault, device key store, and push registrar.
 */
interface AuthRepository {
    fun setAccessToken(token: String?)
    fun readTokens(): dev.paw.android.data.local.contracts.StoredTokens?
    fun writeTokens(tokens: dev.paw.android.data.local.contracts.StoredTokens)
    fun clearTokens()
    fun storageCapabilities(): uniffi.paw_core.SecureStorageCapabilities
    fun loadDeviceKey(): DeviceKeyMaterial?
    fun ensureDeviceKey(): DeviceKeyMaterial

    suspend fun requestOtp(phone: String): JSONObject
    suspend fun verifyOtp(phone: String, code: String): JSONObject
    suspend fun registerDevice(sessionToken: String, deviceName: String, ed25519PublicKey: String): JSONObject
    suspend fun getMe(): JSONObject
    suspend fun updateMe(username: String, discoverableByPhone: Boolean): JSONObject

    suspend fun refreshPush(accessToken: String?): PushRegistrationState
    suspend fun unregisterPush(accessToken: String?): PushRegistrationState
    fun currentPushState(): PushRegistrationState
}
