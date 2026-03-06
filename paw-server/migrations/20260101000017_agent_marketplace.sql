-- Agent Marketplace: extend agent_tokens with marketplace fields + user installs

ALTER TABLE agent_tokens ADD COLUMN category TEXT;
ALTER TABLE agent_tokens ADD COLUMN tags TEXT[] DEFAULT '{}';
ALTER TABLE agent_tokens ADD COLUMN rating_avg FLOAT DEFAULT 0;
ALTER TABLE agent_tokens ADD COLUMN install_count INT DEFAULT 0;
ALTER TABLE agent_tokens ADD COLUMN manifest JSONB;
ALTER TABLE agent_tokens ADD COLUMN is_public BOOL DEFAULT false;

CREATE INDEX idx_agent_tokens_public ON agent_tokens(is_public) WHERE is_public = true;
CREATE INDEX idx_agent_tokens_category ON agent_tokens(category) WHERE is_public = true;

CREATE TABLE user_installed_agents (
    user_id    UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    agent_id   UUID NOT NULL REFERENCES agent_tokens(id) ON DELETE CASCADE,
    installed_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, agent_id)
);

CREATE INDEX idx_user_installed_agents_user ON user_installed_agents(user_id);
