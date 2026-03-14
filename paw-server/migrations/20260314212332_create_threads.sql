CREATE EXTENSION IF NOT EXISTS pgcrypto;

DO $$
BEGIN
    IF NOT EXISTS (
        SELECT 1
        FROM pg_constraint
        WHERE conname = 'messages_id_conversation_unique'
    ) THEN
        ALTER TABLE messages
            ADD CONSTRAINT messages_id_conversation_unique UNIQUE (id, conversation_id);
    END IF;
END $$;

CREATE TABLE IF NOT EXISTS threads (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    root_message_id UUID NOT NULL,
    title TEXT,
    created_by UUID NOT NULL REFERENCES users(id),
    message_count INTEGER NOT NULL DEFAULT 0,
    last_message_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT threads_root_message_unique UNIQUE (root_message_id),
    CONSTRAINT threads_id_conversation_unique UNIQUE (id, conversation_id),
    CONSTRAINT threads_root_message_conversation_fk
        FOREIGN KEY (root_message_id, conversation_id)
        REFERENCES messages(id, conversation_id)
);

CREATE INDEX IF NOT EXISTS idx_threads_conversation_id ON threads(conversation_id);
CREATE INDEX IF NOT EXISTS idx_threads_created_by ON threads(created_by);

CREATE TABLE IF NOT EXISTS thread_agents (
    thread_id UUID NOT NULL,
    conversation_id UUID NOT NULL,
    agent_id UUID NOT NULL,
    bound_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (thread_id, agent_id),
    CONSTRAINT thread_agents_thread_conversation_fk
        FOREIGN KEY (thread_id, conversation_id)
        REFERENCES threads(id, conversation_id)
        ON DELETE CASCADE,
    CONSTRAINT thread_agents_conversation_agent_unique UNIQUE (conversation_id, agent_id)
);

CREATE INDEX IF NOT EXISTS idx_thread_agents_conversation_id ON thread_agents(conversation_id);
CREATE INDEX IF NOT EXISTS idx_thread_agents_agent_id ON thread_agents(agent_id);

-- DOWN:
-- DROP TABLE IF EXISTS thread_agents;
-- DROP TABLE IF EXISTS threads;
-- ALTER TABLE messages DROP CONSTRAINT IF EXISTS messages_id_conversation_unique;
