CREATE TABLE IF NOT EXISTS agent_permissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
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
    granted_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_agent_permissions_unique
    ON agent_permissions (agent_id, conversation_id, permission);

CREATE INDEX IF NOT EXISTS idx_agent_permissions_conversation_agent
    ON agent_permissions (conversation_id, agent_id);
