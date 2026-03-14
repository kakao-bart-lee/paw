package dev.paw.android.domain.model

import uniffi.paw_core.AuthStepView

/**
 * Pure Kotlin domain representation of auth-related UI state.
 * Mirrors the fields from PawBootstrapUiState that relate to the authentication flow.
 */
data class AuthUiState(
    val phoneInput: String = "",
    val otpInput: String = "",
    val deviceNameInput: String = "",
    val usernameInput: String = "",
    val discoverableByPhone: Boolean = false,
    val stagedSessionToken: String? = null,
)

/**
 * Determines the next auth step based on whether the user has a username set.
 */
fun nextStepAfterDeviceRegistration(username: String): AuthStepView =
    if (username.isBlank()) AuthStepView.USERNAME_SETUP else AuthStepView.AUTHENTICATED
