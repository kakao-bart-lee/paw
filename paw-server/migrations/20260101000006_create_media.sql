-- Media attachments stored in S3-compatible storage
CREATE TABLE media_attachments (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    message_id UUID REFERENCES messages(id) ON DELETE SET NULL,
    uploader_id UUID NOT NULL REFERENCES users(id),
    media_type VARCHAR(20) NOT NULL,   -- 'image', 'video', 'audio', 'file'
    mime_type VARCHAR(100) NOT NULL,
    file_name VARCHAR(500) NOT NULL,
    file_size BIGINT NOT NULL,          -- bytes
    s3_key TEXT NOT NULL,              -- S3 object key
    thumbnail_s3_key TEXT,             -- thumbnail for images/videos
    width INT,                          -- for images/videos
    height INT,                         -- for images/videos
    duration_ms INT,                    -- for audio/video
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT media_valid_type CHECK (media_type IN ('image', 'video', 'audio', 'file')),
    CONSTRAINT media_max_size CHECK (file_size <= 104857600)  -- 100MB max
);

CREATE INDEX idx_media_message ON media_attachments(message_id);
CREATE INDEX idx_media_uploader ON media_attachments(uploader_id);
