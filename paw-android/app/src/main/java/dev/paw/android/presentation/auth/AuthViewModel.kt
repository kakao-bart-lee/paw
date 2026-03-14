package dev.paw.android.presentation.auth

import dev.paw.android.data.local.contracts.StoredTokens
import dev.paw.android.domain.model.AuthUiState
import dev.paw.android.domain.model.nextStepAfterDeviceRegistration
import dev.paw.android.domain.model.normalizePhone
import dev.paw.android.domain.repository.AuthRepository
import dev.paw.android.runtime.PawAndroidConfig
import dev.paw.android.runtime.PawBootstrapPreview
import dev.paw.android.runtime.PawCoreBridge
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import uniffi.paw_core.AuthStateView
import uniffi.paw_core.AuthStepView
import uniffi.paw_core.ConnectionSnapshot
import uniffi.paw_core.ConnectionStateView
import java.util.Base64

/**
 * Callback interface for AuthViewModel to notify the coordinator (BootstrapViewModel)
 * about preview and chat state changes.
 */
interface AuthViewModelCallback {
    fun onPreviewUpdated(updater: (PawBootstrapPreview) -> PawBootstrapPreview)
    fun onRequestChatLoad()
    fun onRequestChatClear()
    fun currentPreview(): PawBootstrapPreview
}

