-- users.role column (backward-compatible with is_admin boolean)
ALTER TABLE users ADD COLUMN IF NOT EXISTS role VARCHAR(20) NOT NULL DEFAULT 'user';

-- Migrate existing admins
UPDATE users SET role = 'super_admin' WHERE is_admin = true AND role = 'user';

-- Role constraint
ALTER TABLE users ADD CONSTRAINT users_role_check
  CHECK (role IN ('user', 'moderator', 'admin', 'super_admin'));

CREATE INDEX IF NOT EXISTS idx_users_role ON users(role);

-- audit_logs table
CREATE TABLE IF NOT EXISTS audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    actor_id UUID NOT NULL REFERENCES users(id) ON DELETE SET NULL,
    action VARCHAR(100) NOT NULL,
    target_type VARCHAR(50),
    target_id UUID,
    metadata JSONB,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_audit_logs_actor ON audit_logs(actor_id);
CREATE INDEX IF NOT EXISTS idx_audit_logs_created_at ON audit_logs(created_at DESC);
CREATE INDEX IF NOT EXISTS idx_audit_logs_action ON audit_logs(action);
