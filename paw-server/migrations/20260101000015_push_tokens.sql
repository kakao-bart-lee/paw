CREATE TABLE push_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_id UUID NOT NULL REFERENCES devices(id) ON DELETE CASCADE,
    platform TEXT NOT NULL CHECK (platform IN ('fcm', 'apns')),
    token TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (device_id)
);

CREATE INDEX idx_push_tokens_user ON push_tokens(user_id);

CREATE TABLE conversation_mutes (
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    muted_until TIMESTAMPTZ,
    PRIMARY KEY (user_id, conversation_id)
);
