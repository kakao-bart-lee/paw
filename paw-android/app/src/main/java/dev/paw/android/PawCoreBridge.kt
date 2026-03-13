package dev.paw.android

import uniffi.paw_core.AuthStateView
import uniffi.paw_core.ConnectionStateView
import uniffi.paw_core.LifecycleEvent
import uniffi.paw_core.LifecycleHint
import uniffi.paw_core.LifecycleState
import uniffi.paw_core.PushRegistrationState
import uniffi.paw_core.RuntimeSnapshot
import uniffi.paw_core.SecureStorageCapabilities
import uniffi.paw_core.`emptyPushRegistrationState`
import uniffi.paw_core.`emptyRuntimeSnapshot`
import uniffi.paw_core.`initialAuthStateView`
import uniffi.paw_core.`lifecycleHints`
import uniffi.paw_core.`memoryFallbackSecureStorageCapabilities`
import uniffi.paw_core.ping

data class PawBootstrapPreview(
    val bridgeStatus: String,
    val auth: AuthStateView,
    val runtime: RuntimeSnapshot,
    val storage: SecureStorageCapabilities,
    val push: PushRegistrationState,
    val activeLifecycleHints: List<LifecycleHint>,
    val backgroundLifecycleHints: List<LifecycleHint>,
)

object PawCoreBridge {
    fun describePing(): String = try {
        "connected (${ping()})"
    } catch (error: UnsatisfiedLinkError) {
        "bindings compiled, native lib missing (${error.message})"
    } catch (error: Throwable) {
        "bridge call failed (${error::class.simpleName}: ${error.message})"
    }

    fun loadBootstrapPreview(): PawBootstrapPreview = try {
        PawBootstrapPreview(
            bridgeStatus = describePing(),
            auth = `initialAuthStateView`(),
            runtime = `emptyRuntimeSnapshot`(),
            storage = `memoryFallbackSecureStorageCapabilities`(),
            push = `emptyPushRegistrationState`(),
            activeLifecycleHints = lifecycleHintsFor(LifecycleState.ACTIVE),
            backgroundLifecycleHints = lifecycleHintsFor(LifecycleState.BACKGROUND),
        )
    } catch (error: Throwable) {
        PawBootstrapPreview(
            bridgeStatus = "preview fallback (${error::class.simpleName}: ${error.message})",
            auth = AuthStateView(
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
            ),
            runtime = RuntimeSnapshot(
                connection = uniffi.paw_core.ConnectionSnapshot(
                    state = ConnectionStateView.DISCONNECTED,
                    attempts = 0u,
                    pendingReconnectDelayMs = null,
                    pendingReconnectUri = null,
                ),
                cursors = emptyList(),
                activeStreams = emptyList(),
            ),
            storage = SecureStorageCapabilities(
                provider = uniffi.paw_core.SecureStorageProvider.MEMORY_FALLBACK,
                availability = uniffi.paw_core.SecureStorageAvailability.DEGRADED,
                supportsTokens = true,
                supportsDeviceKeys = true,
                supportsBiometricGate = false,
            ),
            push = PushRegistrationState(
                status = uniffi.paw_core.PushRegistrationStatus.UNREGISTERED,
                token = null,
                platform = null,
                lastError = null,
                lastUpdatedMs = 0,
            ),
            activeLifecycleHints = listOf(LifecycleHint.RECONNECT_SOCKET, LifecycleHint.REFRESH_PUSH_TOKEN),
            backgroundLifecycleHints = listOf(
                LifecycleHint.PAUSE_REALTIME,
                LifecycleHint.FLUSH_ACKS,
                LifecycleHint.PERSIST_DRAFTS,
            ),
        )
    }

    private fun lifecycleHintsFor(state: LifecycleState): List<LifecycleHint> =
        `lifecycleHints`(
            LifecycleEvent(
                state = state,
                timestampMs = System.currentTimeMillis(),
                userInitiated = false,
            )
        )
}
