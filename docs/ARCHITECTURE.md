# Paw Architecture

## System Overview

Paw is an **AI-native messenger** optimized as the ideal OpenClaw integration channel. It provides real-time, low-latency bidirectional communication between AI agents and human users.

### Hub-and-Spoke Model

```
┌─────────────────────────────────────────────────────────┐
│                    Paw Server (Rust/Axum)              │
│                                                         │
│  ┌──────────────────────────────────────────────────┐  │
│  │  WebSocket Hub (tokio broadcast + pg_notify)    │  │
│  │  - Real-time message delivery                   │  │
│  │  - Presence tracking                            │  │
│  │  - Typing indicators                            │  │
│  └──────────────────────────────────────────────────┘  │
│                         ▲                               │
│         ┌───────────────┼───────────────┐              │
│         │               │               │              │
│    ┌────▼────┐    ┌────▼────┐    ┌────▼────┐         │
│    │ paw-core│    │ Python  │    │ OpenClaw│         │
│    │ + Native│    │ SDK     │    │ Adapter │         │
│    └─────────┘    └─────────┘    └─────────┘         │
│                                                         │
│  ┌──────────────────────────────────────────────────┐  │
│  │  PostgreSQL (messages, users, conversations)    │  │
│  │  MinIO S3 (media, attachments)                  │  │
│  └──────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────┘
```

**Why Paw is optimal for OpenClaw:**
- **Low latency**: WebSocket-based, not polling
- **Bidirectional**: Agents push updates; clients receive instantly
- **Scalable**: PostgreSQL + tokio broadcast handles thousands of concurrent connections
- **Reliable**: Message sequencing (seq field) ensures no loss
- **Future-proof**: Protocol versioning (v field) enables evolution without breaking clients

## Tech Stack

| Component | Technology | Version | Purpose |
|-----------|-----------|---------|---------|
| **Server** | Rust + Axum | 1.x | High-performance async HTTP/WebSocket |
| **Client (Web/Desktop)** | Flutter | 3.41+ | Transitional Web/Desktop client with web/macOS verification gates |
| **Client (Mobile)** | Kotlin + SwiftUI + paw-core | current migration | Native mobile shells over shared Rust runtime |
| **SDK** | Python | 3.10+ | Agent integration, scripting |
| **Database** | PostgreSQL | 16 | Persistent storage, pg_notify for pub/sub |
| **Storage** | MinIO (S3-compatible) | Latest | Media, attachments, file uploads |
| **Auth** | Ed25519 + OTP | Phase 1 | Device keys + one-time passwords (NO SRP) |
| **E2EE** | TBD (vodozemac/openmls) | Phase 2 | End-to-end encryption (evaluated in T8) |
| **Pub/Sub** | PostgreSQL LISTEN/NOTIFY | Phase 1 | Real-time message delivery (no NATS) |
| **Streaming** | tokio broadcast | Phase 2 | Agent response streaming (reserved) |

## Monorepo Structure

```
paw/
├── Cargo.toml                    # Rust workspace root
├── Makefile                      # Dev commands (make dev, make test)
├── docker-compose.yml            # PostgreSQL + MinIO services
├── .env.example                  # Environment variables template
├── .github/workflows/
│   ├── core.yml
│   ├── android.yml
│   ├── ios.yml
│   ├── flutter.yml
│   └── server.yml
│
├── paw-server/                   # Rust server (Axum)
│   ├── Cargo.toml
│   ├── src/
│   │   ├── main.rs              # Server entry point
│   │   ├── handlers/            # HTTP/WebSocket handlers
│   │   ├── models/              # Domain types
│   │   ├── db/                  # Database layer (sqlx)
│   │   ├── ws/                  # WebSocket logic
│   │   └── auth/                # Authentication (OTP + Ed25519)
│   └── migrations/              # SQLx migrations
│
├── paw-proto/                    # Protocol types (shared)
│   ├── Cargo.toml
│   └── src/lib.rs               # Message enums, serialization
│
├── paw-crypto/                   # E2EE (Phase 2)
│   ├── Cargo.toml
│   └── src/lib.rs               # Placeholder (TBD after T8)
│
├── docs/
│   ├── ARCHITECTURE.md           # This file
│   ├── protocol-v1.md            # WebSocket protocol spec (T3)
│   ├── e2ee-evaluation.md        # E2EE library comparison (T8)
│   └── deployment.md             # Production setup (T9)
│
├── paw-core/                     # Shared native runtime
├── paw-android/                  # Android native shell scaffold
├── paw-ios/                      # iOS native shell scaffold
└── paw-client/                   # Flutter Web/Desktop path
    ├── pubspec.yaml
    ├── lib/
    │   ├── main.dart
    │   ├── screens/
    │   ├── widgets/
    │   └── services/
    └── test/
```

## Phase Breakdown

### Phase 1: Core Messaging (Current)
**Goal**: Reliable real-time messaging with OTP auth.

**Features**:
- ✅ WebSocket connection management
- ✅ Message send/receive with sequencing
- ✅ Typing indicators
- ✅ Presence tracking (online/offline)
- ✅ OTP + Ed25519 device key authentication
- ✅ PostgreSQL persistence
- ✅ MinIO media uploads
- ✅ TLS encryption (transport layer)

