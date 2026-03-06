CREATE TABLE IF NOT EXISTS conversation_agents (
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    agent_id        UUID NOT NULL REFERENCES agent_tokens(id) ON DELETE CASCADE,
    invited_by      UUID NOT NULL REFERENCES users(id),
    invited_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (conversation_id, agent_id)
);

CREATE INDEX IF NOT EXISTS idx_conversation_agents_conv ON conversation_agents(conversation_id);
