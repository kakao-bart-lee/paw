package dev.paw.android.runtime

import uniffi.paw_core.AuthStateView
import uniffi.paw_core.LifecycleHint
import uniffi.paw_core.LifecycleState
import uniffi.paw_core.PushRegistrationState
import uniffi.paw_core.RuntimeSnapshot
import uniffi.paw_core.SecureStorageCapabilities

/**
 * Snapshot of all runtime state for preview/diagnostics display.
 * Used by the bootstrap coordinator to expose composed state to the UI.
 */
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
