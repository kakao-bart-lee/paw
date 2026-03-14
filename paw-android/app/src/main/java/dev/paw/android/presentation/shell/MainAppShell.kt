package dev.paw.android.presentation.bootstrap

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.BoxWithConstraints
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBar
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import dev.paw.android.PawTestTags
import dev.paw.android.domain.model.ChatMessage
import dev.paw.android.presentation.components.AuthField
import dev.paw.android.presentation.components.EmptyStatePanel
import dev.paw.android.presentation.components.PawPrimaryButton
import dev.paw.android.presentation.components.PawSecondaryButton
import dev.paw.android.presentation.theme.PawAccent
import dev.paw.android.presentation.theme.PawAgentBubble
import dev.paw.android.presentation.theme.PawBackground
import dev.paw.android.presentation.theme.PawMutedText
import dev.paw.android.presentation.theme.PawOutline
import dev.paw.android.presentation.theme.PawPrimary
import dev.paw.android.presentation.theme.PawPrimarySoft
import dev.paw.android.presentation.theme.PawReceivedBubble
import dev.paw.android.presentation.theme.PawSentBubble
import dev.paw.android.presentation.theme.PawStrongText
import dev.paw.android.presentation.theme.PawSurface1
import dev.paw.android.presentation.theme.PawSurface2
import dev.paw.android.presentation.theme.PawSurface3

private data class NavTab(val label: String, val symbol: String)

private val navTabs = listOf(
    NavTab("채팅", "💬"),
    NavTab("Agent", "✦"),
    NavTab("설정", "⚙"),
)

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MainAppShell(uiState: BootstrapUiState, viewModel: BootstrapViewModel) {
    var selectedTab by rememberSaveable { mutableIntStateOf(0) }

    Scaffold(
        containerColor = PawBackground,
        topBar = {
            TopAppBar(
                title = {
                    Column {
                        Text(
                            navTabs[selectedTab].label,
                            style = MaterialTheme.typography.titleLarge,
                            color = PawStrongText,
                        )
                        if (selectedTab == 0) {
                            Text(
                                "AI와 팀 메시지를 한곳에서",
                                style = MaterialTheme.typography.bodySmall,
                                color = PawMutedText,
                            )
                        }
                    }
                },
                colors = TopAppBarDefaults.topAppBarColors(
                    containerColor = PawSurface1,
                    titleContentColor = PawStrongText,
                ),
            )
        },
        bottomBar = {
            Surface(
                modifier = Modifier.fillMaxWidth().padding(horizontal = 12.dp, vertical = 12.dp),
                shape = RoundedCornerShape(10.dp),
                color = PawSurface2,
                border = BorderStroke(1.dp, PawOutline),
                shadowElevation = 24.dp,
            ) {
                Row(
                    modifier = Modifier.fillMaxWidth().padding(horizontal = 8.dp, vertical = 6.dp),
                ) {
                    navTabs.forEachIndexed { index, tab ->
                        Surface(
                            modifier = Modifier
                                .weight(1f)
                                .clickable { selectedTab = index },
                            shape = RoundedCornerShape(8.dp),
                            color = if (selectedTab == index) PawPrimarySoft else PawSurface2,
                        ) {
                            Column(
                                modifier = Modifier.padding(vertical = 10.dp),
                                horizontalAlignment = Alignment.CenterHorizontally,
                            ) {
                                Text(
                                    tab.symbol,
                                    style = MaterialTheme.typography.titleMedium,
                                )
                                Spacer(Modifier.height(4.dp))
                                Text(
                                    tab.label,
                                    style = MaterialTheme.typography.labelSmall,
                                    fontWeight = FontWeight.Bold,
                                    color = if (selectedTab == index) PawAccent else PawMutedText,
                                )
                            }
                        }
                    }
                }
            }
        },
    ) { innerPadding ->
        Box(modifier = Modifier.padding(innerPadding).fillMaxSize()) {
            when (selectedTab) {
                0 -> ChatTab(uiState, viewModel)
                1 -> AgentTab()
                2 -> SettingsTab(uiState, viewModel)
            }
        }
    }
}

// ── Chat Tab ─────────────────────────────────────────────────────────

