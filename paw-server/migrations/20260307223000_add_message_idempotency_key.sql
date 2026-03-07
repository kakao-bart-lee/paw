ALTER TABLE messages
ADD COLUMN IF NOT EXISTS idempotency_key UUID NOT NULL DEFAULT uuid_generate_v4();

CREATE UNIQUE INDEX IF NOT EXISTS idx_messages_idempotency
ON messages(conversation_id, sender_id, idempotency_key);
