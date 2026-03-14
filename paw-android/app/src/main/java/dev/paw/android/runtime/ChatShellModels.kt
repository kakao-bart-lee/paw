package dev.paw.android.runtime

import uniffi.paw_core.ConversationCursorView
import uniffi.paw_core.RuntimeSnapshot

data class AndroidConversationItem(
    val id: String,
    val name: String,
    val lastMessage: String?,
    val unreadCount: Int,
)

data class AndroidChatMessage(
    val id: String,
    val conversationId: String,
    val senderId: String,
    val content: String,
    val format: String,
    val seq: Long,
    val createdAt: String,
    val isMe: Boolean,
    val isAgent: Boolean,
)

data class AndroidChatShellState(
    val conversations: List<AndroidConversationItem> = emptyList(),
    val selectedConversationId: String? = null,
    val messages: List<AndroidChatMessage> = emptyList(),
    val messageDraft: String = "",
    val conversationsLoading: Boolean = false,
    val messagesLoading: Boolean = false,
    val sendingMessage: Boolean = false,
    val conversationsError: String? = null,
    val messagesError: String? = null,
)

fun selectConversationId(
    current: String?,
    conversations: List<AndroidConversationItem>,
): String? = conversations.firstOrNull { it.id == current }?.id ?: conversations.firstOrNull()?.id

fun runtimeSnapshotWithChat(
    base: RuntimeSnapshot,
    selectedConversationId: String?,
    messages: List<AndroidChatMessage>,
): RuntimeSnapshot {
    val lastSeq = selectedConversationId?.let { conversationId ->
        messages
            .asSequence()
            .filter { it.conversationId == conversationId }
            .maxOfOrNull { it.seq }
    } ?: 0L

    return base.copy(
        cursors = selectedConversationId
            ?.let { listOf(ConversationCursorView(conversationId = it, lastSeq = lastSeq)) }
            ?: emptyList(),
    )
}
