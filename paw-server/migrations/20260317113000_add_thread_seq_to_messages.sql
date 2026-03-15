ALTER TABLE messages
ADD COLUMN IF NOT EXISTS thread_seq BIGINT;

WITH ranked_thread_messages AS (
    SELECT
        id,
        ROW_NUMBER() OVER (
            PARTITION BY thread_id
            ORDER BY seq ASC, created_at ASC, id ASC
        )::BIGINT AS next_thread_seq
    FROM messages
    WHERE thread_id IS NOT NULL
)
UPDATE messages
SET thread_seq = ranked_thread_messages.next_thread_seq
FROM ranked_thread_messages
WHERE messages.id = ranked_thread_messages.id
  AND messages.thread_seq IS NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_messages_thread_thread_seq_unique
ON messages(thread_id, thread_seq)
WHERE thread_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_messages_conversation_thread_thread_seq
ON messages(conversation_id, thread_id, thread_seq)
WHERE thread_id IS NOT NULL;

ALTER TABLE messages
DROP CONSTRAINT IF EXISTS messages_thread_seq_consistency;

ALTER TABLE messages
ADD CONSTRAINT messages_thread_seq_consistency CHECK (
    (thread_id IS NULL AND thread_seq IS NULL)
    OR (thread_id IS NOT NULL AND thread_seq IS NOT NULL)
);

CREATE OR REPLACE FUNCTION notify_new_message()
RETURNS TRIGGER AS $$
BEGIN
    PERFORM pg_notify(
        'new_message',
        json_build_object(
            'id', NEW.id,
            'conversation_id', NEW.conversation_id,
            'thread_id', NEW.thread_id,
            'thread_seq', NEW.thread_seq,
            'sender_id', NEW.sender_id,
            'seq', NEW.seq,
            'content', NEW.content,
            'format', NEW.format,
            'blocks', NEW.blocks,
            'forwarded_from', NEW.forwarded_from,
            'created_at', NEW.created_at
        )::text
    );

    UPDATE conversations
    SET last_message_at = NEW.created_at, updated_at = NOW()
    WHERE id = NEW.conversation_id;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
