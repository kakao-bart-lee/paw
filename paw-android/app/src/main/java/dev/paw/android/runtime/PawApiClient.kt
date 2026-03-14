package dev.paw.android.runtime

import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.withContext
import org.json.JSONArray
import org.json.JSONObject
import java.io.BufferedReader
import java.io.InputStreamReader
import java.net.HttpURLConnection
import java.net.URL
import java.util.UUID

data class SendMessageResult(
    val id: String,
    val seq: Long,
    val createdAt: String,
)

class PawApiClient(
    private val baseUrl: String,
) {
    private var accessToken: String? = null

    fun setAccessToken(token: String?) {
        accessToken = token
    }

    suspend fun requestOtp(phone: String): JSONObject = requestJson(
        method = "POST",
        path = "/auth/request-otp",
        body = JSONObject().put("phone", phone),
    )

    suspend fun verifyOtp(phone: String, code: String): JSONObject = requestJson(
        method = "POST",
        path = "/auth/verify-otp",
        body = JSONObject().put("phone", phone).put("code", code),
    )

    suspend fun registerDevice(
        sessionToken: String,
        deviceName: String,
        ed25519PublicKey: String,
    ): JSONObject = requestJson(
        method = "POST",
        path = "/auth/register-device",
        body = JSONObject()
            .put("session_token", sessionToken)
            .put("device_name", deviceName)
            .put("ed25519_public_key", ed25519PublicKey),
    )

    suspend fun getMe(): JSONObject = requestJson(method = "GET", path = "/users/me")

    suspend fun updateMe(username: String, discoverableByPhone: Boolean): JSONObject = requestJson(
        method = "PATCH",
        path = "/users/me",
        body = JSONObject()
            .put("username", username)
            .put("discoverable_by_phone", discoverableByPhone),
    )

    suspend fun getConversations(): List<AndroidConversationItem> {
        val payload = requestJson(method = "GET", path = "/conversations")
        val rows = payload.optJSONArray("conversations") ?: JSONArray()
        return List(rows.length()) { index ->
            val row = rows.optJSONObject(index) ?: JSONObject()
            AndroidConversationItem(
                id = row.optString("id"),
                name = row.optString("name").ifBlank { "Conversation ${index + 1}" },
                lastMessage = row.optString("last_message").takeIf { it.isNotBlank() },
                unreadCount = row.optInt("unread_count", 0),
            )
        }
    }

    suspend fun getMessages(conversationId: String): List<AndroidChatMessage> {
        val payload = requestJson(method = "GET", path = "/conversations/$conversationId/messages")
        val rows = payload.optJSONArray("messages") ?: JSONArray()
        return List(rows.length()) { index ->
            val row = rows.optJSONObject(index) ?: JSONObject()
            AndroidChatMessage(
                id = row.optString("id").ifBlank { "msg-$index" },
                conversationId = row.optString("conversation_id").ifBlank { conversationId },
                senderId = row.optString("sender_id").ifBlank { "unknown" },
                content = row.optString("content"),
                format = row.optString("format").ifBlank { "plain" },
                seq = row.optLong("seq", 0L),
                createdAt = row.optString("created_at"),
                isMe = false,
                isAgent = false,
            )
        }.sortedBy { it.seq }
    }

    suspend fun sendMessage(
        conversationId: String,
        content: String,
    ): SendMessageResult {
        val payload = requestJson(
            method = "POST",
            path = "/conversations/$conversationId/messages",
            body = JSONObject()
                .put("content", content)
                .put("format", "plain")
                .put("idempotency_key", UUID.randomUUID().toString()),
        )
        return SendMessageResult(
            id = payload.optString("id"),
            seq = payload.optLong("seq", 0L),
            createdAt = payload.optString("created_at"),
        )
    }

    suspend fun registerPush(token: String) {
        requestJson(
            method = "POST",
            path = "/api/v1/push/register",
            body = JSONObject()
                .put("token", token)
                .put("platform", "fcm"),
        )
    }

    suspend fun unregisterPush() {
        requestJson(method = "DELETE", path = "/api/v1/push/register")
    }

    private suspend fun requestJson(
        method: String,
        path: String,
        body: JSONObject? = null,
    ): JSONObject = withContext(Dispatchers.IO) {
        val url = URL(baseUrl.trimEnd('/') + path)
        val connection = (url.openConnection() as HttpURLConnection).apply {
            requestMethod = method
            connectTimeout = 15_000
            readTimeout = 15_000
            doInput = true
            setRequestProperty("Content-Type", "application/json")
            accessToken?.takeIf { it.isNotBlank() }?.let {
                setRequestProperty("Authorization", "Bearer $it")
            }
            if (body != null) {
                doOutput = true
                outputStream.use { output ->
                    output.write(body.toString().toByteArray())
                }
            }
        }

        val status = connection.responseCode
        val stream = if (status in 200..299) connection.inputStream else connection.errorStream
        val payload = stream?.use {
            BufferedReader(InputStreamReader(it)).readText()
        }.orEmpty()
        connection.disconnect()

        if (status !in 200..299) {
            val message = runCatching {
                JSONObject(payload).optString("message").ifBlank {
                    JSONObject(payload).optString("error")
                }
            }.getOrDefault("HTTP $status")
            throw PawApiException(status, if (message.isBlank()) "HTTP $status" else message)
        }

        if (payload.isBlank()) {
            JSONObject()
        } else {
            JSONObject(payload)
        }
    }
}

class PawApiException(
    val statusCode: Int,
    override val message: String,
) : Exception(message)
