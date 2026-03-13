package dev.paw.android

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.FilterChip
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedTextField
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Brush
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewmodel.compose.viewModel
import dev.paw.android.runtime.AndroidChatMessage
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
            val lifecycleState by viewModel.lifecycleObserver().state.collectAsStateWithLifecycle()
            val uiState = viewModel.uiState

            Scaffold(containerColor = PawBackground) { innerPadding ->
                Column(
                    modifier = Modifier
                        .testTag(PawTestTags.SCREEN_ROOT)
                        .fillMaxSize()
                        .background(brush = Brush.verticalGradient(colors = listOf(PawSurface1, PawBackground)))
                        .padding(innerPadding)
                        .padding(24.dp)
                        .verticalScroll(rememberScrollState()),
                    verticalArrangement = Arrangement.spacedBy(16.dp),
                ) {
                    Text(
                        "Paw Android",
                        modifier = Modifier.testTag(PawTestTags.APP_TITLE),
                        style = MaterialTheme.typography.headlineMedium,
                        color = PawStrongText,
                    )
                    Text(
                        text = "Keystore + FCM + real bootstrap/auth wiring 상태를 Android shell에서 바로 검증합니다.",
                        style = MaterialTheme.typography.bodyLarge,
                        color = PawMutedText,
                    )

                    MoodCard(
                        title = "Bootstrap",
                        subtitle = "stored token restore · lifecycle hints · runtime snapshot",
                        background = PawReceivedBubble,
                    ) {
                        MetadataLine("bridge", uiState.preview.bridgeStatus)
                        MetadataLine("server", dev.paw.android.runtime.PawAndroidConfig.apiBaseUrl)
                        MetadataLine("connection", uiState.preview.runtime.connection.state.name)
                        MetadataLine("storage", uiState.preview.storage.provider.name)
                        MetadataLine("device keys", if (uiState.preview.deviceKeyReady) "ready" else "missing")
                        MetadataLine("bootstrap", uiState.preview.bootstrapMessage)
                        MetadataLine("lifecycle", lifecycleState.name)
                    }

                    MoodCard(
                        title = "Auth state",
                        subtitle = "real step transitions wired from Android view model",
                        background = PawPrimarySoft,
                    ) {
                        Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                            AuthStepChip("초기", PawTestTags.AUTH_CHIP_RESET, uiState.preview.auth.step == AuthStepView.AUTH_METHOD_SELECT, viewModel::backToAuthMethodSelect)
                            AuthStepChip("전화 입력", PawTestTags.AUTH_CHIP_PHONE, uiState.preview.auth.step == AuthStepView.PHONE_INPUT, viewModel::showPhoneOtp)
                            AuthStepChip("새로고침", PawTestTags.AUTH_CHIP_REFRESH, false, viewModel::refresh)
                        }
                        MetadataLine("current step", uiState.preview.auth.step.name, PawTestTags.AUTH_STEP_VALUE)
                        MetadataLine("discoverable", uiState.preview.auth.discoverableByPhone.toString())
                        MetadataLine("has access token", uiState.preview.auth.hasAccessToken.toString())
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
                            title = "Push / secure storage",
                            subtitle = "FCM + Android Keystore",
                            background = PawAgentBubble,
                            modifier = Modifier.weight(1f),
                        ) {
                            MetadataLine("push", uiState.preview.push.status.name)
                            MetadataLine("push error", uiState.preview.push.lastError ?: "-")
                            MetadataLine("token cached", (!uiState.preview.push.token.isNullOrBlank()).toString())
                        }
                    }

                    if (uiState.preview.auth.step == AuthStepView.AUTHENTICATED) {
                        ChatShellCard(uiState, viewModel)
                        Button(onClick = viewModel::logout, modifier = Modifier.testTag(PawTestTags.LOGOUT_BUTTON)) {
                            Text("로그아웃")
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun ChatShellCard(uiState: PawBootstrapUiState, viewModel: PawBootstrapViewModel) {
    MoodCard(
        title = "Conversations / chat runtime",
        subtitle = "real authenticated shell backed by /conversations + /messages",
        background = PawSurface1,
    ) {
        MetadataLine("conversation count", uiState.chat.conversations.size.toString())
        MetadataLine("selected", uiState.chat.selectedConversationId ?: "-")
        MetadataLine("runtime cursors", uiState.preview.runtime.cursors.joinToString { "${it.conversationId.take(8)}:${it.lastSeq}" }.ifBlank { "-" })

        uiState.chat.conversationsError?.let { MetadataLine("conversation error", it) }
        uiState.chat.messagesError?.let { MetadataLine("message error", it) }

        if (uiState.chat.conversationsLoading) {
            CircularProgressIndicator(modifier = Modifier.padding(top = 12.dp))
        } else if (uiState.chat.conversations.isEmpty()) {
            Text("아직 표시할 대화가 없습니다. 인증 상태는 live 이지만 대화 목록은 비어 있습니다.", color = PawMutedText, modifier = Modifier.padding(top = 12.dp))
        } else {
            Column(modifier = Modifier.padding(top = 12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                uiState.chat.conversations.forEach { conversation ->
                    ConversationRow(
                        name = conversation.name,
                        selected = conversation.id == uiState.chat.selectedConversationId,
                        preview = conversation.lastMessage ?: "최근 메시지 없음",
                        unreadCount = conversation.unreadCount,
                        onClick = { viewModel.selectConversation(conversation.id) },
                    )
                }
            }
        }

        if (uiState.chat.selectedConversationId != null) {
            Column(modifier = Modifier.padding(top = 16.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                if (uiState.chat.messagesLoading) {
                    CircularProgressIndicator()
                } else if (uiState.chat.messages.isEmpty()) {
                    Text("메시지가 없습니다.", color = PawMutedText)
                } else {
                    Column(
                        modifier = Modifier
                            .fillMaxWidth()
                            .heightIn(max = 360.dp)
                            .verticalScroll(rememberScrollState()),
                        verticalArrangement = Arrangement.spacedBy(8.dp),
                    ) {
                        uiState.chat.messages.forEach { message ->
                            ChatBubble(message)
                        }
                    }
                }

                AuthField("메시지", uiState.chat.messageDraft, viewModel::onMessageDraftChanged, testTag = PawTestTags.CHAT_MESSAGE_INPUT)
                Button(
                    onClick = viewModel::sendMessage,
                    enabled = !uiState.chat.sendingMessage,
                    modifier = Modifier.testTag(PawTestTags.CHAT_SEND_MESSAGE),
                ) {
                    Text(if (uiState.chat.sendingMessage) "전송 중…" else "메시지 보내기")
                }
            }
        }
    }
}

@Composable
private fun AuthStepPanel(uiState: PawBootstrapUiState, viewModel: PawBootstrapViewModel) {
    when (uiState.preview.auth.step) {
        AuthStepView.AUTH_METHOD_SELECT -> {
            Button(
                onClick = viewModel::showPhoneOtp,
                modifier = Modifier
                    .padding(top = 12.dp)
                    .testTag(PawTestTags.AUTH_CONTINUE_PHONE),
            ) {
                Text("전화번호로 계속")
            }
        }
        AuthStepView.PHONE_INPUT -> {
            AuthField("전화번호", uiState.phoneInput, viewModel::onPhoneChanged, testTag = PawTestTags.AUTH_PHONE_INPUT)
            Button(
                onClick = viewModel::requestOtp,
                modifier = Modifier
                    .padding(top = 12.dp)
                    .testTag(PawTestTags.AUTH_REQUEST_OTP),
            ) {
                Text("OTP 요청")
            }
        }
        AuthStepView.OTP_VERIFY -> {
            AuthField("OTP 코드", uiState.otpInput, viewModel::onOtpChanged, secret = true, testTag = PawTestTags.AUTH_OTP_INPUT)
            Button(
                onClick = viewModel::verifyOtp,
                modifier = Modifier
                    .padding(top = 12.dp)
                    .testTag(PawTestTags.AUTH_VERIFY_OTP),
            ) {
                Text("OTP 확인")
            }
        }
        AuthStepView.DEVICE_NAME -> {
            AuthField("디바이스 이름", uiState.deviceNameInput, viewModel::onDeviceNameChanged, testTag = PawTestTags.AUTH_DEVICE_NAME_INPUT)
            Button(
                onClick = viewModel::registerDevice,
                modifier = Modifier
                    .padding(top = 12.dp)
                    .testTag(PawTestTags.AUTH_REGISTER_DEVICE),
            ) {
                Text("디바이스 등록")
            }
        }
        AuthStepView.USERNAME_SETUP -> {
            AuthField("username", uiState.usernameInput, viewModel::onUsernameChanged, testTag = PawTestTags.AUTH_USERNAME_INPUT)
            Row(modifier = Modifier.padding(top = 12.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                Text("전화번호 검색 허용", color = PawMutedText)
                Switch(checked = uiState.discoverableByPhone, onCheckedChange = viewModel::onDiscoverableChanged)
            }
            Row(modifier = Modifier.padding(top = 12.dp), horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                Button(onClick = viewModel::completeUsernameSetup, modifier = Modifier.testTag(PawTestTags.AUTH_COMPLETE_USERNAME)) {
                    Text("완료")
                }
                Button(onClick = viewModel::skipUsernameSetup, modifier = Modifier.testTag(PawTestTags.AUTH_SKIP_USERNAME)) {
                    Text("건너뛰기")
                }
            }
        }
        AuthStepView.AUTHENTICATED -> {
            MetadataLine("username", uiState.preview.auth.username.ifBlank { "(unset)" })
            MetadataLine("device", uiState.preview.auth.deviceName.ifBlank { uiState.deviceNameInput })
        }
    }
}

@Composable
private fun AuthField(
    label: String,
    value: String,
    onValueChange: (String) -> Unit,
    secret: Boolean = false,
    testTag: String? = null,
) {
    OutlinedTextField(
        modifier = Modifier
            .fillMaxWidth()
            .padding(top = 12.dp)
            .then(if (testTag != null) Modifier.testTag(testTag) else Modifier),
        value = value,
        onValueChange = onValueChange,
        label = { Text(label) },
        visualTransformation = if (secret) PasswordVisualTransformation() else androidx.compose.ui.text.input.VisualTransformation.None,
    )
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
        Column(modifier = Modifier.padding(18.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
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
private fun MetadataLine(label: String, value: String, valueTestTag: String? = null) {
    Column(verticalArrangement = Arrangement.spacedBy(2.dp)) {
        Text(label, style = MaterialTheme.typography.labelSmall, color = PawPrimary)
        Text(
            value,
            modifier = if (valueTestTag != null) Modifier.testTag(valueTestTag) else Modifier,
            style = MaterialTheme.typography.bodySmall,
            color = PawStrongText,
        )
    }
}

@Composable
private fun AuthStepChip(label: String, testTag: String, selected: Boolean, onClick: () -> Unit) {
    FilterChip(
        selected = selected,
        onClick = onClick,
        modifier = Modifier.testTag(testTag),
        label = { Text(label) },
    )
}

@Composable
private fun ConversationRow(
    name: String,
    selected: Boolean,
    preview: String,
    unreadCount: Int,
    onClick: () -> Unit,
) {
    Card(
        modifier = Modifier
            .fillMaxWidth()
            .clickable(onClick = onClick)
            .border(1.dp, if (selected) PawPrimary else PawOutline, RoundedCornerShape(18.dp)),
        shape = RoundedCornerShape(18.dp),
        colors = CardDefaults.cardColors(containerColor = if (selected) PawPrimarySoft else PawReceivedBubble),
    ) {
        Column(modifier = Modifier.padding(14.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
            Text(name, color = PawStrongText, fontWeight = FontWeight.SemiBold)
            Text(preview, color = PawMutedText, style = MaterialTheme.typography.bodySmall)
            Text("unread $unreadCount", color = PawPrimary, style = MaterialTheme.typography.labelSmall)
        }
    }
}

@Composable
private fun ChatBubble(message: AndroidChatMessage) {
    val background = when {
        message.isMe -> PawSentBubble
        message.isAgent -> PawAgentBubble
        else -> PawReceivedBubble
    }
    MoodCard(
        title = if (message.isMe) "You" else if (message.isAgent) "Agent" else message.senderId.take(8),
        subtitle = "seq ${message.seq}",
        background = background,
    ) {
        Text(message.content, color = PawStrongText)
    }
}
