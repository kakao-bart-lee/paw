package dev.paw.android

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Button
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.FilterChip
import androidx.compose.material3.AssistChip
import androidx.compose.material3.AssistChipDefaults
import androidx.compose.material3.ButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.OutlinedButton
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
import androidx.compose.ui.text.input.KeyboardType
import androidx.compose.ui.text.input.PasswordVisualTransformation
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewmodel.compose.viewModel
import dev.paw.android.runtime.AndroidChatMessage
import dev.paw.android.ui.theme.PawAccent
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
import dev.paw.android.ui.theme.PawSurface2
import dev.paw.android.ui.theme.PawSurface3
import dev.paw.android.ui.theme.PawSurface4
import uniffi.paw_core.AuthStepView

@Composable
fun PawAndroidApp(viewModel: PawBootstrapViewModel = viewModel()) {
    PawAndroidTheme {
        Surface(modifier = Modifier.fillMaxSize(), color = MaterialTheme.colorScheme.background) {
            val lifecycleState by viewModel.lifecycleObserver().state.collectAsStateWithLifecycle()
            val uiState = viewModel.uiState

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
                        val showDiagnostics =
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
                                    AuthStepChip("처음부터", PawTestTags.AUTH_CHIP_RESET, false, viewModel::backToAuthMethodSelect)
                                }
                                AuthStepChip(
                                    label = "전화 입력",
                                    testTag = PawTestTags.AUTH_CHIP_PHONE,
                                    selected = uiState.preview.auth.step == AuthStepView.PHONE_INPUT,
                                    onClick = viewModel::showPhoneOtp,
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

                        if (showDiagnostics) {
                            if (wideLayout) {
                                Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                                    MoodCard(
                                        title = "Lifecycle",
                                        subtitle = "runtime hint contract",
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
                                        MetadataLine("token cached", (!uiState.preview.push.token.isNullOrBlank()).toString())
                                        uiState.preview.push.lastError?.let { MetadataLine("push error", it) }
                                    }
                                }
                            } else {
                                MoodCard(
                                    title = "Lifecycle",
                                    subtitle = "active/background runtime hints",
                                    background = PawSentBubble,
                                ) {
                                    MetadataLine("active", uiState.preview.activeLifecycleHints.joinToString())
                                    MetadataLine("background", uiState.preview.backgroundLifecycleHints.joinToString())
                                }
                                MoodCard(
                                    title = "Push / secure storage",
                                    subtitle = "FCM + Android Keystore",
                                    background = PawAgentBubble,
                                ) {
                                    MetadataLine("push", uiState.preview.push.status.name)
                                    MetadataLine("token cached", (!uiState.preview.push.token.isNullOrBlank()).toString())
                                    uiState.preview.push.lastError?.let { MetadataLine("push error", it) }
                                }
                            }
                        }

                        if (uiState.preview.auth.step == AuthStepView.AUTHENTICATED) {
                            ChatShellCard(uiState, viewModel, wideLayout)
                            PawSecondaryButton(onClick = viewModel::logout, modifier = Modifier.testTag(PawTestTags.LOGOUT_BUTTON)) {
                                Text("로그아웃")
                            }
                        }
                    }
                }
            }
        }
    }
}