@Composable
private fun ChatTab(uiState: BootstrapUiState, viewModel: BootstrapViewModel) {
    BoxWithConstraints(modifier = Modifier.fillMaxSize()) {
        val wideLayout = maxWidth >= 840.dp

        if (wideLayout) {
            Row(
                modifier = Modifier.fillMaxSize().padding(16.dp),
                horizontalArrangement = Arrangement.spacedBy(16.dp),
            ) {
                ConversationListPanel(
                    uiState = uiState,
                    viewModel = viewModel,
                    modifier = Modifier.width(360.dp).fillMaxHeight(),
                )
                ChatDetailPanel(
                    uiState = uiState,
                    viewModel = viewModel,
                    modifier = Modifier.weight(1f).fillMaxHeight(),
                )
            }
        } else {
            Column(modifier = Modifier.fillMaxSize()) {
                ConversationListPanel(
                    uiState = uiState,
                    viewModel = viewModel,
                    modifier = Modifier.fillMaxWidth().weight(0.4f),
                )
                ChatDetailPanel(
                    uiState = uiState,
                    viewModel = viewModel,
                    modifier = Modifier.fillMaxWidth().weight(0.6f),
                )
            }
        }
    }
}

@Composable
private fun ConversationListPanel(
    uiState: BootstrapUiState,
    viewModel: BootstrapViewModel,
    modifier: Modifier = Modifier,
) {
    val chatVm = viewModel.chatViewModel

    Column(
        modifier = modifier
            .clip(RoundedCornerShape(10.dp))
            .background(PawSurface2)
            .border(1.dp, PawOutline, RoundedCornerShape(10.dp)),
    ) {
        // Search placeholder
        Surface(
            modifier = Modifier
                .fillMaxWidth()
                .padding(horizontal = 16.dp, vertical = 12.dp),
            shape = RoundedCornerShape(8.dp),
            color = PawSurface3,
            border = BorderStroke(1.dp, PawOutline),
        ) {
            Row(
                modifier = Modifier.padding(horizontal = 14.dp, vertical = 13.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text("🔍", style = MaterialTheme.typography.bodyMedium)
                Spacer(Modifier.width(12.dp))
                Text(
                    "메시지 검색",
                    color = PawMutedText,
                    style = MaterialTheme.typography.bodyMedium,
                    modifier = Modifier.weight(1f),
                )
                Text(
                    "${uiState.chat.conversations.size}개",
                    color = PawAccent,
                    style = MaterialTheme.typography.labelSmall,
                    fontWeight = FontWeight.Bold,
                )
            }
        }

        // Conversation list
        when {
            uiState.chat.conversationsLoading -> {
                Box(modifier = Modifier.fillMaxSize(), contentAlignment = Alignment.Center) {
                    EmptyStatePanel(
                        title = "대화 목록을 불러오는 중",
                        body = "최근 대화와 읽지 않은 메시지를 정리하고 있습니다.",
                        loading = true,
                    )
                }
            }
            uiState.chat.conversationsError != null -> {
                Box(
                    modifier = Modifier.fillMaxSize().padding(16.dp),
                    contentAlignment = Alignment.Center,
                ) {
                    EmptyStatePanel(
                        title = "대화 목록을 가져오지 못했습니다",
                        body = uiState.chat.conversationsError,
                        actionLabel = "다시 시도",
                        onAction = chatVm::retryConversations,
                    )
                }
            }
            uiState.chat.conversations.isEmpty() -> {
                Box(
                    modifier = Modifier.fillMaxSize().padding(16.dp),
                    contentAlignment = Alignment.Center,
                ) {
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        Surface(
                            shape = RoundedCornerShape(10.dp),
                            color = PawSurface3,
                            border = BorderStroke(1.dp, PawOutline),
                            modifier = Modifier.size(72.dp),
                        ) {
                            Box(contentAlignment = Alignment.Center) {
                                Text("💬", style = MaterialTheme.typography.headlineMedium)
                            }
                        }
                        Spacer(Modifier.height(18.dp))
                        Text(
                            "아직 대화가 없습니다",
                            style = MaterialTheme.typography.titleMedium,
                            color = PawStrongText,
                        )
                        Spacer(Modifier.height(8.dp))
                        Text(
                            "새 대화를 시작하거나\n검색으로 메시지를 찾아보세요.",
                            style = MaterialTheme.typography.bodySmall,
                            color = PawMutedText,
                        )
                    }
                }
            }
            else -> {
                LazyColumn(
                    modifier = Modifier.fillMaxSize(),
                    contentPadding = PaddingValues(horizontal = 12.dp, vertical = 8.dp),
                    verticalArrangement = Arrangement.spacedBy(6.dp),
                ) {
                    items(uiState.chat.conversations, key = { it.id }) { conversation ->
                        ConversationRow(
                            name = conversation.name,
                            selected = conversation.id == uiState.chat.selectedConversationId,
                            preview = conversation.lastMessage ?: "최근 메시지 없음",
                            unreadCount = conversation.unreadCount,
                            onClick = { chatVm.selectConversation(conversation.id) },
                        )
                    }
                }
            }
        }
    }
}

