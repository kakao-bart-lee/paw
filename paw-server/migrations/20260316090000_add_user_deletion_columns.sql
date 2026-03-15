ALTER TABLE users
    ADD COLUMN IF NOT EXISTS deleted_at TIMESTAMPTZ,
    ADD COLUMN IF NOT EXISTS token_revoked_at TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS idx_users_deleted_at
    ON users (deleted_at)
    WHERE deleted_at IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_users_token_revoked_at
    ON users (token_revoked_at)
    WHERE token_revoked_at IS NOT NULL;

ALTER TABLE conversation_members
    ADD COLUMN IF NOT EXISTS left_at TIMESTAMPTZ;

CREATE INDEX IF NOT EXISTS idx_conversation_members_user_left_at
    ON conversation_members (user_id, left_at);
