//! Database schema documentation
//!
//! Tables:
//! - users: User accounts (phone/email optionality, username, phone privacy flags)
//! - devices: Ed25519 public keys per device (Signal model, NO SRP)
//! - otp_codes: One-time passwords for authentication
//! - conversations: 1:1 and group conversations (max 100 members)
//! - conversation_members: Members with roles and last_read_seq
//! - conversation_seq: Monotonic seq counter per conversation
//! - messages: Messages with server-assigned seq
//! - media_attachments: S3 media references
//! - typing_indicators: Ephemeral typing state
//!
//! Key invariants:
//! - (conversation_id, seq) is UNIQUE — server guarantees ordering
//! - pg_notify on INSERT — enables WebSocket fan-out without NATS (Phase 1)
//! - NATS introduced in Phase 2 for Agent Gateway routing only
