package dev.paw.android.runtime

import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Test
import uniffi.paw_core.ConnectionSnapshot
import uniffi.paw_core.ConnectionStateView
import uniffi.paw_core.RuntimeSnapshot

class ChatShellModelsTest {
    @Test
    fun `select conversation keeps current when still present`() {
        val conversations = listOf(
            AndroidConversationItem("a", "A", null, 0),
            AndroidConversationItem("b", "B", null, 0),
        )

        assertEquals("b", selectConversationId("b", conversations))
    }

    @Test
    fun `select conversation falls back to first item`() {
        val conversations = listOf(AndroidConversationItem("a", "A", null, 0))

        assertEquals("a", selectConversationId("missing", conversations))
        assertNull(selectConversationId(null, emptyList()))
    }

    @Test
    fun `runtime snapshot adds cursor for selected conversation`() {
        val snapshot = runtimeSnapshotWithChat(
            base = RuntimeSnapshot(
                connection = ConnectionSnapshot(
                    state = ConnectionStateView.CONNECTED,
                    attempts = 0u,
                    pendingReconnectDelayMs = null,
                    pendingReconnectUri = null,
                ),
                cursors = emptyList(),
                activeStreams = emptyList(),
            ),
            selectedConversationId = "conv-1",
            messages = listOf(
                AndroidChatMessage("1", "conv-1", "me", "hi", "plain", 3, "", true, false),
                AndroidChatMessage("2", "conv-1", "me", "again", "plain", 7, "", true, false),
            ),
        )

        assertEquals(1, snapshot.cursors.size)
        assertEquals("conv-1", snapshot.cursors.first().conversationId)
        assertEquals(7L, snapshot.cursors.first().lastSeq)
    }
}
