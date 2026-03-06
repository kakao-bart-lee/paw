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

## [2026-03-07] T9: Message Relay Decisions

- Implemented a dedicated `messages` module (`mod.rs`, `models.rs`, `service.rs`, `handlers.rs`) to isolate REST endpoint contracts from SQL orchestration and membership policy checks.
- `POST /conversations/:conv_id/messages` enforces membership first, performs idempotency lookup by `(conversation_id, sender_id, idempotency_key)`, inserts with `next_message_seq($1)`, and relies on DB trigger-based `pg_notify` fan-out (no manual notify).
- `GET /conversations/:conv_id/messages` uses `after_seq` + capped `limit` (max 50), fetches `limit + 1` rows for `has_more`, and returns results ordered by `seq ASC`.
- `GET /conversations` joins `conversation_members` with `conversations` + latest message lateral query and computes `unread_count` from `last_read_seq` against message seq values.
- `POST /conversations` creates a conversation in a transaction, infers `direct` vs `group`, normalizes optional name, deduplicates members, and inserts creator as `owner`.
- Added protected routes in `main.rs` behind JWT middleware (`auth_middleware`) while preserving existing auth and websocket routes.

## [2026-03-07] T11: Chat UI Decisions
- Implemented `Message` and `Conversation` models with `isMe` and `isAgent` flags for UI rendering.
- Created `chat_provider.dart` using Riverpod with mock data for Phase 1.
- Designed `MessageBubble` with distinct colors for sent (#6C63FF), received (#252525), and agent (#1E2A3A) messages. Agent messages include a 🤖 badge.
- Implemented `ConversationTile` with unread badge and timestamp formatting.
- Built `MessageInput` with expandable text field and dynamic send button state.
- Updated `ConversationsScreen` and `ChatScreen` to use Riverpod providers and render the new widgets.
- Used `ListView.builder` with `reverse: true` in `ChatScreen` to keep newest messages at the bottom and implemented auto-scroll on new message send.

## [2026-03-07] T10: User Profile API Decisions

- Created `users` module (`mod.rs`, `models.rs`, `handlers.rs`) with four REST endpoints: `GET /users/me`, `PATCH /users/me`, `GET /users/search`, `GET /users/:user_id`.
- `User` struct (full profile with phone) is returned only from `/users/me` endpoints; `PublicUser` struct (no phone) is used for `/users/search` and `/users/:user_id` to prevent phone leakage.
- `PATCH /users/me` uses `COALESCE($param, existing_column)` for partial updates — omitted fields retain current values.
- Phone search is exact match only (`WHERE phone = $1`) — no fuzzy/LIKE queries to prevent enumeration attacks.
- All user routes are behind `auth_middleware` via `route_layer` on the existing `protected_routes` Router, reusing the same JWT `UserId` extension pattern as message handlers.
- `MethodRouter::patch()` method used for chaining GET+PATCH on `/users/me` — no `axum::routing::patch` import needed.

## [2026-03-07] T12: Media Upload Decisions

- Used `aws-sdk-s3` v1 (official AWS SDK for Rust) with `force_path_style(true)` for MinIO compatibility. Custom endpoint URL from `S3_ENDPOINT` env var.
- `MediaService` struct wraps `aws_sdk_s3::Client` and bucket name; initialized via `new_from_env()` reading `S3_ENDPOINT`, `S3_BUCKET`, `S3_ACCESS_KEY`, `S3_SECRET_KEY`, `S3_REGION` (defaults to `us-east-1`).
- `MediaAttachment` model aligns with actual migration table `media_attachments` — uses `mime_type`/`file_size`/`media_type`/`file_name` column names (not the task spec's `content_type`/`size_bytes`). Response JSON uses `content_type`/`size_bytes` for client-facing API.
- `media_type` column derived from MIME type prefix: `image/*` → `"image"`, `video/*` → `"video"`, `audio/*` → `"audio"`, everything else → `"file"`. Matches migration CHECK constraint.
- S3 key format: `media/{uploader_id}/{uuid}/{file_name}` — UUID per upload prevents collisions, uploader prefix aids bucket organization.
- Upload route uses `DefaultBodyLimit::max(50MB)` layer applied only to `/media/upload` via a merged sub-router, keeping default limit on other routes.
- Allowed content types enforced server-side: image/jpeg, image/png, image/gif, image/webp, video/mp4, audio/mpeg, application/pdf. Returns 400 for unsupported types, 413 for >50MB.
- Presigned URL TTL is 1 hour. `GET /media/:media_id/url` returns `{ url, expires_at }` with UTC timestamp.
- `AppState` extended with `media_service: Arc<MediaService>` in `auth/mod.rs`. Both media routes placed behind existing `auth_middleware`.

## [2026-03-07] T14: Local DB Decisions

- Drift + SQLCipher for encrypted local storage. Tables: `MessagesTable`, `ConversationsTable`. DAOs: `MessagesDao`, `ConversationsDao`.
- SQLCipher PRAGMA config: `key` (Phase 1 fixed dev key), `cipher_page_size = 4096`, `kdf_iter = 64000`. Phase 2 will derive key from Ed25519 device key via flutter_secure_storage.
- `MessagesTable` has composite index on `(conversation_id, seq)` for fast conversation message queries and gap-fill.
- `getLastSeq(conversationId)` returns `MAX(seq)` for a conversation — critical for reconnection gap-fill sync frame.
- `ConversationsTable.lastSeq` tracks last known server seq per conversation, enabling client-side gap detection.
- DAOs registered in GetIt via `AppDatabase` singleton — `messagesDao` and `conversationsDao` are accessed from the database instance.
- `.g.dart` files are stubs until `flutter pub run build_runner build` is run with Flutter SDK installed.

## [2026-03-07] T15: Markdown Rendering Decisions

- Implemented `MarkdownMessage` using `flutter_markdown` package for rendering CommonMark in message bubbles.
- Created `CodeBlock` and `CodeBlockBuilder` to handle code blocks with syntax highlighting (monospace font) and a copy-to-clipboard button.
- Updated `MessageBubble` to conditionally render `MarkdownMessage` if `message.format == MessageFormat.markdown`, otherwise fallback to plain `Text`.
- Styled markdown elements (headers, lists, blockquotes, code blocks) to match the app's dark theme and message bubble colors (sent vs received).
- Deferred LaTeX and Mermaid rendering to Phase 3+ as per requirements.

## [2026-03-07] T13: WebSocket Client Decisions

- Added `ApiClient` (`lib/core/http/api_client.dart`) with explicit REST wrappers for auth, conversations, messages, and users using server route contracts (`/auth/*`, `/conversations`, `/users/*`).
- Added `WsService` (`lib/core/ws/ws_service.dart`) with tokenized websocket URL conversion (`http/https -> ws/wss`), `connect` frame send, `v:1` client message helpers (`typing_*`, `message_ack`, `sync`), and bounded reconnect attempts.
- Added `WsMessageHandler` (`lib/core/ws/ws_message_handler.dart`) as a dispatcher layer from raw `ServerMessage` stream to typed callbacks (`message_received`, `typing`, `presence`, `hello_ok`, `hello_error`).
- Rewired chat/auth providers from mock-only flow to service-backed flow using GetIt, with mock data retained as fallback when network/ws is unavailable.
- `AuthNotifier` now persists `access_token`/`refresh_token` in `flutter_secure_storage`, restores session on startup, sets bearer token on `ApiClient`, and establishes WS connection after device registration.
- `setDeviceName()` intentionally uses a base64 Ed25519 public key stub placeholder (no key generation) to match T23 ownership boundary.

## [2026-03-07] T16: Read Receipts + Typing Decisions

### Server: Already Implemented — Minimal Changes
- `MessageAck` handler was already in `connection.rs` (T6/T9) — updates `conversation_members.last_read_seq` via `GREATEST`. No separate `read_receipts` table needed; the migration confirms `last_read_seq` lives in `conversation_members`.
- `TypingStart`/`TypingStop` fan-out was already in `connection.rs` (T6) but broadcast to ALL members. Changed to filter out the sender (`user_id`) before broadcast.
- Added optional `user_id: Option<Uuid>` to `TypingMsg` in `paw-proto` so server injects sender identity before fan-out. Field is `skip_serializing_if = "Option::is_none"` + `default` for backward-compatible deserialization from clients that don't send it.

### Flutter: Phase 1 Scope
- `ReadReceiptIndicator`: always shows `sent` status (single gray ✓). `delivered` and `read` states ready but not wired until server push receipts in Phase 2.
- `TypingIndicator`: animated 3-dot bounce with `AnimatedBuilder` + `SingleTickerProviderStateMixin`. Shows generic "상대방이 입력 중..." — Phase 2 will resolve actual user names.
- `TypingNotifier`: `@riverpod` codegen pattern matching existing `chat_provider.dart`. Tracks `Map<conversationId, Set<userId>>` with 5s auto-expire timer to handle missed `typing_stop` frames.
- `.g.dart` codegen files need `flutter pub run build_runner build` before compilation (same as T14 pattern).
