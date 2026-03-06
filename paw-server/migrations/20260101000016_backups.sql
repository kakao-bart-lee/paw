-- Encrypted cloud backup metadata
-- Server never stores plaintext backup content — only metadata and presigned URLs

CREATE TABLE backups (
    id          UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id     UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    s3_key      TEXT NOT NULL,
    size_bytes  BIGINT NOT NULL DEFAULT 0,
    created_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    expires_at  TIMESTAMPTZ
);

CREATE INDEX idx_backups_user_id ON backups(user_id);

CREATE TABLE backup_settings (
    user_id     UUID PRIMARY KEY REFERENCES users(id) ON DELETE CASCADE,
    frequency   TEXT NOT NULL DEFAULT 'never' CHECK (frequency IN ('daily', 'weekly', 'never')),
    updated_at  TIMESTAMPTZ NOT NULL DEFAULT now()
);
