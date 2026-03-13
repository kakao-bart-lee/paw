ALTER TABLE users
    ADD COLUMN username VARCHAR(32) UNIQUE,
    ADD COLUMN discoverable_by_phone BOOLEAN NOT NULL DEFAULT FALSE,
    ADD COLUMN phone_verified_at TIMESTAMPTZ;

CREATE INDEX idx_users_username ON users(username) WHERE username IS NOT NULL;
CREATE INDEX idx_users_phone_discoverable
    ON users(phone)
    WHERE phone IS NOT NULL AND discoverable_by_phone = TRUE;
