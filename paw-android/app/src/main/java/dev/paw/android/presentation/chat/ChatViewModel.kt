package dev.paw.android.presentation.chat

import dev.paw.android.domain.model.ChatMessage
import dev.paw.android.domain.model.ChatShellState
import dev.paw.android.domain.model.selectConversationId
import dev.paw.android.domain.repository.ChatRepository
import dev.paw.android.runtime.PawBootstrapPreview
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import java.util.UUID

/**
 * Callback interface for ChatViewModel to notify the coordinator
 * about runtime snapshot sync needs.
 */
interface ChatViewModelCallback {
    fun onChatStateChanged(chat: ChatShellState)
    fun currentPreview(): PawBootstrapPreview
}

class ChatViewModel(
    private val chatRepository: ChatRepository,
    private val callback: ChatViewModelCallback,
    private val scope: CoroutineScope,
) {

    private val _chatState = MutableStateFlow(ChatShellState())
    val chatState: StateFlow<ChatShellState> = _chatState.asStateFlow()

    fun onMessageDraftChanged(value: String) {
        _chatState.value = _chatState.value.copy(messageDraft = value)
    }

    fun selectConversation(conversationId: String) {
        _chatState.value = _chatState.value.copy(
            selectedConversationId = conversationId,
            messages = emptyList(),
            messagesError = null,
        )
        scope.launch {
            loadMessages(conversationId)
        }
    }

    fun retryConversations() {
        scope.launch {
            loadChatShellInternal()
        }
    }

    fun retryMessages() {
        val conversationId = _chatState.value.selectedConversationId ?: return
        scope.launch {
            loadMessages(conversationId)
        }
    }

    fun sendMessage() {
        val conversationId = _chatState.value.selectedConversationId
        if (conversationId == null) {
            _chatState.value = _chatState.value.copy(messagesError = "먼저 대화를 선택하세요.")
            syncRuntime()
            return
        }
        val draft = _chatState.value.messageDraft.trim()
        if (draft.isBlank()) {
            _chatState.value = _chatState.value.copy(messagesError = "메시지를 입력하세요.")
            syncRuntime()
            return
        }

        val optimistic = ChatMessage(
            id = UUID.randomUUID().toString(),
            conversationId = conversationId,
            senderId = "me",
            content = draft,
            format = "plain",
            seq = (_chatState.value.messages.maxOfOrNull { it.seq } ?: 0L) + 1,
            createdAt = System.currentTimeMillis().toString(),
            isMe = true,
            isAgent = false,
        )
        _chatState.value = _chatState.value.copy(
            messageDraft = "",
            sendingMessage = true,
            messagesError = null,
            messages = _chatState.value.messages + optimistic,
        )
        syncRuntime()

        scope.launch {
            runCatching { chatRepository.sendMessage(conversationId, draft) }
                .onSuccess { response ->
                    val confirmed = optimistic.copy(
                        id = response.id.ifBlank { optimistic.id },
                        seq = if (response.seq > 0) response.seq else optimistic.seq,
                        createdAt = response.createdAt.ifBlank { optimistic.createdAt },
                    )
                    _chatState.value = _chatState.value.copy(
                        sendingMessage = false,
                        messages = _chatState.value.messages.map { message ->
                            if (message.id == optimistic.id) confirmed else message
                        },
                        conversations = _chatState.value.conversations.map { conversation ->
                            if (conversation.id == conversationId) {
                                conversation.copy(lastMessage = draft)
                            } else {
                                conversation
                            }
                        },
                    )
                    syncRuntime()
                }
                .onFailure { error ->
                    _chatState.value = _chatState.value.copy(
                        sendingMessage = false,
                        messages = _chatState.value.messages.filterNot { it.id == optimistic.id },
                        messagesError = error.message ?: "메시지를 전송하지 못했습니다.",
                    )
                    syncRuntime()
                }
        }
    }

    fun loadChatShell() {
        scope.launch {
            loadChatShellInternal()
        }
    }

    fun clearChat() {
        _chatState.value = ChatShellState()
        syncRuntime()
    }

    private suspend fun loadChatShellInternal() {
        val hasAccessToken = callback.currentPreview().auth.hasAccessToken
        if (!hasAccessToken) {
            _chatState.value = ChatShellState()
            syncRuntime()
            return
        }

        _chatState.value = _chatState.value.copy(conversationsLoading = true, conversationsError = null)
        runCatching { chatRepository.getConversations() }
            .onSuccess { conversations ->
                val selectedId = selectConversationId(
                    current = _chatState.value.selectedConversationId,
                    conversations = conversations,
                )
                _chatState.value = _chatState.value.copy(
                    conversations = conversations,
                    selectedConversationId = selectedId,
                    messages = if (selectedId == null) emptyList() else _chatState.value.messages,
                    conversationsLoading = false,
                    conversationsError = null,
                )
                syncRuntime()
                selectedId?.let { loadMessages(it) }
            }
            .onFailure { error ->
                _chatState.value = _chatState.value.copy(
                    conversationsLoading = false,
                    conversationsError = error.message ?: "대화를 불러오지 못했습니다.",
                )
                syncRuntime()
            }
    }

    private suspend fun loadMessages(conversationId: String) {
        _chatState.value = _chatState.value.copy(messagesLoading = true, messagesError = null)
        runCatching { chatRepository.getMessages(conversationId) }
            .onSuccess { messages ->
                _chatState.value = _chatState.value.copy(
                    selectedConversationId = conversationId,
                    messages = messages,
                    messagesLoading = false,
                    messagesError = null,
                )
                syncRuntime()
            }
            .onFailure { error ->
                _chatState.value = _chatState.value.copy(
                    selectedConversationId = conversationId,
                    messages = emptyList(),
                    messagesLoading = false,
                    messagesError = error.message ?: "메시지를 불러오지 못했습니다.",
                )
                syncRuntime()
            }
    }

    private fun syncRuntime() {
        callback.onChatStateChanged(_chatState.value)
    }
}
