-- Moderation: reports, blocks, spam keywords, user suspensions

-- Admin flag on users table (simplest approach for admin checks)
ALTER TABLE users ADD COLUMN IF NOT EXISTS is_admin BOOLEAN NOT NULL DEFAULT false;

-- Reports table
CREATE TABLE IF NOT EXISTS reports (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    reporter_id UUID NOT NULL REFERENCES users(id),
    target_type TEXT NOT NULL CHECK (target_type IN ('message', 'user', 'agent')),
    target_id UUID NOT NULL,
    reason TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'pending',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_reports_status ON reports(status);
CREATE INDEX IF NOT EXISTS idx_reports_reporter ON reports(reporter_id);

-- User blocks
CREATE TABLE IF NOT EXISTS user_blocks (
    blocker_id UUID NOT NULL REFERENCES users(id),
    blocked_id UUID NOT NULL REFERENCES users(id),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (blocker_id, blocked_id)
);

-- Spam keywords
CREATE TABLE IF NOT EXISTS spam_keywords (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    keyword TEXT NOT NULL UNIQUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- User suspensions
CREATE TABLE IF NOT EXISTS user_suspensions (
    user_id UUID PRIMARY KEY REFERENCES users(id),
    suspended_until TIMESTAMPTZ NOT NULL,
    reason TEXT,
    suspended_by UUID NOT NULL REFERENCES users(id)
);
