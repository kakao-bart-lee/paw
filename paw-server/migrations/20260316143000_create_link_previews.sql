CREATE TABLE link_previews (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    url TEXT NOT NULL,
    title TEXT,
    description TEXT,
    image_url TEXT,
    fetched_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT link_previews_message_url_unique UNIQUE (message_id, url),
    CONSTRAINT link_previews_https_image_only CHECK (
        image_url IS NULL OR image_url LIKE 'https://%'
    )
);

CREATE INDEX idx_link_previews_message_id ON link_previews(message_id, fetched_at DESC);
