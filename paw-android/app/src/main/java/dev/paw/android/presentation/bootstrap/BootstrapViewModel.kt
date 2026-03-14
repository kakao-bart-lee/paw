package dev.paw.android.presentation.bootstrap

import android.app.Application
import android.os.Build
import androidx.lifecycle.AndroidViewModel
import androidx.lifecycle.viewModelScope
import dev.paw.android.di.AppContainer
import dev.paw.android.domain.model.ChatShellState
import dev.paw.android.domain.model.runtimeSnapshotWithChat
import dev.paw.android.presentation.auth.AuthViewModel
import dev.paw.android.presentation.auth.AuthViewModelCallback
import dev.paw.android.presentation.chat.ChatViewModel
import dev.paw.android.presentation.chat.ChatViewModelCallback
import dev.paw.android.runtime.PawBootstrapPreview
import dev.paw.android.runtime.PawCoreBridge
import dev.paw.android.runtime.PawLifecycleBridge
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch
import uniffi.paw_core.AuthStepView
import uniffi.paw_core.ConnectionSnapshot
import uniffi.paw_core.ConnectionStateView
import uniffi.paw_core.LifecycleState
import uniffi.paw_core.RuntimeSnapshot

/**
 * Composed UI state matching the original PawBootstrapUiState shape.
 * Aggregates preview, chat state, and auth input state for the UI layer.
 */
data class BootstrapUiState(
    val preview: PawBootstrapPreview,
    val chat: ChatShellState = ChatShellState(),
    val phoneInput: String = "",
    val otpInput: String = "",
    val deviceNameInput: String = defaultDeviceName(),
    val usernameInput: String = "",
    val discoverableByPhone: Boolean = false,
    val stagedSessionToken: String? = null,
) {
    companion object {
        fun initial() = BootstrapUiState(
            preview = PawCoreBridge.preview(
                storage = PawCoreBridge.disconnectedStoragePreview(),
                activeLifecycleHints = emptyList(),
                backgroundLifecycleHints = emptyList(),
                lastLifecycleState = LifecycleState.LAUNCHING,
                bootstrapMessage = "Starting Android bootstrap...",
                deviceKeyMaterial = null,
            ),
        )

        fun defaultDeviceName(): String = "Android-${Build.MODEL}"
    }
}

class BootstrapViewModel(application: Application) : AndroidViewModel(application) {

    private val container = AppContainer(application)
    private val lifecycleBridge = PawLifecycleBridge()

    private val _uiState = MutableStateFlow(BootstrapUiState.initial())
    val uiState: StateFlow<BootstrapUiState> = _uiState.asStateFlow()

    private val authCallback = object : AuthViewModelCallback {
        override fun onPreviewUpdated(updater: (PawBootstrapPreview) -> PawBootstrapPreview) {
            val current = _uiState.value
            _uiState.value = current.copy(preview = updater(current.preview))
        }

        override fun onRequestChatLoad() {
            chatViewModel.loadChatShell()
        }

        override fun onRequestChatClear() {
            chatViewModel.clearChat()
        }

        override fun currentPreview(): PawBootstrapPreview = _uiState.value.preview
    }

    private val chatCallback = object : ChatViewModelCallback {
        override fun onChatStateChanged(chat: ChatShellState) {
            val current = _uiState.value
            val updatedRuntime = runtimeSnapshotWithChat(
                base = current.preview.runtime,
                selectedConversationId = chat.selectedConversationId,
                messages = chat.messages,
            )
            _uiState.value = current.copy(
                chat = chat,
                preview = PawCoreBridge.preview(
                    auth = current.preview.auth,
                    runtime = updatedRuntime,
                    storage = current.preview.storage,
                    push = current.preview.push,
                    activeLifecycleHints = current.preview.activeLifecycleHints,
                    backgroundLifecycleHints = current.preview.backgroundLifecycleHints,
                    lastLifecycleState = current.preview.lastLifecycleState,
                    bootstrapMessage = current.preview.bootstrapMessage,
                    deviceKeyMaterial = container.authRepository.loadDeviceKey(),
                ),
            )
        }

        override fun currentPreview(): PawBootstrapPreview = _uiState.value.preview
    }

    val authViewModel = AuthViewModel(container.authRepository, authCallback, viewModelScope)
    val chatViewModel = ChatViewModel(container.chatRepository, chatCallback, viewModelScope)

