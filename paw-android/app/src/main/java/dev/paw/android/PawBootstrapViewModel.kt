package dev.paw.android

import android.app.Application
import android.os.Build
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import dev.paw.android.runtime.AndroidDeviceKeyStore
import dev.paw.android.runtime.AndroidSecureTokenVault
import dev.paw.android.runtime.FirebasePushRegistrar
import dev.paw.android.runtime.PawAndroidConfig
import dev.paw.android.runtime.PawApiClient
import dev.paw.android.runtime.PawLifecycleBridge
import dev.paw.android.runtime.StoredTokens
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch
import java.util.Base64
import uniffi.paw_core.AuthStateView
import uniffi.paw_core.AuthStepView
import uniffi.paw_core.ConnectionSnapshot
import uniffi.paw_core.ConnectionStateView
import uniffi.paw_core.DeviceKeyMaterial
import uniffi.paw_core.LifecycleState
import uniffi.paw_core.PushRegistrationState
import uniffi.paw_core.RuntimeSnapshot

data class PawBootstrapUiState(
    val preview: PawBootstrapPreview,
    val phoneInput: String = "",
    val otpInput: String = "",
    val deviceNameInput: String = defaultDeviceName(),
    val usernameInput: String = "",
    val discoverableByPhone: Boolean = false,
    val stagedSessionToken: String? = null,
) {
    companion object {
        fun initial() = PawBootstrapUiState(
            preview = PawCoreBridge.preview(
                storage = PawCoreBridge.disconnectedStoragePreview(),
                activeLifecycleHints = emptyList(),
                backgroundLifecycleHints = emptyList(),
                lastLifecycleState = LifecycleState.LAUNCHING,
                bootstrapMessage = "Starting Android bootstrap…",
                deviceKeyMaterial = null,
            ),
        )

        private fun defaultDeviceName(): String = "Android-${Build.MODEL}"
    }
}

class PawBootstrapViewModel(application: Application) : AndroidViewModel(application) {
    private val apiClient = PawApiClient(PawAndroidConfig.apiBaseUrl)
    private val tokenVault = AndroidSecureTokenVault(application)
    private val deviceKeyStore = AndroidDeviceKeyStore(application)
    private val lifecycleBridge = PawLifecycleBridge()
    private val pushRegistrar = FirebasePushRegistrar(apiClient)

    var uiState by mutableStateOf(PawBootstrapUiState.initial())
        private set

    init {
        viewModelScope.launch {
            lifecycleBridge.state.collectLatest { state ->
                uiState = uiState.copy(
                    preview = uiState.preview.copy(lastLifecycleState = state),
                )
            }
        }

        viewModelScope.launch {
            bootstrap()
        }
    }

    fun lifecycleObserver() = lifecycleBridge

    fun onPhoneChanged(value: String) {
        uiState = uiState.copy(phoneInput = value)
    }

    fun onOtpChanged(value: String) {
        uiState = uiState.copy(otpInput = value)
    }

    fun onDeviceNameChanged(value: String) {
        uiState = uiState.copy(deviceNameInput = value)
    }

    fun onUsernameChanged(value: String) {
        uiState = uiState.copy(usernameInput = value)
    }

    fun onDiscoverableChanged(value: Boolean) {
        uiState = uiState.copy(discoverableByPhone = value)
    }

    fun showPhoneOtp() = updateAuthState { current ->
        current.copy(step = AuthStepView.PHONE_INPUT, error = null)
    }

    fun backToAuthMethodSelect() = updateAuthState {
        PawCoreBridge.blankAuthState()
    }

    fun refresh() {
        viewModelScope.launch {
            bootstrap(forceRefresh = true)
        }
    }

    fun requestOtp() {
        val phone = uiState.phoneInput.trim()
        if (phone.isBlank()) {
            setError("전화번호를 입력하세요.")
            return
        }

        executeAuthUpdate(stepOverride = AuthStepView.PHONE_INPUT) {
            apiClient.requestOtp(phone)
            currentAuth().copy(
                step = AuthStepView.OTP_VERIFY,
                phone = phone,
                isLoading = false,
                error = null,
            )
        }
    }