**Tech**:
- Axum WebSocket handlers
- PostgreSQL + sqlx
- tokio broadcast for real-time delivery
- pg_notify for cross-process pub/sub

**No E2EE**: Phase 1 uses TLS only. E2EE deferred to Phase 2 pending library evaluation.

### Phase 2: Agent Streaming & E2EE
**Goal**: Enable AI agent response streaming and end-to-end encryption.

**Features**:
- Streaming message deltas (for agent responses)
- Tool invocation tracking
- E2EE with evaluated library (vodozemac or openmls)
- Message reactions
- Conversation threading

**Tech**:
- tokio broadcast for streaming
- E2EE library (TBD in T8)
- Protobuf for streaming frames (optional)

### Phase 3: Advanced Features
**Goal**: Scalability, federation, and advanced UX.

**Features**:
- NATS for distributed pub/sub (replaces pg_notify)
- Message search (Elasticsearch or similar)
- Conversation federation
- Rich media (voice, video)
- Conversation encryption keys (group E2EE)

## Security Model

### Phase 1: TLS Only
- **Transport**: TLS 1.3 (enforced)
- **Auth**: OTP (one-time password) + Ed25519 device keys
- **No E2EE**: Messages encrypted in transit but decrypted on server
- **Why**: Simplifies Phase 1, allows server-side features (search, moderation)

### Phase 2: E2EE
- **Library**: vodozemac (Signal protocol) or openmls (MLS) — evaluated in T8
- **Key Exchange**: Double Ratchet or MLS tree
- **Server Role**: Stores encrypted messages, cannot read content

### Why NOT SRP
- **SRP is authentication, not encryption**: Doesn't protect message content
- **Paw uses Ed25519 keys**: Simpler, faster, post-quantum resistant (with hybrid)
- **OTP is stateless**: No password database to breach

## Why PostgreSQL (Not NATS) in Phase 1

| Aspect | PostgreSQL | NATS |
|--------|-----------|------|
| **Persistence** | ✅ Built-in | ❌ Requires separate DB |
| **Pub/Sub** | ✅ LISTEN/NOTIFY | ✅ Native |
| **Complexity** | ✅ Single service | ❌ Two services |
| **Scaling** | ⚠️ Vertical | ✅ Horizontal |
| **Cost** | ✅ Lower (Phase 1) | ⚠️ Higher |

**Decision**: PostgreSQL for Phase 1 (simpler, fewer moving parts). NATS added in Phase 3 when horizontal scaling is critical.

## Message Protocol (v1)

All WebSocket messages include a `v` field (currently `1`) for protocol versioning.

### Client → Server
```json
{
  "type": "message_send",
  "v": 1,
  "conversation_id": "uuid",
  "content": "Hello, Paw!",
  "format": "markdown",
  "idempotency_key": "uuid"
}
```

### Server → Client
```json
{
  "type": "message_received",
  "v": 1,
  "id": "uuid",
  "conversation_id": "uuid",
  "sender_id": "uuid",
  "content": "Hello, Paw!",
  "format": "markdown",
  "seq": 42,
  "created_at": "2026-01-01T00:00:00Z"
}
```

**Key fields**:
- `v`: Protocol version (enables future evolution)
- `seq`: Message sequence number (prevents loss/duplication)
- `format`: markdown or plain text
- `idempotency_key`: Client-generated UUID for deduplication

See `docs/protocol-v1.md` for full specification (T3).

## Development Workflow

### Local Setup
```bash
# Start services
make docker-up

# Run server
make dev

# Run tests
make test

# Format & lint
make fmt
make lint
```

### Database Migrations
```bash
# Create migration
make migrate-add name=create_users_table

# Run migrations
make migrate
```

### CI/CD
- GitHub Actions runs on push to `main` and `develop`
- Rust: format check, clippy, build, test
- Flutter: analyze, test, build web
- See `.github/workflows/ci.yml`

## Deployment (T9)

- **Server**: Docker container (Rust binary)
- **Database**: Managed PostgreSQL (AWS RDS, Heroku, etc.)
- **Storage**: S3 or MinIO
- **Auth**: JWT tokens (issued after OTP verification)
- **TLS**: Let's Encrypt (auto-renewal)

See `docs/deployment.md` (created in T9).

## Future Considerations

1. **Horizontal Scaling**: Add NATS in Phase 3 for multi-server deployments
2. **Message Search**: Elasticsearch or Meilisearch for full-text search
3. **Voice/Video**: WebRTC integration (Phase 3)
4. **Moderation**: Content filtering, user reports (Phase 2)
5. **Analytics**: Message metrics, user engagement (Phase 3)
6. **Backup/Recovery**: Automated snapshots, disaster recovery (T9)

## References

- Rust: https://www.rust-lang.org/
- Axum: https://github.com/tokio-rs/axum
- Flutter: https://flutter.dev/
- PostgreSQL: https://www.postgresql.org/
- MinIO: https://min.io/
- Signal Protocol: https://signal.org/docs/
- MLS: https://messaginglayersecurity.rocks/
