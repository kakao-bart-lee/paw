CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_messages_conv_seq ON messages(conversation_id, seq DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_messages_sender ON messages(sender_id, created_at DESC);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_channel_subs_user ON channel_subscriptions(user_id);
CREATE INDEX CONCURRENTLY IF NOT EXISTS idx_user_installed_agents_user ON user_installed_agents(user_id);