@Composable
private fun ChatShellCard(
    uiState: PawBootstrapUiState,
    viewModel: PawBootstrapViewModel,
    wideLayout: Boolean,
) {
    MoodCard(
        title = "Conversations",
        subtitle = "authenticated shell backed by /conversations + /messages",
        background = PawSurface1,
    ) {
        val selectedConversation = uiState.chat.conversations.firstOrNull { it.id == uiState.chat.selectedConversationId }

        FlowRow(
            horizontalArrangement = Arrangement.spacedBy(8.dp),
            verticalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            ShellStatChip("threads", uiState.chat.conversations.size.toString())
            ShellStatChip("selected", selectedConversation?.name ?: "none")
            ShellStatChip("messages", uiState.chat.messages.size.toString())
        }

        val listContent: @Composable () -> Unit = {
            uiState.chat.conversationsError?.let { EditorialNote(it) }
            if (uiState.chat.conversationsLoading) {
                CircularProgressIndicator(modifier = Modifier.padding(top = 12.dp))
            } else if (uiState.chat.conversations.isEmpty()) {
                Text(
                    "아직 표시할 대화가 없습니다. 인증은 완료되었지만 대화 목록이 비어 있습니다.",
                    color = PawMutedText,
                    modifier = Modifier.padding(top = 12.dp),
                )
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
        }

        val detailContent: @Composable () -> Unit = {
            selectedConversation?.let {
                MetadataLine("active thread", it.name)
            } ?: EditorialNote("왼쪽에서 대화를 선택하면 메시지와 작성창이 열립니다.")

            uiState.chat.messagesError?.let { EditorialNote(it) }

            if (uiState.chat.selectedConversationId != null) {
                Column(modifier = Modifier.padding(top = 8.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
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
                    PawPrimaryButton(
                        onClick = viewModel::sendMessage,
                        enabled = !uiState.chat.sendingMessage,
                        modifier = Modifier.testTag(PawTestTags.CHAT_SEND_MESSAGE),
                    ) {
                        Text(if (uiState.chat.sendingMessage) "전송 중…" else "메시지 보내기")
                    }
                }
            }
        }

        if (wideLayout) {
            Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                EditorialPanel(
                    title = "Thread list",
                    subtitle = "recent rooms and unread counts",
                    modifier = Modifier.weight(0.95f),
                    content = listContent,
                )
                EditorialPanel(
                    title = "Chat shell",
                    subtitle = "message history and composer",
                    modifier = Modifier.weight(1.05f),
                    content = detailContent,
                )
            }
        } else {
            EditorialPanel(
                title = "Thread list",
                subtitle = "recent rooms and unread counts",
                content = listContent,
            )
            EditorialPanel(
                title = "Chat shell",
                subtitle = "message history and composer",
                modifier = Modifier.padding(top = 12.dp),
                content = detailContent,
            )
        }

        val cursorSummary = uiState.preview.runtime.cursors
            .joinToString { "${it.conversationId.take(8)}:${it.lastSeq}" }
            .ifBlank { "-" }
        MetadataLine("runtime cursors", cursorSummary)
    }
}

@Composable
private fun AuthStepPanel(uiState: PawBootstrapUiState, viewModel: PawBootstrapViewModel) {
    when (uiState.preview.auth.step) {
        AuthStepView.AUTH_METHOD_SELECT -> {
            AuthSectionIntro(
                title = "전화번호로 시작하기",
                description = "기존 사용자도 같은 OTP 흐름으로 바로 로그인할 수 있습니다. Android에서는 이 흐름을 먼저 안정화합니다.",
            )
            PawPrimaryButton(
                onClick = viewModel::showPhoneOtp,
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(top = 12.dp)
                    .testTag(PawTestTags.AUTH_CONTINUE_PHONE),
            ) {
                Text("전화번호로 계속")
            }
        }
        AuthStepView.PHONE_INPUT -> {
            AuthSectionIntro(
                title = "번호 확인",
                description = "국가 코드를 생략하면 한국 번호(+82)로 자동 보정합니다. 예: 01012341234",
            )
            AuthField(
                label = "전화번호",
                value = uiState.phoneInput,
                onValueChange = viewModel::onPhoneChanged,
                keyboardType = KeyboardType.Phone,
                testTag = PawTestTags.AUTH_PHONE_INPUT,
            )
            PawPrimaryButton(
                onClick = viewModel::requestOtp,
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(top = 12.dp)
                    .testTag(PawTestTags.AUTH_REQUEST_OTP),
            ) {
                Text("OTP 요청")
            }
        }
        AuthStepView.OTP_VERIFY -> {
            AuthSectionIntro(
                title = "인증번호 입력",
                description = "개발 서버에서는 고정 OTP 137900을 사용할 수 있습니다.",
            )
            EditorialPanel(
                title = "Developer shortcut",
                subtitle = "fixed OTP for local bootstrap only",
                modifier = Modifier.padding(top = 12.dp),
            ) {
                AssistChip(
                    onClick = viewModel::useDebugOtp,
                    label = { Text("개발용 OTP ${dev.paw.android.runtime.PawAndroidConfig.debugFixedOtp}") },
                    shape = RoundedCornerShape(6.dp),
                    colors = AssistChipDefaults.assistChipColors(
                        containerColor = PawSurface3,
                        labelColor = PawStrongText,
                    ),
                    border = AssistChipDefaults.assistChipBorder(
                        enabled = true,
                        borderColor = PawOutline,
                    ),
                )
            }
            AuthField(
                label = "OTP 코드",
                value = uiState.otpInput,
                onValueChange = viewModel::onOtpChanged,
                keyboardType = KeyboardType.NumberPassword,
                testTag = PawTestTags.AUTH_OTP_INPUT,
            )
            FlowRow(
                modifier = Modifier.padding(top = 12.dp),
                horizontalArrangement = Arrangement.spacedBy(12.dp),
                verticalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                PawPrimaryButton(
                    onClick = viewModel::verifyOtp,
                    modifier = Modifier.testTag(PawTestTags.AUTH_VERIFY_OTP),
                ) {
                    Text("OTP 확인")
                }
            }
        }
        AuthStepView.DEVICE_NAME -> {
            AuthSectionIntro(
                title = "디바이스 등록",
                description = "이 기기 이름으로 세션을 등록하고 다음 단계로 진행합니다.",
            )
            EditorialPanel(
                title = "Session restore",
                subtitle = "기기 키와 이름을 함께 저장해 다음 실행에서 바로 복구합니다.",
                modifier = Modifier.padding(top = 12.dp),
            ) {
                MetadataLine("device keys", if (uiState.preview.deviceKeyReady) "ready" else "missing")
                MetadataLine("staged phone", uiState.preview.auth.phone.ifBlank { "(pending)" })
            }
            AuthField("디바이스 이름", uiState.deviceNameInput, viewModel::onDeviceNameChanged, testTag = PawTestTags.AUTH_DEVICE_NAME_INPUT)
            PawPrimaryButton(
                onClick = viewModel::registerDevice,
                modifier = Modifier
                    .fillMaxWidth()
                    .padding(top = 12.dp)
                    .testTag(PawTestTags.AUTH_REGISTER_DEVICE),
            ) {
                Text("디바이스 등록")
            }
        }
        AuthStepView.USERNAME_SETUP -> {
            AuthSectionIntro(
                title = "프로필 마무리",
                description = "username을 설정하면 검색/프로필 링크에 사용됩니다. 지금은 건너뛰고 나중에 설정할 수도 있습니다.",
            )
            AuthField("username", uiState.usernameInput, viewModel::onUsernameChanged, testTag = PawTestTags.AUTH_USERNAME_INPUT)
            EditorialPanel(
                title = "Search visibility",
                subtitle = "전화번호 기반 검색 허용 여부를 여기서 정합니다.",
                modifier = Modifier.padding(top = 12.dp),
            ) {
                Row(horizontalArrangement = Arrangement.spacedBy(12.dp)) {
                    Text("전화번호 검색 허용", color = PawMutedText)
                    Switch(checked = uiState.discoverableByPhone, onCheckedChange = viewModel::onDiscoverableChanged)
                }
            }
            FlowRow(modifier = Modifier.padding(top = 12.dp), horizontalArrangement = Arrangement.spacedBy(12.dp), verticalArrangement = Arrangement.spacedBy(12.dp)) {
                PawPrimaryButton(onClick = viewModel::completeUsernameSetup, modifier = Modifier.testTag(PawTestTags.AUTH_COMPLETE_USERNAME)) {
                    Text("완료")
                }
                PawSecondaryButton(onClick = viewModel::skipUsernameSetup, modifier = Modifier.testTag(PawTestTags.AUTH_SKIP_USERNAME)) {
                    Text("건너뛰기")
                }
            }
        }
        AuthStepView.AUTHENTICATED -> {
            AuthSectionIntro(
                title = "로그인 완료",
                description = "이제 대화 목록과 채팅 런타임을 사용할 수 있습니다.",
            )
            MetadataLine("username", uiState.preview.auth.username.ifBlank { "(unset)" })
            MetadataLine("device", uiState.preview.auth.deviceName.ifBlank { uiState.deviceNameInput })
        }
    }
}

@Composable
private fun AuthProgressSummary(step: AuthStepView) {
    val label = when (step) {
        AuthStepView.AUTH_METHOD_SELECT -> "1. 로그인 방식 선택"
        AuthStepView.PHONE_INPUT -> "2. 전화번호 입력"
        AuthStepView.OTP_VERIFY -> "3. OTP 확인"
        AuthStepView.DEVICE_NAME -> "4. 디바이스 등록"
        AuthStepView.USERNAME_SETUP -> "5. username 설정"
        AuthStepView.AUTHENTICATED -> "완료 · 채팅 진입 가능"
    }

    Text(
        text = label,
        modifier = Modifier.testTag(PawTestTags.AUTH_STEP_VALUE),
        style = MaterialTheme.typography.titleMedium,
        color = PawStrongText,
    )
}

@Composable
private fun AuthSectionIntro(
    title: String,
    description: String,
) {
    Column(verticalArrangement = Arrangement.spacedBy(6.dp)) {
        Text(title, style = MaterialTheme.typography.titleLarge, color = PawStrongText)
        Text(description, style = MaterialTheme.typography.bodyMedium, color = PawMutedText)
    }
}

@Composable
private fun AuthField(
    label: String,
    value: String,
    onValueChange: (String) -> Unit,
    secret: Boolean = false,
    keyboardType: KeyboardType = KeyboardType.Text,
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
        keyboardOptions = androidx.compose.foundation.text.KeyboardOptions(keyboardType = keyboardType),
        visualTransformation = if (secret) PasswordVisualTransformation() else androidx.compose.ui.text.input.VisualTransformation.None,
        shape = RoundedCornerShape(6.dp),
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
            .border(1.dp, PawOutline, RoundedCornerShape(8.dp)),
        shape = RoundedCornerShape(8.dp),
        colors = CardDefaults.cardColors(containerColor = background),
    ) {
        Column(modifier = Modifier.padding(18.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
            Text(title, style = MaterialTheme.typography.titleMedium, color = PawStrongText)
            Text(subtitle, style = MaterialTheme.typography.bodySmall, color = PawMutedText)
            if (content != null) {
                Column(
                    modifier = Modifier.padding(top = 8.dp),
                    verticalArrangement = Arrangement.spacedBy(8.dp),
                ) {
                    content()
                }
            }
        }
    }
}

@Composable
private fun EditorialPanel(
    title: String,
    subtitle: String,
    modifier: Modifier = Modifier,
    content: @Composable () -> Unit,
) {
    Surface(
        modifier = modifier
            .fillMaxWidth()
            .border(1.dp, PawOutline, RoundedCornerShape(8.dp)),
        shape = RoundedCornerShape(8.dp),
        color = PawSurface2,
    ) {
        Column(modifier = Modifier.padding(14.dp), verticalArrangement = Arrangement.spacedBy(6.dp)) {
            Text(title, style = MaterialTheme.typography.titleMedium, color = PawStrongText)
            Text(subtitle, style = MaterialTheme.typography.bodySmall, color = PawMutedText)
            Column(modifier = Modifier.padding(top = 6.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                content()
            }
        }
    }
}

@Composable
private fun EditorialNote(text: String) {
    Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
        Box(
            modifier = Modifier
                .padding(top = 4.dp)
                .size(6.dp)
                .background(PawAccent, RoundedCornerShape(50)),
        )
        Text(text, style = MaterialTheme.typography.bodySmall, color = PawMutedText)
    }
}

@Composable
private fun ShellStatChip(label: String, value: String) {
    Surface(
        shape = RoundedCornerShape(6.dp),
        color = PawSurface3,
        border = BorderStroke(1.dp, PawOutline),
    ) {
        Row(
            modifier = Modifier.padding(horizontal = 10.dp, vertical = 6.dp),
            horizontalArrangement = Arrangement.spacedBy(6.dp),
        ) {
            Text(label, style = MaterialTheme.typography.labelSmall, color = PawPrimary)
            Text(value, style = MaterialTheme.typography.bodySmall, color = PawStrongText)
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
        shape = RoundedCornerShape(6.dp),
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
            .border(1.dp, if (selected) PawAccent else PawOutline, RoundedCornerShape(8.dp)),
        shape = RoundedCornerShape(8.dp),
        colors = CardDefaults.cardColors(containerColor = if (selected) PawPrimarySoft else PawReceivedBubble),
    ) {
        Column(modifier = Modifier.padding(14.dp), verticalArrangement = Arrangement.spacedBy(4.dp)) {
            Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                Text(name, color = PawStrongText, fontWeight = FontWeight.SemiBold, modifier = Modifier.weight(1f, fill = false))
                if (unreadCount > 0) {
                    Surface(
                        shape = RoundedCornerShape(999.dp),
                        color = PawSurface3,
                        border = BorderStroke(1.dp, PawOutline),
                    ) {
                        Text(
                            unreadCount.toString(),
                            modifier = Modifier.padding(horizontal = 8.dp, vertical = 2.dp),
                            color = PawPrimary,
                            style = MaterialTheme.typography.labelSmall,
                        )
                    }
                }
            }
            Text(preview, color = PawMutedText, style = MaterialTheme.typography.bodySmall)
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

@Composable
private fun PawPrimaryButton(
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    enabled: Boolean = true,
    content: @Composable () -> Unit,
) {
    Button(
        onClick = onClick,
        modifier = modifier,
        enabled = enabled,
        shape = RoundedCornerShape(6.dp),
        colors = ButtonDefaults.buttonColors(
            containerColor = PawAccent,
            contentColor = PawBackground,
            disabledContainerColor = PawSurface4,
            disabledContentColor = PawMutedText,
        ),
    ) {
        content()
    }
}

@Composable
private fun PawSecondaryButton(
    onClick: () -> Unit,
    modifier: Modifier = Modifier,
    enabled: Boolean = true,
    content: @Composable () -> Unit,
) {
    OutlinedButton(
        onClick = onClick,
        modifier = modifier,
        enabled = enabled,
        shape = RoundedCornerShape(6.dp),
        colors = ButtonDefaults.outlinedButtonColors(
            contentColor = PawStrongText,
            disabledContentColor = PawMutedText,
        ),
        border = androidx.compose.foundation.BorderStroke(1.dp, PawOutline),
    ) {
        content()
    }
}
