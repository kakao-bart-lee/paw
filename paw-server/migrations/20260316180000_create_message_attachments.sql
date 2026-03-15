CREATE TABLE message_attachments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    message_id UUID NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    media_id UUID NOT NULL REFERENCES media_attachments(id) ON DELETE RESTRICT,
    file_type TEXT NOT NULL,
    file_url TEXT NOT NULL,
    file_size BIGINT NOT NULL,
    mime_type TEXT NOT NULL,
    thumbnail_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT message_attachments_file_size_non_negative CHECK (file_size >= 0),
    CONSTRAINT message_attachments_message_media_unique UNIQUE (message_id, media_id)
);

CREATE INDEX idx_message_attachments_message_id ON message_attachments(message_id);