    init {
        authViewModel.setDefaultDeviceName(BootstrapUiState.defaultDeviceName())

        viewModelScope.launch {
            lifecycleBridge.state.collectLatest { state ->
                val current = _uiState.value
                _uiState.value = current.copy(
                    preview = current.preview.copy(lastLifecycleState = state),
                )
            }
        }

        viewModelScope.launch {
            bootstrap()
        }

        // Sync auth input state changes to composed UI state
        viewModelScope.launch {
            authViewModel.authUiState.collectLatest { authUi ->
                val current = _uiState.value
                _uiState.value = current.copy(
                    phoneInput = authUi.phoneInput,
                    otpInput = authUi.otpInput,
                    deviceNameInput = authUi.deviceNameInput,
                    usernameInput = authUi.usernameInput,
                    discoverableByPhone = authUi.discoverableByPhone,
                    stagedSessionToken = authUi.stagedSessionToken,
                )
            }
        }
    }

    fun lifecycleObserver() = lifecycleBridge

    fun refresh() {
        viewModelScope.launch {
            bootstrap(forceRefresh = true)
        }
    }

    private suspend fun bootstrap(forceRefresh: Boolean = false) {
        val authRepo = container.authRepository
        val deviceKeys = authRepo.ensureDeviceKey()
        val tokens = authRepo.readTokens()
        val bootstrapAuth = PawCoreBridge.blankAuthState()
        val storage = authRepo.storageCapabilities()
        var push = authRepo.currentPushState()
        var runtime = connectionSnapshot(connected = false)
        var message = if (forceRefresh) "Refreshing bootstrap from Android runtime..." else "Android bootstrap ready."
        var auth = bootstrapAuth

        if (tokens == null) {
            auth = bootstrapAuth.copy(step = AuthStepView.AUTH_METHOD_SELECT)
            message = "No stored session token found in Android Keystore."
        } else {
            authRepo.setAccessToken(tokens.accessToken)
            auth = try {
                val me = authRepo.getMe()
                push = authRepo.refreshPush(tokens.accessToken)
                runtime = connectionSnapshot(connected = true)
                val username = me.optString("username")
                val discoverable = me.optBoolean("discoverable_by_phone", false)
                authViewModel.restoreFromBootstrap(username, discoverable)
                message = "Restored stored token from Android Keystore and refreshed runtime snapshot."
                bootstrapAuth.copy(
                    step = if (username.isBlank()) AuthStepView.USERNAME_SETUP else AuthStepView.AUTHENTICATED,
                    username = username,
                    discoverableByPhone = discoverable,
                    hasAccessToken = true,
                    hasRefreshToken = true,
                )
            } catch (error: Throwable) {
                authRepo.clearTokens()
                authRepo.setAccessToken(null)
                message = "Stored token restore failed; cleared invalid session. ${error.message.orEmpty()}".trim()
                bootstrapAuth.copy(error = error.message)
            }
        }

        val current = _uiState.value
        _uiState.value = current.copy(
            preview = PawCoreBridge.preview(
                auth = auth,
                runtime = runtimeSnapshotWithChat(
                    base = runtime,
                    selectedConversationId = current.chat.selectedConversationId,
                    messages = current.chat.messages,
                ),
                storage = storage,
                push = push,
                activeLifecycleHints = lifecycleBridge.activeHints.value,
                backgroundLifecycleHints = lifecycleBridge.backgroundHints.value,
                lastLifecycleState = lifecycleBridge.state.value,
                bootstrapMessage = message,
                deviceKeyMaterial = deviceKeys,
            ),
        )

        if (auth.step == AuthStepView.AUTHENTICATED || auth.step == AuthStepView.USERNAME_SETUP) {
            chatViewModel.loadChatShell()
        } else {
            chatViewModel.clearChat()
        }
    }

    private fun connectionSnapshot(connected: Boolean): RuntimeSnapshot = PawCoreBridge.blankRuntimeSnapshot().copy(
        connection = ConnectionSnapshot(
            state = if (connected) ConnectionStateView.CONNECTED else ConnectionStateView.DISCONNECTED,
            attempts = 0u,
            pendingReconnectDelayMs = null,
            pendingReconnectEndpoint = if (connected) dev.paw.android.runtime.PawAndroidConfig.apiBaseUrl else null,
        ),
    )
}
