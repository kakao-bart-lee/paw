ALTER TABLE threads
    ADD COLUMN IF NOT EXISTS last_seq BIGINT,
    ADD COLUMN IF NOT EXISTS archived_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS archived_by UUID REFERENCES users(id);

CREATE INDEX IF NOT EXISTS idx_threads_conversation_active_created
    ON threads(conversation_id, created_at ASC)
    WHERE archived_at IS NULL;

CREATE INDEX IF NOT EXISTS idx_threads_archived_at
    ON threads(archived_at)
    WHERE archived_at IS NOT NULL;
