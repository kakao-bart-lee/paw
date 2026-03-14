package dev.paw.android.presentation.bootstrap

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import dev.paw.android.PawTestTags
import dev.paw.android.presentation.auth.AuthProgressSummary
import dev.paw.android.presentation.auth.AuthStepChip
import dev.paw.android.presentation.auth.AuthStepPanel
import dev.paw.android.presentation.chat.ChatShellCard
import dev.paw.android.presentation.components.MetadataLine
import dev.paw.android.presentation.components.MoodCard
import dev.paw.android.presentation.components.PawSecondaryButton
import dev.paw.android.presentation.components.ShellStatChip
import dev.paw.android.presentation.components.EditorialNote
import dev.paw.android.presentation.theme.PawAgentBubble
import dev.paw.android.presentation.theme.PawAndroidTheme
import dev.paw.android.presentation.theme.PawBackground
import dev.paw.android.presentation.theme.PawMutedText
import dev.paw.android.presentation.theme.PawPrimarySoft
import dev.paw.android.presentation.theme.PawReceivedBubble
import dev.paw.android.presentation.theme.PawStrongText
import dev.paw.android.presentation.theme.PawSurface1
import uniffi.paw_core.AuthStepView

@Composable
fun BootstrapScreen(viewModel: BootstrapViewModel) {
    PawAndroidTheme {
        Surface(modifier = Modifier.fillMaxSize(), color = MaterialTheme.colorScheme.background) {
            val lifecycleState by viewModel.lifecycleObserver().state.collectAsStateWithLifecycle()
            val uiState by viewModel.uiState.collectAsStateWithLifecycle()

            Scaffold(containerColor = PawBackground) { innerPadding ->
                BoxWithConstraints(
                    modifier = Modifier
                        .testTag(PawTestTags.SCREEN_ROOT)
                        .fillMaxSize()
                        .background(brush = Brush.verticalGradient(colors = listOf(PawSurface1, PawBackground)))
                        .padding(innerPadding)
                        .padding(24.dp),
                ) {
                    val wideLayout = maxWidth >= 840.dp

                    Column(
                        modifier = Modifier
                            .fillMaxSize()
                            .verticalScroll(rememberScrollState()),
                        verticalArrangement = Arrangement.spacedBy(16.dp),
                    ) {
                        val showDiagnostics = uiState.preview.auth.step == AuthStepView.DEVICE_NAME
                        val showRuntimeStatus =
                            uiState.preview.auth.step == AuthStepView.AUTHENTICATED ||
                                uiState.preview.auth.step == AuthStepView.USERNAME_SETUP ||
                                uiState.preview.auth.step == AuthStepView.DEVICE_NAME

                        Text(
                            "Paw Android",
                            modifier = Modifier.testTag(PawTestTags.APP_TITLE),
                            style = MaterialTheme.typography.headlineMedium,
                            color = PawStrongText,
                        )
                        Text(
                            text = "네이티브 Android 앱 기준으로 로그인과 대화 흐름을 먼저 정리합니다.",
                            style = MaterialTheme.typography.bodyLarge,
                            color = PawMutedText,
                        )

                        if (showDiagnostics) {
                            MoodCard(
                                title = "Bootstrap",
                                subtitle = "stored token restore · lifecycle snapshot",
                                background = PawReceivedBubble,
                            ) {
                                MetadataLine("bridge", uiState.preview.bridgeStatus)
                                MetadataLine("connection", uiState.preview.runtime.connection.state.name)
                                MetadataLine("storage", uiState.preview.storage.provider.name)
                                MetadataLine("device keys", if (uiState.preview.deviceKeyReady) "ready" else "missing")
                                MetadataLine("lifecycle", lifecycleState.name)
                                uiState.preview.bootstrapMessage
                                    .takeIf { it.isNotBlank() && it != "ready" }
                                    ?.let { MetadataLine("status", it) }
                            }
                        } else {
                            val readyTitle = when (uiState.preview.auth.step) {
                                AuthStepView.AUTH_METHOD_SELECT -> "Ready for sign-in"
                                AuthStepView.PHONE_INPUT -> "Phone number"
                                AuthStepView.OTP_VERIFY -> "Verification code"
                                else -> "Ready for sign-in"
                            }
                            val readySubtitle = when (uiState.preview.auth.step) {
                                AuthStepView.AUTH_METHOD_SELECT -> "local Android shell + shared auth contract"
                                AuthStepView.PHONE_INPUT -> "Korean numbers are auto-normalized to +82"
                                AuthStepView.OTP_VERIFY -> "Use the fixed dev OTP or the code from server logs"
                                else -> "local Android shell + shared auth contract"
                            }
                            MoodCard(
                                title = readyTitle,
                                subtitle = readySubtitle,
                                background = PawReceivedBubble,
                            ) {
                                MetadataLine("storage", uiState.preview.storage.provider.name)
                                MetadataLine("device keys", if (uiState.preview.deviceKeyReady) "ready" else "missing")
                            }
                        }

                        val authCardTitle = when (uiState.preview.auth.step) {
                            AuthStepView.AUTH_METHOD_SELECT -> "Sign in"
                            AuthStepView.PHONE_INPUT -> "Phone verification"
                            AuthStepView.OTP_VERIFY -> "OTP verification"
                            AuthStepView.DEVICE_NAME -> "Device registration"
                            AuthStepView.USERNAME_SETUP -> "Finish profile"
                            AuthStepView.AUTHENTICATED -> "Authenticated"
                        }
                        val authCardSubtitle = when (uiState.preview.auth.step) {
                            AuthStepView.AUTH_METHOD_SELECT -> "start with the same OTP flow used in Flutter"
                            AuthStepView.PHONE_INPUT -> "enter the number that will receive the OTP"
                            AuthStepView.OTP_VERIFY -> "confirm the code, then unlock native bootstrap"
                            AuthStepView.DEVICE_NAME -> "register this device and enable session restore"
                            AuthStepView.USERNAME_SETUP -> "choose a username for profile/search"
                            AuthStepView.AUTHENTICATED -> "chat runtime and push wiring are now available"
                        }

                        MoodCard(
                            title = authCardTitle,
                            subtitle = authCardSubtitle,
                            background = PawPrimarySoft,
                        ) {
                            FlowRow(
                                horizontalArrangement = Arrangement.spacedBy(8.dp),
                                verticalArrangement = Arrangement.spacedBy(8.dp),
                            ) {
                                if (uiState.preview.auth.step != AuthStepView.AUTH_METHOD_SELECT) {
                                    AuthStepChip("처음부터", PawTestTags.AUTH_CHIP_RESET, false, viewModel.authViewModel::backToAuthMethodSelect)
                                }
                                AuthStepChip(
                                    label = "전화 입력",
                                    testTag = PawTestTags.AUTH_CHIP_PHONE,
                                    selected = uiState.preview.auth.step == AuthStepView.PHONE_INPUT,
                                    onClick = viewModel.authViewModel::showPhoneOtp,
                                )
                                if (uiState.preview.auth.step != AuthStepView.AUTH_METHOD_SELECT) {
                                    AuthStepChip("새로고침", PawTestTags.AUTH_CHIP_REFRESH, false, viewModel::refresh)
                                }
                            }
                            AuthProgressSummary(uiState.preview.auth.step)
                            uiState.preview.auth.error?.takeIf { it.isNotBlank() }?.let {
                                MetadataLine("error", it)
                            }
                            if (uiState.preview.auth.isLoading) {
                                Box(modifier = Modifier.padding(top = 12.dp)) {
                                    CircularProgressIndicator()
                                }
                            }
                            AuthStepPanel(uiState, viewModel)
                        }

                        if (showRuntimeStatus) {
                            MoodCard(
                                title = "Session status",
                                subtitle = "compact runtime health for post-auth flow",
                                background = PawAgentBubble,
                            ) {
                                FlowRow(
                                    horizontalArrangement = Arrangement.spacedBy(8.dp),
                                    verticalArrangement = Arrangement.spacedBy(8.dp),
                                ) {
                                    ShellStatChip("connection", uiState.preview.runtime.connection.state.name.lowercase())
                                    ShellStatChip("push", uiState.preview.push.status.name.lowercase())
                                    ShellStatChip("keys", if (uiState.preview.deviceKeyReady) "ready" else "missing")
                                    ShellStatChip("storage", uiState.preview.storage.provider.name.lowercase())
                                }
                                uiState.preview.push.lastError?.let { EditorialNote("Push issue: $it") }
                                if (showDiagnostics) {
                                    MetadataLine("active hints", uiState.preview.activeLifecycleHints.joinToString().ifBlank { "-" })
                                    MetadataLine("background hints", uiState.preview.backgroundLifecycleHints.joinToString().ifBlank { "-" })
                                }
                            }
                        }

                        if (uiState.preview.auth.step == AuthStepView.AUTHENTICATED) {
                            ChatShellCard(uiState, viewModel, wideLayout)
                            PawSecondaryButton(onClick = viewModel.authViewModel::logout, modifier = Modifier.testTag(PawTestTags.LOGOUT_BUTTON)) {
                                Text("로그아웃")
                            }
                        }
                    }
                }
            }
        }
    }
}
