---
name: migration-author
description: Create PostgreSQL migrations following Paw timestamp conventions
origin: Paw
---

# Migration Author Skill

All database schema changes go through SQLx migrations in
`paw-server/migrations/`.

## Naming Convention

```
YYYYMMDDHHMMSS_description.sql
```

Examples from the codebase:
- `20260101000005_create_messages.sql`
- `20260307223000_add_message_idempotency_key.sql`
- `20260314212332_create_threads.sql`
- `20260315180000_add_messages_thread_id.sql`

Preferred command from repo root:

```bash
make migrate-add name=<description>
```

Direct SQLx command (equivalent):

```bash
cd paw-server && cargo sqlx migrate add <description>
```

This creates a timestamped file automatically.

## Schema Rules

1. **Add-only**: Never remove or rename columns. Drop columns only via a
   separate, future deprecation migration after all readers have stopped
   using the old column.
2. **IF NOT EXISTS**: Use `CREATE TABLE IF NOT EXISTS` and
   `CREATE INDEX IF NOT EXISTS` for idempotent replay.
3. **UUID primary keys**: Always `UUID PRIMARY KEY DEFAULT gen_random_uuid()`
   (or `uuid_generate_v4()` where the extension is already enabled).
4. **Timestamps**: Always `TIMESTAMPTZ NOT NULL DEFAULT NOW()` -- never bare
   `TIMESTAMP`.
5. **Foreign keys with indexes**: Every `REFERENCES` column must have a
   corresponding `CREATE INDEX`.
6. **CHECK constraints**: Name them `<table>_valid_<field>` for consistency
   (e.g., `CONSTRAINT messages_valid_format CHECK (format IN (...))`).
7. **Down migration comment**: Include a `-- DOWN:` comment at the bottom
   describing the reverse operation, even though SQLx does not run it
   automatically.
8. **Extension guard**: If using `gen_random_uuid()`, include
   `CREATE EXTENSION IF NOT EXISTS pgcrypto;` in that migration.

## Monotonic Sequences

For tables that need ordering within a parent (like messages within a
conversation), use the `next_message_seq()` pattern:

```sql
CREATE OR REPLACE FUNCTION next_<entity>_seq(parent_id UUID)
RETURNS BIGINT AS $$
DECLARE
    next_seq BIGINT;
BEGIN
    INSERT INTO <entity>_seq (parent_id, last_seq)
    VALUES (parent_id, 1)
    ON CONFLICT (parent_id) DO UPDATE
    SET last_seq = <entity>_seq.last_seq + 1
    RETURNING last_seq INTO next_seq;
    RETURN next_seq;
END;
$$ LANGUAGE plpgsql;
```

Current project usage:
- `conversation_seq` table + `next_message_seq(conv_id UUID)`
- uniqueness guard `UNIQUE (conversation_id, seq)` on `messages`

## pg_notify Triggers

When a new row must fan out to WebSocket clients in real time, add a trigger:

```sql
CREATE OR REPLACE FUNCTION notify_new_<entity>()
RETURNS TRIGGER AS $$
BEGIN
    PERFORM pg_notify(
        'new_<entity>',
        json_build_object('id', NEW.id, ...)::text
    );
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_notify_new_<entity>
    AFTER INSERT ON <entity>
    FOR EACH ROW
    EXECUTE FUNCTION notify_new_<entity>();
```

Current project usage:
- `notify_new_message()` on `messages`
- channel: `new_message`
- payload includes `id`, `conversation_id`, `sender_id`, `seq`, `content`,
  `format`, `blocks`, `created_at`

## Paw-Specific Patterns

- Message dedupe: `messages` has `idempotency_key UUID` and unique index on
  `(conversation_id, sender_id, idempotency_key)`.
- Thread model: `threads` and `thread_agents` enforce conversation-scoped
  integrity with composite FK `(thread_id, conversation_id)`.
- Thread message queries: partial indexes are used for thread lookups:
  `idx_messages_thread_id` and `idx_messages_conversation_thread` with
  `WHERE thread_id IS NOT NULL`.

## Verification

After writing a migration:

```bash
make migrate
cargo test -p paw-server --test architecture_test
cargo test -p paw-server
```
