package dev.paw.android.data.repository

import dev.paw.android.data.local.AndroidSecureTokenVault
import dev.paw.android.data.local.contracts.DeviceKeyStoreContract
import dev.paw.android.data.local.contracts.StoredTokens
import dev.paw.android.data.remote.ApiClientContract
import dev.paw.android.domain.repository.AuthRepository
import dev.paw.android.runtime.PushRegistrarContract
import org.json.JSONObject
import uniffi.paw_core.DeviceKeyMaterial
import uniffi.paw_core.PushRegistrationState
import uniffi.paw_core.SecureStorageCapabilities

class AuthRepositoryImpl(
    private val apiClient: ApiClientContract,
    private val tokenVault: AndroidSecureTokenVault,
    private val deviceKeyStore: DeviceKeyStoreContract,
    private val pushRegistrar: PushRegistrarContract,
) : AuthRepository {

    override fun setAccessToken(token: String?) {
        apiClient.setAccessToken(token)
    }

    override fun readTokens(): StoredTokens? = tokenVault.read()

    override fun writeTokens(tokens: StoredTokens) {
        tokenVault.write(tokens)
    }

    override fun clearTokens() {
        tokenVault.clear()
    }

    override fun storageCapabilities(): SecureStorageCapabilities = tokenVault.capabilities()

    override fun loadDeviceKey(): DeviceKeyMaterial? = deviceKeyStore.load()

    override fun ensureDeviceKey(): DeviceKeyMaterial = deviceKeyStore.loadOrCreate()

    override suspend fun requestOtp(phone: String): JSONObject = apiClient.requestOtp(phone)

    override suspend fun verifyOtp(phone: String, code: String): JSONObject =
        apiClient.verifyOtp(phone, code)

    override suspend fun registerDevice(
        sessionToken: String,
        deviceName: String,
        ed25519PublicKey: String,
    ): JSONObject = apiClient.registerDevice(sessionToken, deviceName, ed25519PublicKey)

    override suspend fun getMe(): JSONObject = apiClient.getMe()

    override suspend fun updateMe(username: String, discoverableByPhone: Boolean): JSONObject =
        apiClient.updateMe(username, discoverableByPhone)

    override suspend fun refreshPush(accessToken: String?): PushRegistrationState =
        pushRegistrar.register(accessToken)

    override suspend fun unregisterPush(accessToken: String?): PushRegistrationState =
        pushRegistrar.unregister(accessToken)

    override fun currentPushState(): PushRegistrationState = pushRegistrar.currentState()
}
