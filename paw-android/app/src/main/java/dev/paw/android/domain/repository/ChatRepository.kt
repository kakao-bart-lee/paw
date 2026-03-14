package dev.paw.android.domain.repository

import dev.paw.android.domain.model.ChatMessage
import dev.paw.android.domain.model.ConversationItem

/**
 * Result of a successfully sent message as confirmed by the server.
 */
data class SendMessageResult(
    val id: String,
    val seq: Long,
    val createdAt: String,
)

/**
 * Repository interface for chat-related operations.
 * Abstracts API calls for conversations and messages.
 */
interface ChatRepository {
    suspend fun getConversations(): List<ConversationItem>
    suspend fun getMessages(conversationId: String): List<ChatMessage>
    suspend fun sendMessage(conversationId: String, content: String): SendMessageResult
}
