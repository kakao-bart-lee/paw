package dev.paw.android

import uniffi.paw_core.AuthStateView
import uniffi.paw_core.ConnectionSnapshot
import uniffi.paw_core.ConnectionStateView
import uniffi.paw_core.DeviceKeyMaterial
import uniffi.paw_core.LifecycleHint
import uniffi.paw_core.LifecycleState
import uniffi.paw_core.PushRegistrationState
import uniffi.paw_core.RuntimeSnapshot
import uniffi.paw_core.SecureStorageCapabilities
import uniffi.paw_core.SecureStorageProvider
import uniffi.paw_core.`emptyPushRegistrationState`
import uniffi.paw_core.`emptyRuntimeSnapshot`
import uniffi.paw_core.ping

data class PawBootstrapPreview(
    val bridgeStatus: String,
    val auth: AuthStateView,
    val runtime: RuntimeSnapshot,
    val storage: SecureStorageCapabilities,
    val push: PushRegistrationState,
    val activeLifecycleHints: List<LifecycleHint>,
    val backgroundLifecycleHints: List<LifecycleHint>,
    val lastLifecycleState: LifecycleState,
    val bootstrapMessage: String,
    val deviceKeyReady: Boolean,
)

object PawCoreBridge {
    fun describePing(): String = try {
        "connected (${ping()})"
    } catch (error: UnsatisfiedLinkError) {
        "bindings compiled, native lib missing (${error.message})"
    } catch (error: Throwable) {
        "bridge call failed (${error::class.simpleName}: ${error.message})"
    }

    fun blankAuthState(): AuthStateView = AuthStateView(
        step = uniffi.paw_core.AuthStepView.AUTH_METHOD_SELECT,
        phone = "",
        deviceName = "",
        username = "",
        discoverableByPhone = false,
        hasSessionToken = false,
        hasAccessToken = false,
        hasRefreshToken = false,
        isLoading = false,
        error = null,
    )

    fun blankRuntimeSnapshot(): RuntimeSnapshot = `emptyRuntimeSnapshot`().copy(
        connection = ConnectionSnapshot(
            state = ConnectionStateView.DISCONNECTED,
            attempts = 0u,
            pendingReconnectDelayMs = null,
            pendingReconnectUri = null,
        ),
        cursors = emptyList(),
        activeStreams = emptyList(),
    )

    fun preview(
        auth: AuthStateView = blankAuthState(),
        runtime: RuntimeSnapshot = blankRuntimeSnapshot(),
        storage: SecureStorageCapabilities,
        push: PushRegistrationState = `emptyPushRegistrationState`(),
        activeLifecycleHints: List<LifecycleHint>,
        backgroundLifecycleHints: List<LifecycleHint>,
        lastLifecycleState: LifecycleState,
        bootstrapMessage: String,
        deviceKeyMaterial: DeviceKeyMaterial?,
    ): PawBootstrapPreview = PawBootstrapPreview(
        bridgeStatus = describePing(),
        auth = auth,
        runtime = runtime,
        storage = storage,
        push = push,
        activeLifecycleHints = activeLifecycleHints,
        backgroundLifecycleHints = backgroundLifecycleHints,
        lastLifecycleState = lastLifecycleState,
        bootstrapMessage = bootstrapMessage,
        deviceKeyReady = deviceKeyMaterial != null,
    )

    fun disconnectedStoragePreview(): SecureStorageCapabilities = SecureStorageCapabilities(
        provider = SecureStorageProvider.MEMORY_FALLBACK,
        availability = uniffi.paw_core.SecureStorageAvailability.DEGRADED,
        supportsTokens = true,
        supportsDeviceKeys = true,
        supportsBiometricGate = false,
    )
}
