ALTER TABLE agent_tokens ADD COLUMN avatar_url TEXT;
ALTER TABLE agent_tokens ADD COLUMN revoked_at TIMESTAMPTZ;
