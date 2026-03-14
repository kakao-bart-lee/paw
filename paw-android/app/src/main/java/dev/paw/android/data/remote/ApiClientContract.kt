package dev.paw.android.data.remote

import dev.paw.android.domain.model.ChatMessage
import dev.paw.android.domain.model.ConversationItem
import org.json.JSONObject

/**
 * Result of a successfully sent message as returned by the API.
 */
data class SendMessageResult(
    val id: String,
    val seq: Long,
    val createdAt: String,
)

/**
 * Platform-independent contract for the PAW API client.
 * Enables mocking in unit tests without network dependencies.
 */
interface ApiClientContract {
    fun setAccessToken(token: String?)
    suspend fun requestOtp(phone: String): JSONObject
    suspend fun verifyOtp(phone: String, code: String): JSONObject
    suspend fun registerDevice(sessionToken: String, deviceName: String, ed25519PublicKey: String): JSONObject
    suspend fun getMe(): JSONObject
    suspend fun updateMe(username: String, discoverableByPhone: Boolean): JSONObject
    suspend fun getConversations(): List<ConversationItem>
    suspend fun getMessages(conversationId: String): List<ChatMessage>
    suspend fun sendMessage(conversationId: String, content: String): SendMessageResult
    suspend fun registerPush(token: String)
    suspend fun unregisterPush()
}
