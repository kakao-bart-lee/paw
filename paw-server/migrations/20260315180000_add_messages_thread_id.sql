-- Add thread_id to messages table for thread-scoped messaging.
-- NULL = main timeline message, UUID = belongs to that thread.
-- See: Epic #1 (Thread/Topic), ADR-010, contracts/protocol-v1.md

ALTER TABLE messages
    ADD COLUMN IF NOT EXISTS thread_id UUID REFERENCES threads(id) ON DELETE SET NULL;

CREATE INDEX IF NOT EXISTS idx_messages_thread_id ON messages(thread_id)
    WHERE thread_id IS NOT NULL;

-- Composite index for efficient thread message queries:
-- GET /conversations/{id}/threads/{tid}/messages
CREATE INDEX IF NOT EXISTS idx_messages_conversation_thread
    ON messages(conversation_id, thread_id, seq)
    WHERE thread_id IS NOT NULL;
