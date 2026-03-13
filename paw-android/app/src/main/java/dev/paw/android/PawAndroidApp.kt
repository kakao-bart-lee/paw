package dev.paw.android

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.unit.dp
import androidx.lifecycle.viewmodel.compose.viewModel
import dev.paw.android.ui.theme.PawAgentBubble
import dev.paw.android.ui.theme.PawAndroidTheme
import dev.paw.android.ui.theme.PawBackground
import dev.paw.android.ui.theme.PawMutedText
import dev.paw.android.ui.theme.PawOutline
import dev.paw.android.ui.theme.PawPrimary
import dev.paw.android.ui.theme.PawPrimarySoft
import dev.paw.android.ui.theme.PawReceivedBubble
import dev.paw.android.ui.theme.PawSentBubble
import dev.paw.android.ui.theme.PawStrongText
import dev.paw.android.ui.theme.PawSurface1
import uniffi.paw_core.AuthStepView

@Composable
fun PawAndroidApp(viewModel: PawBootstrapViewModel = viewModel()) {
    PawAndroidTheme {
        Surface(modifier = Modifier.fillMaxSize(), color = MaterialTheme.colorScheme.background) {
            val uiState = viewModel.uiState

            Scaffold(
                containerColor = PawBackground,
            ) { innerPadding ->
                Column(
                    modifier = Modifier
                        .fillMaxSize()
                        .background(
                            brush = Brush.verticalGradient(
                                colors = listOf(PawSurface1, PawBackground),
                            ),
                        )
                        .padding(innerPadding)
                        .padding(24.dp),
                    verticalArrangement = Arrangement.spacedBy(16.dp),
                ) {
                    Text(
                        text = "Paw Android",
                        style = MaterialTheme.typography.headlineMedium,
                        color = PawStrongText,
                    )
                    Text(
                        text = "Flutter 버전의 다크 메신저 무드와 AI-first 분위기를 Android shell로 옮기는 기준 화면입니다.",
                        style = MaterialTheme.typography.bodyLarge,
                        color = PawMutedText,
                    )

                    MoodCard(
                        title = "Bootstrap",
                        subtitle = "bridge · runtime snapshot · contract readiness",
                        background = PawReceivedBubble,
                    ) {
                        MetadataLine("bridge", uiState.preview.bridgeStatus)
                        MetadataLine("connection", uiState.preview.runtime.connection.state.name)
                        MetadataLine("storage", uiState.preview.storage.provider.name)
                        MetadataLine("push", uiState.preview.push.status.name)
                    }

                    MoodCard(
                        title = "Auth preview",
                        subtitle = "same step contract for Android / iOS",
                        background = PawPrimarySoft,
                    ) {
                        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                            AuthStepChip(
                                label = "초기",
                                selected = uiState.currentAuthStep == uiState.preview.auth.step,
                                onClick = viewModel::resetPreview,
                            )
                            AuthStepChip(
                                label = "전화 입력",
                                selected = uiState.currentAuthStep == AuthStepView.PHONE_INPUT,
                                onClick = viewModel::previewPhoneOtpFlow,
                            )
                        }
                        MetadataLine("current step", uiState.currentAuthStep.name)
                        MetadataLine("discoverable", uiState.preview.auth.discoverableByPhone.toString())
                        MetadataLine("has access token", uiState.preview.auth.hasAccessToken.toString())
                    }

                    Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                        MoodCard(
                            title = "Lifecycle",
                            subtitle = "active/background runtime hints",
                            background = PawSentBubble,
                            modifier = Modifier.weight(1f),
                        ) {
                            MetadataLine("active", uiState.preview.activeLifecycleHints.joinToString())
                            MetadataLine("background", uiState.preview.backgroundLifecycleHints.joinToString())
                        }
                        MoodCard(
                            title = "Next app work",
                            subtitle = "platform adapters + auth/bootstrap",
                            background = PawAgentBubble,
                            modifier = Modifier.weight(1f),
                        ) {
                            MetadataLine("1", "Keystore token vault")
                            MetadataLine("2", "FCM registrar")
                            MetadataLine("3", "real bootstrap wiring")
                        }
                    }

                    Text(
                        text = "paw-core status: ${uiState.preview.bridgeStatus}",
                        style = MaterialTheme.typography.bodyMedium,
                        color = PawStrongText,
                    )
                }
            }
        }
    }
}

@Composable
private fun MoodCard(
    title: String,
    subtitle: String,
    background: androidx.compose.ui.graphics.Color,
    modifier: Modifier = Modifier,
    content: @Composable (() -> Unit)? = null,
) {
    Card(
        modifier = modifier
            .fillMaxWidth()
            .border(1.dp, PawOutline, RoundedCornerShape(22.dp)),
        shape = RoundedCornerShape(22.dp),
        colors = CardDefaults.cardColors(containerColor = background),
    ) {
        Column(
            modifier = Modifier.padding(18.dp),
            verticalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            Text(title, style = MaterialTheme.typography.titleMedium, color = PawStrongText)
            Text(subtitle, style = MaterialTheme.typography.bodySmall, color = PawMutedText)
            if (content != null) {
                Box(modifier = Modifier.padding(top = 8.dp)) {
                    content()
                }
            }
        }
    }
}

@Composable
private fun MetadataLine(label: String, value: String) {
    Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
        Text(label, style = MaterialTheme.typography.labelSmall, color = PawPrimary)
        Text(value, style = MaterialTheme.typography.bodySmall, color = PawStrongText)
    }
}

@Composable
private fun AuthStepChip(
    label: String,
    selected: Boolean,
    onClick: () -> Unit,
) {
    FilterChip(
        selected = selected,
        onClick = onClick,
        label = {
            Text(label)
        },
    )
}
