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

## [2026-03-07] T5: Auth Service Decisions

### OTP and Session Bootstrap
- OTP is 6-digit random numeric code with 5-minute TTL, stored in `otp_codes` and logged via tracing for Phase 1 (no SMS integration).
- OTP verification checks code match + expiry + unused state, then marks the OTP as used with an atomic update guard.
- On successful OTP verification, user is upserted by `phone` and a short-lived `session` JWT is issued for one-time device registration flow.

### Device Registration and Token Model
- Device key registration decodes base64 Ed25519 public key and rejects any key not exactly 32 bytes before DB insert.
- `devices` insert uses `platform='cli'` for current API shape (request payload did not include platform) while satisfying DB constraint.
- Auth tokens are HS256 JWTs with token-type enforcement:
  - `session` (15 minutes) for `/auth/register-device`
  - `access` (7 days)
  - `refresh` (30 days)

### API and Middleware Shape
- Added Axum auth handlers for `/auth/request-otp`, `/auth/verify-otp`, `/auth/register-device`, and `/auth/refresh`.
- Handler responses are JSON with standardized error payload shape: `{ "error": "code", "message": "human readable" }`.
- Added JWT middleware that validates `Authorization: Bearer <token>`, requires `access` token type, and injects `UserId` extension on success.

## [2026-03-07] T7: Auth UI Decisions
- Implemented OTP-based phone authentication flow with 3 screens: PhoneInputScreen, OtpVerifyScreen, DeviceNameScreen.
- Used manual `copyWith` for `AuthState` since `freezed` is not available in `pubspec.yaml`.
- Created custom `PhoneInputField` with country code dropdown and `OtpInputField` with 6 individual boxes supporting auto-advance and paste.
- Updated `LoginScreen` to automatically redirect to `/auth/phone` using `WidgetsBinding.instance.addPostFrameCallback`.
- Added new auth routes to `app_router.dart`.

## [2026-03-07] T8: E2EE Protocol Decision
Recommendation: Use OpenMLS (RFC 9420) for Paw Phase 2 E2EE.
Rationale: MIT license compatibility across Apache-2.0 components, first-class group messaging via TreeKEM/MLS lifecycle, and standards-based long-term protocol direction. Phase 1 PoC validates credential/key package/group creation and member add flow in paw-crypto.

## [2026-03-07] T6: WebSocket Server Decisions

- Added a dedicated `ws` module split into `handler`, `connection`, `hub`, and `pg_listener` to keep upgrade/auth, per-socket loop, fan-out registry, and DB notification concerns isolated.
- WebSocket auth uses existing access JWT verification (`auth::jwt::verify_token`) with token query param (`/ws?token=`), requiring `token_type=access` and a non-null `device_id` claim.
- Server sends `hello_ok` immediately after successful upgrade and enforces protocol version `v=1` for all parsed client frames.
- Heartbeat behavior is implemented as server ping every 30s with a 90s pong timeout; timeout closes with code 1000 (normal closure).
- `Hub` stores `user_id -> Vec<UnboundedSender<Message>>` and supports register/unregister, single-user send, and multi-user fan-out.
- PostgreSQL `LISTEN new_message` integration parses trigger payload JSON and broadcasts `message_received` frames to all online conversation members from `conversation_members`.
- `sync` frame performs gap-fill via `messages WHERE seq > last_seq ORDER BY seq ASC`, returning `message_received` frames with `v=1`.
