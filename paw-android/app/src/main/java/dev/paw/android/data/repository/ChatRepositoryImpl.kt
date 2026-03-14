package dev.paw.android.data.repository

import dev.paw.android.data.remote.ApiClientContract
import dev.paw.android.domain.model.ChatMessage
import dev.paw.android.domain.model.ConversationItem
import dev.paw.android.domain.repository.ChatRepository
import dev.paw.android.domain.repository.SendMessageResult

class ChatRepositoryImpl(
    private val apiClient: ApiClientContract,
) : ChatRepository {

    override suspend fun getConversations(): List<ConversationItem> =
        apiClient.getConversations()

    override suspend fun getMessages(conversationId: String): List<ChatMessage> =
        apiClient.getMessages(conversationId)

    override suspend fun sendMessage(conversationId: String, content: String): SendMessageResult {
        val result = apiClient.sendMessage(conversationId, content)
        return SendMessageResult(
            id = result.id,
            seq = result.seq,
            createdAt = result.createdAt,
        )
    }
}
