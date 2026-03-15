CREATE TABLE link_previews (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    url TEXT NOT NULL,
    canonical_url TEXT,
    title TEXT,
    description TEXT,
    image_url TEXT,
    site_name TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    fetched_at TIMESTAMPTZ,
    error_text TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT link_previews_url_unique UNIQUE (url),
    CONSTRAINT link_previews_status_check CHECK (status IN ('pending', 'ready', 'failed'))
);

CREATE INDEX idx_link_previews_status_updated
    ON link_previews(status, updated_at DESC);
