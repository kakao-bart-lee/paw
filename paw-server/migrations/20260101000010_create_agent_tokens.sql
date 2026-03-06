CREATE TABLE agent_tokens (
    id            UUID        PRIMARY KEY DEFAULT gen_random_uuid(),
    name          VARCHAR(100) NOT NULL,
    description   TEXT,
    token_hash    VARCHAR(64) NOT NULL UNIQUE,
    owner_user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at    TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    last_used_at  TIMESTAMPTZ
);

CREATE INDEX idx_agent_tokens_hash ON agent_tokens(token_hash);
