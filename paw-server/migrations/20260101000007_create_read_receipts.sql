-- Read receipts (last_read_seq is already in conversation_members)
-- This table tracks typing indicators state (ephemeral, for reference)
CREATE TABLE typing_indicators (
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    PRIMARY KEY (conversation_id, user_id)
);

-- Note: Typing indicators use pg_notify directly, not this table
-- This table exists for dashboard/debugging purposes only