    fun verifyOtp() {
        val code = uiState.otpInput.trim()
        if (code.isBlank()) {
            setError("OTP 코드를 입력하세요.")
            return
        }

        executeAuthUpdate(stepOverride = AuthStepView.OTP_VERIFY) {
            val response = apiClient.verifyOtp(currentAuth().phone, code)
            val sessionToken = response.optString("session_token")
            require(sessionToken.isNotBlank()) { "Missing session token" }
            uiState = uiState.copy(stagedSessionToken = sessionToken)
            currentAuth().copy(
                step = AuthStepView.DEVICE_NAME,
                hasSessionToken = true,
                isLoading = false,
                error = null,
            )
        }
    }

    fun registerDevice() {
        val deviceName = uiState.deviceNameInput.trim()
        if (deviceName.isBlank()) {
            setError("디바이스 이름을 입력하세요.")
            return
        }

        executeAuthUpdate(stepOverride = AuthStepView.DEVICE_NAME) {
            val response = apiClient.registerDevice(
                sessionToken = requireSessionToken(),
                deviceName = deviceName,
                ed25519PublicKey = Base64.getEncoder().encodeToString(ensureDeviceKeyMaterial().identityKey),
            )
            val accessToken = response.optString("access_token")
            val refreshToken = response.optString("refresh_token")
            require(accessToken.isNotBlank() && refreshToken.isNotBlank()) { "Missing tokens from register-device response" }

            tokenVault.write(StoredTokens(accessToken, refreshToken))
            apiClient.setAccessToken(accessToken)
            val me = apiClient.getMe()
            refreshPush(accessToken)

            val username = me.optString("username")
            val discoverable = me.optBoolean("discoverable_by_phone", false)
            val nextStep = if (username.isBlank()) AuthStepView.USERNAME_SETUP else AuthStepView.AUTHENTICATED
            uiState = uiState.copy(
                usernameInput = username,
                discoverableByPhone = discoverable,
                stagedSessionToken = null,
            )
            updatePreview(
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
                runtime = connectionSnapshot(connected = true),
                push = pushRegistrar.currentState(),
                bootstrapMessage = "Device registered and bootstrap is live.",
                deviceKeyMaterial = ensureDeviceKeyMaterial(),
            )
            currentAuth()
        }
    }

