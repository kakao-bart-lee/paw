package dev.paw.android.runtime

import android.content.Context
import android.content.SharedPreferences
import android.os.Build
import uniffi.paw_core.DeviceKeyMaterial
import uniffi.paw_core.SecureStorageAvailability
import uniffi.paw_core.SecureStorageCapabilities
import uniffi.paw_core.SecureStorageProvider
import uniffi.paw_core.`createAccount`

private const val TOKEN_PREFS = "paw_secure_tokens"
private const val KEY_PREFS = "paw_device_keys"

data class StoredTokens(
    val accessToken: String,
    val refreshToken: String,
)

class AndroidSecureTokenVault(
    context: Context,
) {
    private val prefs = context.getSharedPreferences(TOKEN_PREFS, Context.MODE_PRIVATE)
    private val cipher = AndroidKeystoreBox("paw-android-token-vault")

    fun capabilities(): SecureStorageCapabilities = SecureStorageCapabilities(
        provider = SecureStorageProvider.KEYSTORE,
        availability = SecureStorageAvailability.AVAILABLE,
        supportsTokens = true,
        supportsDeviceKeys = true,
        supportsBiometricGate = Build.VERSION.SDK_INT >= Build.VERSION_CODES.P,
    )

    fun read(): StoredTokens? {
        val encryptedAccess = prefs.getString(KEY_ACCESS_TOKEN, null) ?: return null
        val encryptedRefresh = prefs.getString(KEY_REFRESH_TOKEN, null) ?: return null
        return runCatching {
            StoredTokens(
                accessToken = cipher.decryptString(encryptedAccess),
                refreshToken = cipher.decryptString(encryptedRefresh),
            )
        }.getOrNull()
    }

    fun write(tokens: StoredTokens) {
        prefs.edit()
            .putString(KEY_ACCESS_TOKEN, cipher.encryptString(tokens.accessToken))
            .putString(KEY_REFRESH_TOKEN, cipher.encryptString(tokens.refreshToken))
            .apply()
    }

    fun clear() {
        prefs.edit().clear().apply()
    }

    private companion object {
        const val KEY_ACCESS_TOKEN = "access_token"
        const val KEY_REFRESH_TOKEN = "refresh_token"
    }
}

class AndroidDeviceKeyStore(
    context: Context,
) {
    private val prefs: SharedPreferences = context.getSharedPreferences(KEY_PREFS, Context.MODE_PRIVATE)
    private val cipher = AndroidKeystoreBox("paw-android-device-key-store")

    fun loadOrCreate(): DeviceKeyMaterial {
        load()?.let { return it }
        val accountKeys = `createAccount`()
        val material = DeviceKeyMaterial(
            identityKey = accountKeys.identityKey,
            x25519PrivateKey = accountKeys.signedPrekeySecret,
            x25519PublicKey = accountKeys.signedPrekey,
        )
        write(material)
        return material
    }

    fun load(): DeviceKeyMaterial? {
        val identity = prefs.getString(KEY_IDENTITY, null) ?: return null
        val privateKey = prefs.getString(KEY_PRIVATE, null) ?: return null
        val publicKey = prefs.getString(KEY_PUBLIC, null) ?: return null
        return runCatching {
            DeviceKeyMaterial(
                identityKey = cipher.decryptBytes(identity),
                x25519PrivateKey = cipher.decryptBytes(privateKey),
                x25519PublicKey = cipher.decryptBytes(publicKey),
            )
        }.getOrNull()
    }

    fun clear() {
        prefs.edit().clear().apply()
    }

    private fun write(material: DeviceKeyMaterial) {
        prefs.edit()
            .putString(KEY_IDENTITY, cipher.encryptBytes(material.identityKey))
            .putString(KEY_PRIVATE, cipher.encryptBytes(material.x25519PrivateKey))
            .putString(KEY_PUBLIC, cipher.encryptBytes(material.x25519PublicKey))
            .apply()
    }

    private companion object {
        const val KEY_IDENTITY = "identity_key"
        const val KEY_PRIVATE = "x25519_private_key"
        const val KEY_PUBLIC = "x25519_public_key"
    }
}