@Composable
private fun ConversationRow(
    name: String,
    selected: Boolean,
    preview: String,
    unreadCount: Int,
    onClick: () -> Unit,
) {
    Surface(
        modifier = Modifier.fillMaxWidth().clickable(onClick = onClick),
        shape = RoundedCornerShape(8.dp),
        color = if (selected) PawPrimarySoft else PawReceivedBubble,
        border = BorderStroke(1.dp, if (selected) PawAccent else PawOutline),
    ) {
        Column(
            modifier = Modifier.padding(14.dp),
            verticalArrangement = Arrangement.spacedBy(4.dp),
        ) {
            Row(
                horizontalArrangement = Arrangement.spacedBy(8.dp),
                verticalAlignment = Alignment.CenterVertically,
            ) {
                Text(
                    name,
                    color = PawStrongText,
                    fontWeight = FontWeight.SemiBold,
                    style = MaterialTheme.typography.bodyMedium,
                    modifier = Modifier.weight(1f, fill = false),
                    maxLines = 1,
                    overflow = TextOverflow.Ellipsis,
                )
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
            Text(
                preview,
                color = PawMutedText,
                style = MaterialTheme.typography.bodySmall,
                maxLines = 1,
                overflow = TextOverflow.Ellipsis,
            )
        }
    }
}

// ── Chat Detail ──────────────────────────────────────────────────────

@Composable
private fun ChatDetailPanel(
    uiState: BootstrapUiState,
    viewModel: BootstrapViewModel,
    modifier: Modifier = Modifier,
) {
    val chatVm = viewModel.chatViewModel
    val selectedConversation = uiState.chat.conversations.firstOrNull {
        it.id == uiState.chat.selectedConversationId
    }

    Column(
        modifier = modifier
            .clip(RoundedCornerShape(10.dp))
            .background(PawSurface1)
            .border(1.dp, PawOutline, RoundedCornerShape(10.dp)),
    ) {
        when {
            uiState.chat.selectedConversationId == null && uiState.chat.conversations.isNotEmpty() -> {
                Box(
                    modifier = Modifier.fillMaxSize().padding(32.dp),
                    contentAlignment = Alignment.Center,
                ) {
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        Surface(
                            shape = RoundedCornerShape(10.dp),
                            color = PawSurface2,
                            border = BorderStroke(1.dp, PawOutline),
                            modifier = Modifier.size(84.dp),
                        ) {
                            Box(contentAlignment = Alignment.Center) {
                                Text("💬", style = MaterialTheme.typography.headlineLarge)
                            }
                        }
                        Spacer(Modifier.height(20.dp))
                        Text(
                            "대화를 선택하세요",
                            style = MaterialTheme.typography.headlineSmall,
                            color = PawStrongText,
                        )
                        Spacer(Modifier.height(10.dp))
                        Text(
                            "${uiState.chat.conversations.size}개의 대화가 준비되어 있습니다.\n왼쪽 목록에서 하나를 선택하세요.",
                            style = MaterialTheme.typography.bodySmall,
                            color = PawMutedText,
                        )
                    }
                }
            }
            uiState.chat.selectedConversationId != null && uiState.chat.messagesLoading -> {
                Box(
                    modifier = Modifier.fillMaxSize().padding(16.dp),
                    contentAlignment = Alignment.Center,
                ) {
                    EmptyStatePanel(
                        title = "메시지를 불러오는 중",
                        body = "선택한 스레드의 최근 흐름을 준비하고 있습니다.",
                        loading = true,
                    )
                }
            }
            uiState.chat.messagesError != null -> {
                Box(
                    modifier = Modifier.fillMaxSize().padding(16.dp),
                    contentAlignment = Alignment.Center,
                ) {
                    EmptyStatePanel(
                        title = "메시지를 가져오지 못했습니다",
                        body = uiState.chat.messagesError,
                        actionLabel = "다시 시도",
                        onAction = chatVm::retryMessages,
                    )
                }
            }
            uiState.chat.selectedConversationId != null -> {
                // Thread header
                if (selectedConversation != null) {
                    Surface(
                        modifier = Modifier.fillMaxWidth(),
                        color = PawSurface2,
                    ) {
                        Text(
                            selectedConversation.name,
                            modifier = Modifier.padding(horizontal = 16.dp, vertical = 14.dp),
                            style = MaterialTheme.typography.titleMedium,
                            color = PawStrongText,
                            fontWeight = FontWeight.SemiBold,
                        )
                    }
                }

                // Messages
                if (uiState.chat.messages.isEmpty()) {
                    Box(
                        modifier = Modifier.weight(1f).fillMaxWidth().padding(16.dp),
                        contentAlignment = Alignment.Center,
                    ) {
                        EmptyStatePanel(
                            title = "아직 메시지가 없습니다",
                            body = "첫 메시지를 보내 이 스레드의 대화를 시작해 보세요.",
                        )
                    }
                } else {
                    LazyColumn(
                        modifier = Modifier.weight(1f).fillMaxWidth().padding(horizontal = 12.dp),
                        contentPadding = PaddingValues(vertical = 12.dp),
                        verticalArrangement = Arrangement.spacedBy(8.dp),
                    ) {
                        items(uiState.chat.messages, key = { it.id }) { message ->
                            ChatBubble(message)
                        }
                    }
                }

                if (uiState.chat.sendingMessage) {
                    Text(
                        "메시지를 전송하고 있습니다.",
                        modifier = Modifier.padding(horizontal = 16.dp, vertical = 4.dp),
                        style = MaterialTheme.typography.bodySmall,
                        color = PawMutedText,
                    )
                }

                // Composer
                ComposerBar(uiState, viewModel)
            }
            else -> {
                Box(
                    modifier = Modifier.fillMaxSize().padding(32.dp),
                    contentAlignment = Alignment.Center,
                ) {
                    Text("대화를 선택하세요", color = PawMutedText)
                }
            }
        }
    }
}

