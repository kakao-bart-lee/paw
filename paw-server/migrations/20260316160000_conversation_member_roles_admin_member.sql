ALTER TABLE conversation_members
    ADD COLUMN IF NOT EXISTS role TEXT;

UPDATE conversation_members
SET role = 'admin'
WHERE role = 'owner';

UPDATE conversation_members
SET role = 'member'
WHERE role IS NULL;

ALTER TABLE conversation_members
    ALTER COLUMN role SET DEFAULT 'member';

ALTER TABLE conversation_members
    ALTER COLUMN role SET NOT NULL;

ALTER TABLE conversation_members
    DROP CONSTRAINT IF EXISTS members_valid_role;

ALTER TABLE conversation_members
    ADD CONSTRAINT members_valid_role CHECK (role IN ('admin', 'member'));