class AuthViewModel(
    private val authRepository: AuthRepository,
    private val callback: AuthViewModelCallback,
    private val scope: CoroutineScope,
) {

    private val _authUiState = MutableStateFlow(AuthUiState())
    val authUiState: StateFlow<AuthUiState> = _authUiState.asStateFlow()

    fun onPhoneChanged(value: String) {
        _authUiState.value = _authUiState.value.copy(phoneInput = value)
    }

    fun onOtpChanged(value: String) {
        _authUiState.value = _authUiState.value.copy(otpInput = value)
    }

    fun useDebugOtp() {
        _authUiState.value = _authUiState.value.copy(otpInput = PawAndroidConfig.debugFixedOtp)
    }

    fun onDeviceNameChanged(value: String) {
        _authUiState.value = _authUiState.value.copy(deviceNameInput = value)
    }

    fun onUsernameChanged(value: String) {
        _authUiState.value = _authUiState.value.copy(usernameInput = value)
    }

    fun onDiscoverableChanged(value: Boolean) {
        _authUiState.value = _authUiState.value.copy(discoverableByPhone = value)
    }

    fun setDefaultDeviceName(deviceName: String) {
        _authUiState.value = _authUiState.value.copy(deviceNameInput = deviceName)
    }

    fun restoreFromBootstrap(username: String, discoverable: Boolean) {
        _authUiState.value = _authUiState.value.copy(
            usernameInput = username,
            discoverableByPhone = discoverable,
            stagedSessionToken = null,
        )
    }

    fun showPhoneOtp() = updateAuthState { current ->
        current.copy(step = AuthStepView.PHONE_INPUT, error = null)
    }

    fun backToAuthMethodSelect() = updateAuthState {
        PawCoreBridge.blankAuthState()
    }

    fun requestOtp() {
        val phone = normalizePhone(_authUiState.value.phoneInput)
        if (phone.isBlank()) {
            setError("전화번호를 입력하세요.")
            return
        }

        executeAuthUpdate(stepOverride = AuthStepView.PHONE_INPUT) {
            authRepository.requestOtp(phone)
            currentAuth().copy(
                step = AuthStepView.OTP_VERIFY,
                phone = phone,
                isLoading = false,
                error = null,
            )
        }
    }

    fun verifyOtp() {
        val code = _authUiState.value.otpInput.trim()
        if (code.isBlank()) {
            setError("OTP 코드를 입력하세요.")
            return
        }

        executeAuthUpdate(stepOverride = AuthStepView.OTP_VERIFY) {
            val response = authRepository.verifyOtp(currentAuth().phone, code)
            val sessionToken = response.optString("session_token")
            require(sessionToken.isNotBlank()) { "Missing session token" }
            _authUiState.value = _authUiState.value.copy(stagedSessionToken = sessionToken)
            currentAuth().copy(
                step = AuthStepView.DEVICE_NAME,
                hasSessionToken = true,
                isLoading = false,
                error = null,
            )
        }
    }

    fun registerDevice() {
        val deviceName = _authUiState.value.deviceNameInput.trim()
        if (deviceName.isBlank()) {
            setError("디바이스 이름을 입력하세요.")
            return
        }

        executeAuthUpdate(stepOverride = AuthStepView.DEVICE_NAME) {
            val response = authRepository.registerDevice(
                sessionToken = requireSessionToken(),
                deviceName = deviceName,
                ed25519PublicKey = Base64.getEncoder().encodeToString(authRepository.ensureDeviceKey().identityKey),
            )
            val accessToken = response.optString("access_token")
            val refreshToken = response.optString("refresh_token")
            require(accessToken.isNotBlank() && refreshToken.isNotBlank()) { "Missing tokens from register-device response" }

            authRepository.writeTokens(StoredTokens(accessToken, refreshToken))
            authRepository.setAccessToken(accessToken)
            val me = authRepository.getMe()
            authRepository.refreshPush(accessToken)

            val username = me.optString("username")
            val discoverable = me.optBoolean("discoverable_by_phone", false)
            val nextStep = nextStepAfterDeviceRegistration(username)
            _authUiState.value = _authUiState.value.copy(
                usernameInput = username,
                discoverableByPhone = discoverable,
                stagedSessionToken = null,
            )
            callback.onPreviewUpdated { preview ->
                PawCoreBridge.preview(
                    auth = currentAuth().copy(
                        step = nextStep,
                        deviceName = deviceName,
                        username = username,
                        discoverableByPhone = discoverable,
                        hasSessionToken = false,
                        hasAccessToken = true,
                        hasRefreshToken = true,
                        isLoading = false,
                        error = null,
                    ),
                    runtime = preview.runtime,
                    storage = authRepository.storageCapabilities(),
                    push = authRepository.currentPushState(),
                    activeLifecycleHints = preview.activeLifecycleHints,
                    backgroundLifecycleHints = preview.backgroundLifecycleHints,
                    lastLifecycleState = preview.lastLifecycleState,
                    bootstrapMessage = "Device registered and bootstrap is live.",
                    deviceKeyMaterial = authRepository.ensureDeviceKey(),
                )
            }
            callback.onRequestChatLoad()
            currentAuth()
        }
    }

    fun completeUsernameSetup() {
        val username = _authUiState.value.usernameInput.trim()
        if (username.isBlank()) {
            setError("username을 입력하세요.")
            return
        }

        executeAuthUpdate(stepOverride = AuthStepView.USERNAME_SETUP) {
            val updated = authRepository.updateMe(username, _authUiState.value.discoverableByPhone)
            val resolvedUsername = updated.optString("username").ifBlank { username }
            val discoverable = updated.optBoolean("discoverable_by_phone", _authUiState.value.discoverableByPhone)
            _authUiState.value = _authUiState.value.copy(
                usernameInput = resolvedUsername,
                discoverableByPhone = discoverable,
                stagedSessionToken = null,
            )
            callback.onRequestChatLoad()
            currentAuth().copy(
                step = AuthStepView.AUTHENTICATED,
                username = resolvedUsername,
                discoverableByPhone = discoverable,
                isLoading = false,
                error = null,
            )
        }
    }

    fun skipUsernameSetup() {
        updateAuthState { current ->
            current.copy(step = AuthStepView.AUTHENTICATED, isLoading = false, error = null)
        }
        callback.onRequestChatLoad()
    }

    /**
     * Dev-only: bypass all auth steps with demo tokens and go straight to AUTHENTICATED.
     * Only available in debug builds.
     */
    fun devQuickLogin() {
        if (!dev.paw.android.BuildConfig.DEBUG) return
        scope.launch(Dispatchers.IO) {
            val demoTokens = StoredTokens(
                accessToken = "dev-access-token",
                refreshToken = "dev-refresh-token",
            )
            authRepository.writeTokens(demoTokens)
            authRepository.setAccessToken(demoTokens.accessToken)
            _authUiState.value = _authUiState.value.copy(
                usernameInput = "dev",
                discoverableByPhone = false,
                stagedSessionToken = null,
            )
            callback.onPreviewUpdated { preview ->
                PawCoreBridge.preview(
                    auth = currentAuth().copy(
                        step = AuthStepView.AUTHENTICATED,
                        phone = "+82 10-0000-0000",
                        deviceName = "Dev Emulator",
                        username = "dev",
                        discoverableByPhone = false,
                        hasSessionToken = false,
                        hasAccessToken = true,
                        hasRefreshToken = true,
                        isLoading = false,
                        error = null,
                    ),
                    runtime = preview.runtime,
                    storage = authRepository.storageCapabilities(),
                    push = preview.push,
                    activeLifecycleHints = preview.activeLifecycleHints,
                    backgroundLifecycleHints = preview.backgroundLifecycleHints,
                    lastLifecycleState = preview.lastLifecycleState,
                    bootstrapMessage = "Dev quick login — skipped auth flow.",
                    deviceKeyMaterial = authRepository.loadDeviceKey(),
                )
            }
            callback.onRequestChatLoad()
        }
    }

    fun logout() {
        scope.launch(Dispatchers.IO) {
            val accessToken = if (currentAuth().hasAccessToken) authRepository.readTokens()?.accessToken else null
            authRepository.unregisterPush(accessToken)
            authRepository.clearTokens()
            authRepository.setAccessToken(null)
            _authUiState.value = AuthUiState()
            callback.onRequestChatClear()
            callback.onPreviewUpdated { preview ->
                PawCoreBridge.preview(
                    auth = PawCoreBridge.blankAuthState(),
                    runtime = preview.runtime.copy(
                        connection = ConnectionSnapshot(
                            state = ConnectionStateView.DISCONNECTED,
                            attempts = 0u,
                            pendingReconnectDelayMs = null,
                            pendingReconnectEndpoint = null,
                        ),
                    ),
                    storage = authRepository.storageCapabilities(),
                    push = authRepository.currentPushState(),
                    activeLifecycleHints = preview.activeLifecycleHints,
                    backgroundLifecycleHints = preview.backgroundLifecycleHints,
                    lastLifecycleState = preview.lastLifecycleState,
                    bootstrapMessage = "Session cleared from Android Keystore.",
                    deviceKeyMaterial = authRepository.loadDeviceKey(),
                )
            }
        }
    }

    private fun executeAuthUpdate(
        stepOverride: AuthStepView,
        block: suspend () -> AuthStateView,
    ) {
        scope.launch {
            val starting = currentAuth().copy(step = stepOverride, isLoading = true, error = null)
            updatePreviewAuth(starting)

            runCatching { block() }
                .onSuccess { auth -> updatePreviewAuth(auth) }
                .onFailure { error ->
                    setError(error.message ?: error::class.simpleName ?: "Unknown auth error")
                }
        }
    }

    private fun setError(message: String) {
        updateAuthState { current ->
            current.copy(isLoading = false, error = message)
        }
    }

    private fun updateAuthState(transform: (AuthStateView) -> AuthStateView) {
        val nextAuth = transform(currentAuth())
        updatePreviewAuth(nextAuth)
    }

    private fun updatePreviewAuth(auth: AuthStateView) {
        callback.onPreviewUpdated { preview ->
            PawCoreBridge.preview(
                auth = auth,
                runtime = preview.runtime,
                storage = authRepository.storageCapabilities(),
                push = preview.push,
                activeLifecycleHints = preview.activeLifecycleHints,
                backgroundLifecycleHints = preview.backgroundLifecycleHints,
                lastLifecycleState = preview.lastLifecycleState,
                bootstrapMessage = preview.bootstrapMessage,
                deviceKeyMaterial = authRepository.loadDeviceKey(),
            )
        }
    }

    fun currentAuth(): AuthStateView = callback.currentPreview().auth

    private fun requireSessionToken(): String {
        if (!currentAuth().hasSessionToken) {
            throw IllegalStateException("Missing session token for device registration")
        }
        return _authUiState.value.stagedSessionToken
            ?: throw IllegalStateException("Missing staged session token")
    }
}