@Composable
private fun ChatBubble(message: ChatMessage) {
    val background = when {
        message.isMe -> PawSentBubble
        message.isAgent -> PawAgentBubble
        else -> PawReceivedBubble
    }
    val label = when {
        message.isMe -> "You"
        message.isAgent -> "Agent"
        else -> message.senderId.take(8)
    }
    Surface(
        modifier = Modifier.fillMaxWidth(),
        shape = RoundedCornerShape(8.dp),
        color = background,
        border = BorderStroke(1.dp, PawOutline),
    ) {
        Column(
            modifier = Modifier.padding(12.dp),
            verticalArrangement = Arrangement.spacedBy(4.dp),
        ) {
            Text(
                label,
                style = MaterialTheme.typography.labelSmall,
                color = PawAccent,
                fontWeight = FontWeight.Bold,
            )
            Text(
                message.content,
                style = MaterialTheme.typography.bodyMedium,
                color = PawStrongText,
            )
        }
    }
}

@Composable
private fun ComposerBar(uiState: BootstrapUiState, viewModel: BootstrapViewModel) {
    val chatVm = viewModel.chatViewModel
    Surface(modifier = Modifier.fillMaxWidth(), color = PawSurface2) {
        Row(
            modifier = Modifier.padding(horizontal = 12.dp, vertical = 10.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.spacedBy(8.dp),
        ) {
            Box(modifier = Modifier.weight(1f)) {
                AuthField(
                    label = "메시지",
                    value = uiState.chat.messageDraft,
                    onValueChange = chatVm::onMessageDraftChanged,
                    testTag = PawTestTags.CHAT_MESSAGE_INPUT,
                )
            }
            PawPrimaryButton(
                onClick = chatVm::sendMessage,
                enabled = !uiState.chat.sendingMessage && uiState.chat.selectedConversationId != null,
                modifier = Modifier.heightIn(min = 48.dp).testTag(PawTestTags.CHAT_SEND_MESSAGE),
            ) {
                Text(if (uiState.chat.sendingMessage) "전송 중..." else "보내기")
            }
        }
    }
}

// ── Agent Tab ────────────────────────────────────────────────────────

@Composable
private fun AgentTab() {
    Box(
        modifier = Modifier.fillMaxSize().padding(32.dp),
        contentAlignment = Alignment.Center,
    ) {
        Column(horizontalAlignment = Alignment.CenterHorizontally) {
            Surface(
                shape = RoundedCornerShape(10.dp),
                color = PawSurface2,
                border = BorderStroke(1.dp, PawOutline),
                modifier = Modifier.size(84.dp),
            ) {
                Box(contentAlignment = Alignment.Center) {
                    Text("✦", style = MaterialTheme.typography.headlineLarge, color = PawAccent)
                }
            }
            Spacer(Modifier.height(20.dp))
            Text("Agent", style = MaterialTheme.typography.headlineSmall, color = PawStrongText)
            Spacer(Modifier.height(10.dp))
            Text(
                "AI 에이전트를 탐색하고 관리하세요.",
                style = MaterialTheme.typography.bodySmall,
                color = PawMutedText,
            )
        }
    }
}

// ── Settings Tab ─────────────────────────────────────────────────────

@Composable
private fun SettingsTab(uiState: BootstrapUiState, viewModel: BootstrapViewModel) {
    Column(
        modifier = Modifier
            .fillMaxSize()
            .verticalScroll(rememberScrollState())
            .padding(24.dp),
        verticalArrangement = Arrangement.spacedBy(16.dp),
    ) {
        Surface(
            shape = RoundedCornerShape(10.dp),
            color = PawSurface2,
            border = BorderStroke(1.dp, PawOutline),
            modifier = Modifier.fillMaxWidth(),
        ) {
            Column(
                modifier = Modifier.padding(18.dp),
                verticalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                Text(
                    "프로필",
                    style = MaterialTheme.typography.titleMedium,
                    color = PawStrongText,
                    fontWeight = FontWeight.SemiBold,
                )
                SettingsRow("username", uiState.preview.auth.username.ifBlank { "guest" })
                SettingsRow("phone", uiState.preview.auth.phone.ifBlank { "-" })
                SettingsRow("device", uiState.preview.auth.deviceName.ifBlank { "-" })
                SettingsRow(
                    "discoverable",
                    if (uiState.preview.auth.discoverableByPhone) "Yes" else "No",
                )
            }
        }

        Surface(
            shape = RoundedCornerShape(10.dp),
            color = PawSurface2,
            border = BorderStroke(1.dp, PawOutline),
            modifier = Modifier.fillMaxWidth(),
        ) {
            Column(
                modifier = Modifier.padding(18.dp),
                verticalArrangement = Arrangement.spacedBy(12.dp),
            ) {
                Text(
                    "앱 정보",
                    style = MaterialTheme.typography.titleMedium,
                    color = PawStrongText,
                    fontWeight = FontWeight.SemiBold,
                )
                SettingsRow("version", "0.1.0")
                SettingsRow("platform", "Android")
            }
        }

        PawSecondaryButton(
            onClick = viewModel.authViewModel::logout,
            modifier = Modifier.fillMaxWidth(),
        ) {
            Text("로그아웃")
        }
    }
}

@Composable
private fun SettingsRow(label: String, value: String) {
    Row(
        modifier = Modifier.fillMaxWidth(),
        horizontalArrangement = Arrangement.SpaceBetween,
    ) {
        Text(label, style = MaterialTheme.typography.bodyMedium, color = PawMutedText)
        Text(value, style = MaterialTheme.typography.bodyMedium, color = PawStrongText)
    }
}
