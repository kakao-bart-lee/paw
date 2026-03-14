package dev.paw.android.runtime

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import uniffi.paw_core.ConnectionSnapshot
import uniffi.paw_core.ConnectionStateView
import uniffi.paw_core.RuntimeSnapshot

/**
 * Extended chat shell contract tests complementing ChatShellModelsTest.
 * Covers TC-CHAT-04, TC-CHAT-05, TC-CHAT-11 from the platform-independent test contract.
 */
class ChatShellContractTest {

    private fun conversations(vararg ids: String): List<AndroidConversationItem> =
        ids.map { AndroidConversationItem(id = it, name = "Conv $it", lastMessage = null, unreadCount = 0) }

    private fun baseSnapshot(): RuntimeSnapshot = RuntimeSnapshot(
        connection = ConnectionSnapshot(
            state = ConnectionStateView.CONNECTED,
            attempts = 0u,
            pendingReconnectDelayMs = null,
            pendingReconnectEndpoint = null,
        ),
        cursors = emptyList(),
        activeStreams = emptyList(),
    )

    // --- TC-CHAT-04: Selecting an existing conversation ID keeps it ---

    @Test
    fun `TC-CHAT-04 selected conversation id is kept when present in list`() {
        val list = conversations("a", "b", "c")
        assertEquals("b", selectConversationId("b", list))
    }

    @Test
    fun `TC-CHAT-04 first item in list also kept`() {
        val list = conversations("x", "y")
        assertEquals("x", selectConversationId("x", list))
    }

    // --- TC-CHAT-05: Missing ID falls back to first, empty list gives null ---

    @Test
    fun `TC-CHAT-05 missing id falls back to first conversation`() {
        val list = conversations("a", "b")
        assertEquals("a", selectConversationId("missing", list))
    }

    @Test
    fun `TC-CHAT-05 null current with non-empty list falls back to first`() {
        val list = conversations("only")
        assertEquals("only", selectConversationId(null, list))
    }

    @Test
    fun `TC-CHAT-05 empty list returns null`() {
        assertNull(selectConversationId("any", emptyList()))
        assertNull(selectConversationId(null, emptyList()))
    }

    // --- TC-CHAT-11: Runtime cursor reflects max seq ---

    @Test
    fun `TC-CHAT-11 cursor lastSeq reflects maximum seq in messages`() {
        val messages = listOf(
            AndroidChatMessage("1", "conv-1", "me", "hi", "plain", 3, "", true, false),
            AndroidChatMessage("2", "conv-1", "me", "again", "plain", 10, "", true, false),
            AndroidChatMessage("3", "conv-1", "other", "yo", "plain", 7, "", false, false),
        )

        val snapshot = runtimeSnapshotWithChat(
            base = baseSnapshot(),
            selectedConversationId = "conv-1",
            messages = messages,
        )

        assertEquals(1, snapshot.cursors.size)
        assertEquals("conv-1", snapshot.cursors.first().conversationId)
        assertEquals(10L, snapshot.cursors.first().lastSeq)
    }

    @Test
    fun `TC-CHAT-11 no selected conversation produces empty cursors`() {
        val snapshot = runtimeSnapshotWithChat(
            base = baseSnapshot(),
            selectedConversationId = null,
            messages = emptyList(),
        )

        assertEquals(0, snapshot.cursors.size)
    }

    @Test
    fun `TC-CHAT-11 selected conversation with no messages has cursor seq 0`() {
        val snapshot = runtimeSnapshotWithChat(
            base = baseSnapshot(),
            selectedConversationId = "conv-empty",
            messages = emptyList(),
        )

        assertEquals(1, snapshot.cursors.size)
        assertEquals(0L, snapshot.cursors.first().lastSeq)
    }

    @Test
    fun `TC-CHAT-11 cursor filters messages by selected conversation`() {
        val messages = listOf(
            AndroidChatMessage("1", "conv-a", "me", "hi", "plain", 5, "", true, false),
            AndroidChatMessage("2", "conv-b", "me", "yo", "plain", 20, "", true, false),
            AndroidChatMessage("3", "conv-a", "me", "again", "plain", 8, "", true, false),
        )

        val snapshot = runtimeSnapshotWithChat(
            base = baseSnapshot(),
            selectedConversationId = "conv-a",
            messages = messages,
        )

        assertEquals(8L, snapshot.cursors.first().lastSeq)
    }
}
