-- Device keys for Ed25519 authentication (Signal model - NO SRP)
-- Each user can have multiple devices (phone + desktop + web)
CREATE TABLE devices (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    device_name VARCHAR(100) NOT NULL DEFAULT 'Unknown Device',
    ed25519_public_key BYTEA NOT NULL,  -- 32 bytes Ed25519 public key
    platform VARCHAR(20) NOT NULL,       -- 'ios', 'android', 'web', 'desktop'
    last_active_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT devices_valid_platform CHECK (platform IN ('ios', 'android', 'web', 'desktop', 'cli'))
);

CREATE INDEX idx_devices_user_id ON devices(user_id);
CREATE INDEX idx_devices_pubkey ON devices(ed25519_public_key);
