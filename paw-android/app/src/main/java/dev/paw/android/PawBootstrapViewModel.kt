@file:Suppress("unused")
package dev.paw.android

import dev.paw.android.domain.model.ChatShellState
import dev.paw.android.runtime.PawBootstrapPreview

/**
 * Backward-compatibility alias for the composed UI state.
 * New code should use BootstrapUiState from presentation.bootstrap directly.
 */
data class PawBootstrapUiState(
    val preview: PawBootstrapPreview,
    val chat: ChatShellState = ChatShellState(),
    val phoneInput: String = "",
    val otpInput: String = "",
    val deviceNameInput: String = "",
    val usernameInput: String = "",
    val discoverableByPhone: Boolean = false,
    val stagedSessionToken: String? = null,
)

/**
 * Backward-compatibility alias.
 * New code should use BootstrapViewModel from presentation.bootstrap directly.
 */
typealias PawBootstrapViewModel = dev.paw.android.presentation.bootstrap.BootstrapViewModel
