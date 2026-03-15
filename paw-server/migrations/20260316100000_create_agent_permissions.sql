CREATE TABLE IF NOT EXISTS agent_permissions (
    id BIGSERIAL PRIMARY KEY,
    agent_id UUID NOT NULL REFERENCES agent_tokens(id) ON DELETE CASCADE,
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    permission TEXT NOT NULL CHECK (
        permission IN (
            'read_messages',
            'send_messages',
            'manage_thread',
            'access_history',
            'use_tools'
        )
    ),
    granted_by UUID NOT NULL REFERENCES users(id),
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (agent_id, conversation_id, permission)
);

CREATE INDEX IF NOT EXISTS idx_agent_permissions_lookup
    ON agent_permissions (conversation_id, agent_id);
