CREATE TABLE IF NOT EXISTS agent_tool_calls (
    id TEXT PRIMARY KEY,
    message_id UUID REFERENCES messages(id) ON DELETE SET NULL,
    agent_id UUID NOT NULL REFERENCES agent_tokens(id) ON DELETE CASCADE,
    tool_name TEXT NOT NULL,
    arguments JSONB,
    result JSONB,
    status TEXT NOT NULL,
    started_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ,
    CONSTRAINT agent_tool_calls_status_check
        CHECK (status IN ('started', 'running', 'completed', 'error'))
);

CREATE INDEX IF NOT EXISTS idx_agent_tool_calls_agent_id
    ON agent_tool_calls(agent_id, started_at DESC);

CREATE INDEX IF NOT EXISTS idx_agent_tool_calls_message_id
    ON agent_tool_calls(message_id);