    fun completeUsernameSetup() {
        val username = uiState.usernameInput.trim()
        if (username.isBlank()) {
            setError("username을 입력하세요.")
            return
        }

        executeAuthUpdate(stepOverride = AuthStepView.USERNAME_SETUP) {
            val updated = apiClient.updateMe(username, uiState.discoverableByPhone)
            val resolvedUsername = updated.optString("username").ifBlank { username }
            val discoverable = updated.optBoolean("discoverable_by_phone", uiState.discoverableByPhone)
            uiState = uiState.copy(usernameInput = resolvedUsername, discoverableByPhone = discoverable, stagedSessionToken = null)
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
    }

    fun logout() {
        viewModelScope.launch(Dispatchers.IO) {
            val accessToken = if (currentAuth().hasAccessToken) tokenVault.read()?.accessToken else null
            pushRegistrar.unregister(accessToken)
            tokenVault.clear()
            apiClient.setAccessToken(null)
            uiState = uiState.copy(
                usernameInput = "",
                otpInput = "",
                stagedSessionToken = null,
                preview = PawCoreBridge.preview(
                    auth = PawCoreBridge.blankAuthState(),
                    runtime = connectionSnapshot(connected = false),
                    storage = tokenVault.capabilities(),
                    push = pushRegistrar.currentState(),
                    activeLifecycleHints = lifecycleBridge.activeHints.value,
                    backgroundLifecycleHints = lifecycleBridge.backgroundHints.value,
                    lastLifecycleState = lifecycleBridge.state.value,
                    bootstrapMessage = "Session cleared from Android Keystore.",
                    deviceKeyMaterial = deviceKeyStore.load(),
                ),
            )
        }
    }

    private suspend fun bootstrap(forceRefresh: Boolean = false) {
        val deviceKeys = ensureDeviceKeyMaterial()
        val tokens = tokenVault.read()
        val bootstrapAuth = PawCoreBridge.blankAuthState()
        val storage = tokenVault.capabilities()
        var push = pushRegistrar.currentState()
        var runtime = connectionSnapshot(connected = false)
        var message = if (forceRefresh) "Refreshing bootstrap from Android runtime…" else "Android bootstrap ready."
        var auth = bootstrapAuth

        if (tokens == null) {
            auth = bootstrapAuth.copy(step = AuthStepView.AUTH_METHOD_SELECT)
            message = "No stored session token found in Android Keystore."
        } else {
            apiClient.setAccessToken(tokens.accessToken)
            auth = try {
                val me = apiClient.getMe()
                push = refreshPush(tokens.accessToken)
                runtime = connectionSnapshot(connected = true)
                val username = me.optString("username")
                val discoverable = me.optBoolean("discoverable_by_phone", false)
                uiState = uiState.copy(usernameInput = username, discoverableByPhone = discoverable, stagedSessionToken = null)
                message = "Restored stored token from Android Keystore and refreshed runtime snapshot."
                bootstrapAuth.copy(
                    step = if (username.isBlank()) AuthStepView.USERNAME_SETUP else AuthStepView.AUTHENTICATED,
                    username = username,
                    discoverableByPhone = discoverable,
                    hasAccessToken = true,
                    hasRefreshToken = true,
                )
            } catch (error: Throwable) {
                tokenVault.clear()
                apiClient.setAccessToken(null)
                message = "Stored token restore failed; cleared invalid session. ${error.message.orEmpty()}".trim()
                bootstrapAuth.copy(error = error.message)
            }
        }

        updatePreview(
            auth = auth,
            runtime = runtime,
            storage = storage,
            push = push,
            bootstrapMessage = message,
            deviceKeyMaterial = deviceKeys,
        )
    }

    private fun executeAuthUpdate(
        stepOverride: AuthStepView,
        block: suspend () -> AuthStateView,
    ) {
        viewModelScope.launch {
            val starting = currentAuth().copy(step = stepOverride, isLoading = true, error = null)
            updatePreview(
                auth = starting,
                runtime = uiState.preview.runtime,
                push = uiState.preview.push,
                bootstrapMessage = uiState.preview.bootstrapMessage,
                deviceKeyMaterial = deviceKeyStore.load(),
            )

            runCatching { block() }
                .onSuccess { auth ->
                    updatePreview(
                        auth = auth,
                        runtime = uiState.preview.runtime,
                        push = uiState.preview.push,
                        bootstrapMessage = uiState.preview.bootstrapMessage,
                        deviceKeyMaterial = deviceKeyStore.load(),
                    )
                }
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
        updatePreview(
            auth = nextAuth,
            runtime = uiState.preview.runtime,
            push = uiState.preview.push,
            bootstrapMessage = uiState.preview.bootstrapMessage,
            deviceKeyMaterial = deviceKeyStore.load(),
        )
    }

    private fun updatePreview(
        auth: AuthStateView,
        runtime: RuntimeSnapshot,
        storage: uniffi.paw_core.SecureStorageCapabilities = tokenVault.capabilities(),
        push: PushRegistrationState,
        bootstrapMessage: String,
        deviceKeyMaterial: DeviceKeyMaterial?,
    ) {
        uiState = uiState.copy(
            preview = PawCoreBridge.preview(
                auth = auth,
                runtime = runtime,
                storage = storage,
                push = push,
                activeLifecycleHints = lifecycleBridge.activeHints.value,
                backgroundLifecycleHints = lifecycleBridge.backgroundHints.value,
                lastLifecycleState = lifecycleBridge.state.value,
                bootstrapMessage = bootstrapMessage,
                deviceKeyMaterial = deviceKeyMaterial,
            ),
        )
    }

    private fun currentAuth(): AuthStateView = uiState.preview.auth

    private fun requireSessionToken(): String {
        if (!currentAuth().hasSessionToken) {
            throw IllegalStateException("Missing session token for device registration")
        }
        // server session token is not exposed by view contract yet, so keep temporary value in OTP input field state
        return uiState.stagedSessionToken ?: throw IllegalStateException("Missing staged session token")
    }

    private fun ensureDeviceKeyMaterial(): DeviceKeyMaterial = deviceKeyStore.loadOrCreate()

    private suspend fun refreshPush(accessToken: String?): PushRegistrationState = pushRegistrar.register(accessToken)

    private fun connectionSnapshot(connected: Boolean): RuntimeSnapshot = PawCoreBridge.blankRuntimeSnapshot().copy(
        connection = ConnectionSnapshot(
            state = if (connected) ConnectionStateView.CONNECTED else ConnectionStateView.DISCONNECTED,
            attempts = 0u,
            pendingReconnectDelayMs = null,
            pendingReconnectUri = if (connected) PawAndroidConfig.apiBaseUrl else null,
        ),
    )
}
