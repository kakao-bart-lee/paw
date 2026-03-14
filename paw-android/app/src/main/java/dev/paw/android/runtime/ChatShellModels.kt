@file:Suppress("unused")
package dev.paw.android.runtime

/**
 * Backward-compatibility aliases. New code should import from domain.model directly.
 */
typealias AndroidConversationItem = dev.paw.android.domain.model.ConversationItem
typealias AndroidChatMessage = dev.paw.android.domain.model.ChatMessage
typealias AndroidChatShellState = dev.paw.android.domain.model.ChatShellState

fun selectConversationId(
    current: String?,
    conversations: List<AndroidConversationItem>,
): String? = dev.paw.android.domain.model.selectConversationId(current, conversations)

fun runtimeSnapshotWithChat(
    base: uniffi.paw_core.RuntimeSnapshot,
    selectedConversationId: String?,
    messages: List<AndroidChatMessage>,
): uniffi.paw_core.RuntimeSnapshot = dev.paw.android.domain.model.runtimeSnapshotWithChat(
    base, selectedConversationId, messages,
)
