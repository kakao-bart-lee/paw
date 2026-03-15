ALTER TABLE messages
ADD COLUMN IF NOT EXISTS forwarded_from JSONB;

CREATE OR REPLACE FUNCTION notify_new_message()
RETURNS TRIGGER AS $$
BEGIN
    PERFORM pg_notify(
        'new_message',
        json_build_object(
            'id', NEW.id,
            'conversation_id', NEW.conversation_id,
            'sender_id', NEW.sender_id,
            'seq', NEW.seq,
            'content', NEW.content,
            'format', NEW.format,
            'blocks', NEW.blocks,
            'forwarded_from', NEW.forwarded_from,
            'created_at', NEW.created_at
        )::text
    );

    UPDATE conversations
    SET last_message_at = NEW.created_at, updated_at = NOW()
    WHERE id = NEW.conversation_id;

    RETURN NEW;
END;
$$ LANGUAGE plpgsql;
