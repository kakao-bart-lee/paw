package dev.paw.android.domain.model

import uniffi.paw_core.ConversationCursorView
import uniffi.paw_core.RuntimeSnapshot

/**
 * A single conversation summary for the conversation list.
 */
data class ConversationItem(
    val id: String,
    val name: String,
    val lastMessage: String?,
    val unreadCount: Int,
)

/**
 * A single chat message within a conversation thread.
 */
data class ChatMessage(
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

/**
 * Aggregate chat state for the chat shell UI.
 */
data class ChatShellState(
    val conversations: List<ConversationItem> = emptyList(),
    val selectedConversationId: String? = null,
    val messages: List<ChatMessage> = emptyList(),
    val messageDraft: String = "",
    val conversationsLoading: Boolean = false,
    val messagesLoading: Boolean = false,
    val sendingMessage: Boolean = false,
    val conversationsError: String? = null,
    val messagesError: String? = null,
)

/**
 * Selects the best conversation ID: keeps [current] if still in the list,
 * otherwise falls back to the first item, or null for an empty list.
 */
fun selectConversationId(
    current: String?,
    conversations: List<ConversationItem>,
): String? = conversations.firstOrNull { it.id == current }?.id ?: conversations.firstOrNull()?.id

/**
 * Builds a RuntimeSnapshot with a cursor reflecting the chat's max seq for the selected conversation.
 */
fun runtimeSnapshotWithChat(
    base: RuntimeSnapshot,
    selectedConversationId: String?,
    messages: List<ChatMessage>,
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
