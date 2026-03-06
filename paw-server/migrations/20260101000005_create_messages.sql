-- Messages with server-assigned monotonic seq per conversation
-- seq is the source of truth for ordering and gap-fill
CREATE TABLE messages (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    sender_id UUID NOT NULL REFERENCES users(id),
    seq BIGINT NOT NULL,                    -- Monotonic per conversation
    content TEXT NOT NULL,                  -- Plaintext in Phase 1, ciphertext in Phase 2
    format VARCHAR(20) NOT NULL DEFAULT 'markdown',  -- 'markdown', 'plain'
    blocks JSONB DEFAULT '[]'::jsonb,       -- Rich blocks (card, button) for Agent messages
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    deleted_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT messages_valid_format CHECK (format IN ('markdown', 'plain')),
    UNIQUE (conversation_id, seq)           -- Seq is unique per conversation
);

-- Performance indexes
CREATE INDEX idx_messages_conv_seq ON messages(conversation_id, seq);
CREATE INDEX idx_messages_conv_created ON messages(conversation_id, created_at DESC);
CREATE INDEX idx_messages_sender ON messages(sender_id, created_at DESC);

-- Sequence tracking per conversation for monotonic seq assignment
CREATE TABLE conversation_seq (
    conversation_id UUID PRIMARY KEY REFERENCES conversations(id) ON DELETE CASCADE,
    last_seq BIGINT NOT NULL DEFAULT 0
);

-- Function to get next seq atomically
CREATE OR REPLACE FUNCTION next_message_seq(conv_id UUID)
RETURNS BIGINT AS $$
DECLARE
    next_seq BIGINT;
BEGIN
    INSERT INTO conversation_seq (conversation_id, last_seq)
    VALUES (conv_id, 1)
    ON CONFLICT (conversation_id) DO UPDATE
    SET last_seq = conversation_seq.last_seq + 1
    RETURNING last_seq INTO next_seq;
    RETURN next_seq;
END;
$$ LANGUAGE plpgsql;

-- pg_notify trigger for real-time WebSocket fan-out
-- Replaces NATS: PostgreSQL INSERT → Axum WebSocket broadcast
CREATE OR REPLACE FUNCTION notify_new_message()
RETURNS TRIGGER AS $$
BEGIN
    PERFORM pg_notify(
        'new_message',
        json_build_object(
            'id', NEW.id,
            'conversation_id', NEW.conversation_id,
            'sender_id', NEW.sender_id,
            'seq', NEW.seq,
            'content', NEW.content,
            'format', NEW.format,
            'blocks', NEW.blocks,
            'created_at', NEW.created_at
        )::text
    );
    
    -- Update conversation last_message_at
    UPDATE conversations 
    SET last_message_at = NEW.created_at, updated_at = NOW()
    WHERE id = NEW.conversation_id;
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_notify_new_message
    AFTER INSERT ON messages
    FOR EACH ROW
    EXECUTE FUNCTION notify_new_message();
