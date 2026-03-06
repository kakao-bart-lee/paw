## [2026-03-06] T2: PostgreSQL Schema Decisions

### Schema Design Choices

1. **seq is per-conversation monotonic BIGINT**
   - `next_message_seq(conv_id)` function uses advisory locking via INSERT ON CONFLICT
   - Enables gap-fill: `SELECT WHERE seq > last_known_seq ORDER BY seq`
   - Client tracks `last_seq` per conversation for reconnection

2. **pg_notify replaces NATS in Phase 1**
   - Trigger fires after INSERT INTO messages
   - Payload: JSON with all message fields
   - Axum server listens via `LISTEN new_message` → broadcasts to WebSocket clients
   - NATS introduced in Phase 2 for Agent Gateway only

3. **Ed25519 device keys, NO SRP**
   - devices table stores `ed25519_public_key BYTEA` (32 bytes)
   - Authentication via OTP → Ed25519 device key registration → JWT
   - SRP never used (Signal model adopted per Metis recommendation)

4. **Phase 2 E2EE migration plan**
   - Will add: `prekey_bundles` table (identity_key, signed_prekey, one_time_prekeys)
   - Will add: `e2ee_sessions` table (ratchet state per conversation+device pair)
- messages.content will be ciphertext instead of plaintext
    - Zero schema changes to Phase 1 tables needed

## [2026-03-06] T3: WebSocket Protocol Design Decisions

### v Field is Mandatory
Every message MUST include `"v": 1`. Parser rejects messages without it.
Enables non-breaking protocol evolution in future versions.

### Dart Types vs Rust Types
- Rust types: `paw-proto/src/lib.rs` (source of truth for server)
- Dart types: `paw-client/lib/core/proto/messages.dart` (manually synced)
- No codegen bridge between Rust↔Dart (too complex for now)
- json_serializable handles Dart ↔ JSON serialization

### Gap-fill Protocol
Client sends `sync` frame on reconnect with `last_seq`.
Server responds with all messages after that seq.
This replaces CRDT complexity with simple SQL: `WHERE seq > last_seq ORDER BY seq`.

### Phase 2 Streaming Types are Reserved
Types defined in Phase 1 but not used.
This prevents breaking changes when streaming is implemented.
