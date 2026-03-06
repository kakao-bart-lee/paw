-- OTP verification codes for authentication (Signal model)
CREATE TABLE otp_codes (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    phone VARCHAR(20),
    email VARCHAR(255),
    code VARCHAR(10) NOT NULL,
    attempt_count INT NOT NULL DEFAULT 0,
    max_attempts INT NOT NULL DEFAULT 5,
    expires_at TIMESTAMPTZ NOT NULL,
    used_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    
    CONSTRAINT otp_phone_or_email CHECK (phone IS NOT NULL OR email IS NOT NULL)
);

CREATE INDEX idx_otp_codes_phone ON otp_codes(phone) WHERE phone IS NOT NULL AND used_at IS NULL;
CREATE INDEX idx_otp_codes_email ON otp_codes(email) WHERE email IS NOT NULL AND used_at IS NULL;
-- Auto-cleanup: expires_at index for cleanup jobs
CREATE INDEX idx_otp_codes_expires ON otp_codes(expires_at);
