CREATE TABLE IF NOT EXISTS thread_read_state (
    thread_id UUID NOT NULL REFERENCES threads(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    last_read_seq BIGINT NOT NULL DEFAULT 0,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (thread_id, user_id),
    CONSTRAINT thread_read_state_last_read_seq_non_negative CHECK (last_read_seq >= 0)
);

CREATE INDEX IF NOT EXISTS idx_thread_read_state_user_id
    ON thread_read_state(user_id, updated_at DESC);

CREATE INDEX IF NOT EXISTS idx_thread_read_state_thread_id
    ON thread_read_state(thread_id, updated_at DESC);
