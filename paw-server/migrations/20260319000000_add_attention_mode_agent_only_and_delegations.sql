DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_type WHERE typname = 'conversation_attention_mode') THEN
        CREATE TYPE conversation_attention_mode AS ENUM ('all', 'mentions', 'none');
    END IF;
END$$;

ALTER TABLE conversation_members
    ADD COLUMN IF NOT EXISTS attention_mode conversation_attention_mode NOT NULL DEFAULT 'all';

ALTER TABLE conversations
    ADD COLUMN IF NOT EXISTS is_agent_only BOOLEAN NOT NULL DEFAULT FALSE;

CREATE TABLE IF NOT EXISTS agent_delegations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    from_agent_id UUID NOT NULL REFERENCES agent_tokens(id) ON DELETE CASCADE,
    target_agent_id UUID NOT NULL REFERENCES agent_tokens(id) ON DELETE CASCADE,
    delegated_by UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    task_description TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT agent_delegations_non_empty_task CHECK (char_length(trim(task_description)) > 0),
    CONSTRAINT agent_delegations_distinct_agents CHECK (from_agent_id <> target_agent_id)
);

CREATE INDEX IF NOT EXISTS idx_agent_delegations_conversation_created
    ON agent_delegations(conversation_id, created_at DESC);

CREATE INDEX IF NOT EXISTS idx_agent_delegations_target_created
    ON agent_delegations(target_agent_id, created_at DESC);
