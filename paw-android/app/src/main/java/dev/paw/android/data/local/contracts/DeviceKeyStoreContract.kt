package dev.paw.android.data.local.contracts

import uniffi.paw_core.DeviceKeyMaterial

/**
 * Platform-independent contract for device key storage.
 * Implementations handle key generation and secure persistence.
 */
interface DeviceKeyStoreContract {
    fun load(): DeviceKeyMaterial?
    fun loadOrCreate(): DeviceKeyMaterial
    fun hasKey(): Boolean
}
