CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TABLE IF NOT EXISTS threads (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    conversation_id UUID NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    root_message_id UUID NOT NULL REFERENCES messages(id),
    title TEXT,
    created_by UUID NOT NULL REFERENCES users(id),
    message_count INTEGER NOT NULL DEFAULT 0,
    last_message_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT threads_root_message_unique UNIQUE (root_message_id),
    CONSTRAINT threads_id_conversation_unique UNIQUE (id, conversation_id)
);

CREATE INDEX IF NOT EXISTS idx_threads_conversation_id ON threads(conversation_id);
CREATE INDEX IF NOT EXISTS idx_threads_created_by ON threads(created_by);

CREATE TABLE IF NOT EXISTS thread_agents (
    thread_id UUID NOT NULL,
    conversation_id UUID NOT NULL,
    agent_id UUID NOT NULL REFERENCES agent_tokens(id) ON DELETE CASCADE,
    bound_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    PRIMARY KEY (thread_id, agent_id),
    CONSTRAINT thread_agents_thread_conversation_fk
        FOREIGN KEY (thread_id, conversation_id)
        REFERENCES threads(id, conversation_id)
        ON DELETE CASCADE,
    CONSTRAINT thread_agents_conversation_agent_fk
        FOREIGN KEY (conversation_id, agent_id)
        REFERENCES conversation_agents(conversation_id, agent_id)
        ON DELETE CASCADE,
    CONSTRAINT thread_agents_conversation_agent_unique UNIQUE (conversation_id, agent_id)
);

CREATE INDEX IF NOT EXISTS idx_thread_agents_conversation_id ON thread_agents(conversation_id);
CREATE INDEX IF NOT EXISTS idx_thread_agents_agent_id ON thread_agents(agent_id);

-- DOWN:
-- DROP TABLE IF EXISTS thread_agents;
-- DROP TABLE IF EXISTS threads;
