package dev.paw.android.presentation.chat

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.FlowRow
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.testTag
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.unit.dp
import dev.paw.android.PawTestTags
import dev.paw.android.domain.model.ChatMessage
import dev.paw.android.presentation.bootstrap.BootstrapUiState
import dev.paw.android.presentation.bootstrap.BootstrapViewModel
import dev.paw.android.presentation.components.AuthField
import dev.paw.android.presentation.components.EditorialNote
import dev.paw.android.presentation.components.EditorialPanel
import dev.paw.android.presentation.components.EmptyStatePanel
import dev.paw.android.presentation.components.MetadataLine
import dev.paw.android.presentation.components.MoodCard
import dev.paw.android.presentation.components.PawPrimaryButton
import dev.paw.android.presentation.components.ShellStatChip
import dev.paw.android.presentation.theme.PawAccent
import dev.paw.android.presentation.theme.PawAgentBubble
import dev.paw.android.presentation.theme.PawMutedText
import dev.paw.android.presentation.theme.PawOutline
import dev.paw.android.presentation.theme.PawPrimary
import dev.paw.android.presentation.theme.PawPrimarySoft
import dev.paw.android.presentation.theme.PawReceivedBubble
import dev.paw.android.presentation.theme.PawSentBubble
import dev.paw.android.presentation.theme.PawStrongText
import dev.paw.android.presentation.theme.PawSurface1

@Composable
fun ChatShellCard(
    uiState: BootstrapUiState,
    viewModel: BootstrapViewModel,
    wideLayout: Boolean,
) {
    val chatVm = viewModel.chatViewModel
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
            if ((selectedConversation?.unreadCount ?: 0) > 0) {
                ShellStatChip("unread", selectedConversation?.unreadCount.toString())
            }
        }

        val listContent: @Composable () -> Unit = {
            if (uiState.chat.conversationsLoading) {
                EmptyStatePanel(
                    title = "대화 목록을 불러오는 중",
                    body = "최근 대화와 읽지 않은 메시지를 정리하고 있습니다.",
                    loading = true,
                )
            } else if (uiState.chat.conversationsError != null) {
                EmptyStatePanel(
                    title = "대화 목록을 가져오지 못했습니다",
                    body = uiState.chat.conversationsError,
                    actionLabel = "다시 시도",
                    onAction = chatVm::retryConversations,
                )
            } else if (uiState.chat.conversations.isEmpty()) {
                EmptyStatePanel(
                    title = "아직 시작된 대화가 없습니다",
                    body = "새 대화가 생기면 이 공간에서 스레드, 미리보기, 읽지 않은 수를 확인할 수 있습니다.",
                )
            } else {
                Column(modifier = Modifier.padding(top = 12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                    EditorialNote("최근 대화를 선택하면 오른쪽에서 메시지 흐름과 작성창이 이어집니다.")
                    uiState.chat.conversations.forEach { conversation ->
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

        val detailContent: @Composable () -> Unit = {
            selectedConversation?.let {
                MetadataLine("active thread", it.name)
            }

            when {
                uiState.chat.selectedConversationId == null && uiState.chat.conversations.isNotEmpty() -> {
                    EmptyStatePanel(
                        title = "대화를 선택하세요",
                        body = "왼쪽 목록에서 스레드를 고르면 메시지 기록과 작성창이 바로 열립니다.",
                    )
                }
                uiState.chat.selectedConversationId != null && uiState.chat.messagesLoading -> {
                    EmptyStatePanel(
                        title = "메시지를 불러오는 중",
                        body = "선택한 스레드의 최근 흐름을 준비하고 있습니다.",
                        loading = true,
                    )
                }
                uiState.chat.messagesError != null -> {
                    EmptyStatePanel(
                        title = "메시지를 가져오지 못했습니다",
                        body = uiState.chat.messagesError,
                        actionLabel = "다시 시도",
                        onAction = chatVm::retryMessages,
                    )
                }
                uiState.chat.selectedConversationId != null && uiState.chat.messages.isEmpty() -> {
                    EmptyStatePanel(
                        title = "아직 메시지가 없습니다",
                        body = "첫 메시지를 보내 이 스레드의 대화를 시작해 보세요.",
                    )
                    ComposerPanel(uiState, viewModel)
                }
                uiState.chat.selectedConversationId != null -> {
                    Column(modifier = Modifier.padding(top = 8.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
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
                        if (uiState.chat.sendingMessage) {
                            EditorialNote("메시지를 전송하고 있습니다.")
                        }
                        ComposerPanel(uiState, viewModel)
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
private fun ComposerPanel(
    uiState: BootstrapUiState,
    viewModel: BootstrapViewModel,
) {
    val chatVm = viewModel.chatViewModel
    Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
        AuthField(
            label = "메시지",
            value = uiState.chat.messageDraft,
            onValueChange = chatVm::onMessageDraftChanged,
            testTag = PawTestTags.CHAT_MESSAGE_INPUT,
        )
        PawPrimaryButton(
            onClick = chatVm::sendMessage,
            enabled = !uiState.chat.sendingMessage && uiState.chat.selectedConversationId != null,
            modifier = Modifier.testTag(PawTestTags.CHAT_SEND_MESSAGE),
        ) {
            Text(if (uiState.chat.sendingMessage) "전송 중..." else "메시지 보내기")
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
                        color = dev.paw.android.presentation.theme.PawSurface3,
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
private fun ChatBubble(message: ChatMessage) {
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
