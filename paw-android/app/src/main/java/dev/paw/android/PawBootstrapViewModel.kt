package dev.paw.android

import androidx.lifecycle.ViewModel
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import uniffi.paw_core.AuthStepView

data class PawBootstrapUiState(
    val preview: PawBootstrapPreview = PawCoreBridge.loadBootstrapPreview(),
    val currentAuthStep: AuthStepView = PawCoreBridge.loadBootstrapPreview().auth.step,
)

class PawBootstrapViewModel : ViewModel() {
    var uiState by mutableStateOf(PawBootstrapUiState())
        private set

    fun refresh() {
        val preview = PawCoreBridge.loadBootstrapPreview()
        uiState = uiState.copy(
            preview = preview,
            currentAuthStep = preview.auth.step,
        )
    }

    fun previewPhoneOtpFlow() {
        uiState = uiState.copy(currentAuthStep = AuthStepView.PHONE_INPUT)
    }

    fun resetPreview() {
        uiState = uiState.copy(currentAuthStep = uiState.preview.auth.step)
    }
}
