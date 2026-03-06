ALTER TABLE conversations
    DROP CONSTRAINT IF EXISTS conversations_valid_type;

ALTER TABLE conversations
    ADD CONSTRAINT conversations_valid_type CHECK (type IN ('direct', 'group', 'channel'));

CREATE TABLE channels (
    id UUID PRIMARY KEY REFERENCES conversations(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    owner_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    is_public BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE channel_subscriptions (
    channel_id UUID NOT NULL REFERENCES channels(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    subscribed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (channel_id, user_id)
);

CREATE INDEX idx_channels_public_name ON channels(is_public, name);
CREATE INDEX idx_channel_subscriptions_user ON channel_subscriptions(user_id);
