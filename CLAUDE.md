# Paw -- Rust Platform

## Overview

E2EE messaging platform: Axum REST/WebSocket server + UniFFI client-core SDK (Android/iOS).

## Quick Start

```bash
make bootstrap-local   # install deps, prepare repo
make local-stack       # postgres + minio + migrations + server (port 38173)
make dev               # server + Flutter web client (default: chrome)
make test              # cargo test --workspace
make lint              # cargo clippy --workspace -- -D warnings
```

Env vars (defaults in main.rs): `DATABASE_URL`, `JWT_SECRET`, `NATS_URL`, `PAW_HOST`, `PAW_PORT`, `PAW_DEFAULT_LOCALE`.

## Architecture Rules (MUST follow)

- Dependency direction: paw-proto -> paw-crypto -> paw-core. paw-server depends on paw-proto only. Reverse imports forbidden.
- All WS messages MUST include `"v": 1` (see `PROTOCOL_VERSION` in paw-proto).
- Message ordering: monotonic seq via `next_message_seq()` PL/pgSQL function.
- Auth: OTP + Ed25519. NO SRP (see paw-hq ADR-001).
- E2EE: OpenMLS only. NO vodozemac/AGPL (see paw-hq ADR-002).
- Pub/Sub: pg_notify Phase 1-3, NATS JetStream for horizontal scaling (see paw-hq ADR-003).
- New API endpoint -> MUST add to `docs/api/openapi.yaml`.
- New DB schema change -> MUST use migration file (`make migrate-add name=...`), no direct ALTER.
- paw-proto changes -> backward compatible only (add fields/variants, never remove or rename).
- Tests required for new features (80%+ coverage target).

## Workspace Structure

| Crate | Role |
|---|---|
| `paw-proto` | Shared WS protocol types (`ClientMessage`, `ServerMessage` enums, serde-tagged) |
| `paw-crypto` | E2EE primitives (OpenMLS wrapper) |
| `paw-core` | Client-side SDK: auth state machine, sync engine, WS reconnect, local DB, UniFFI bindings |
| `paw-server` | Axum HTTP/WS server: REST API, auth, media, channels, agents, moderation, push |

## Code Navigation

### paw-server
- `paw-server/src/main.rs` -- entry point, all route registration, AppState init
- `paw-server/src/auth/mod.rs` -- AppState struct, auth module root
- `paw-server/src/auth/handlers.rs` -- OTP request/verify, device register, token refresh
- `paw-server/src/auth/middleware.rs` -- JWT auth middleware
- `paw-server/src/ws/handler.rs` -- WebSocket upgrade handler
- `paw-server/src/ws/connection.rs` -- per-connection WS message loop
- `paw-server/src/ws/hub.rs` -- in-memory connection registry, broadcast
- `paw-server/src/ws/pg_listener.rs` -- pg_notify listener for real-time fan-out
- `paw-server/src/messages/handlers.rs` -- conversation CRUD, send/get messages
- `paw-server/src/agents/handlers.rs` -- agent registration, marketplace, agent WS
- `paw-server/src/i18n.rs` -- locale middleware, error localization
- `paw-server/src/db/mod.rs` -- connection pool creation (sqlx + postgres)
- `paw-server/migrations/` -- sequential SQL migrations (sqlx)

### paw-core
- `paw-core/src/core.rs` -- CoreRuntime: top-level client orchestrator
- `paw-core/src/ws/service.rs` -- WS client, connection state machine
- `paw-core/src/sync/engine.rs` -- SyncEngine: message ordering, gap detection
- `paw-core/src/auth/state_machine.rs` -- client auth flow state machine
- `paw-core/src/events/mod.rs` -- CoreEvent enum (all events emitted to UI layer)

### paw-proto / paw-crypto
- `paw-proto/src/lib.rs` -- ClientMessage/ServerMessage enums, all WS frame types
- `paw-crypto/src/mls.rs` -- OpenMLS group E2EE operations

## Testing

```bash
make test              # all workspace tests
make e2e-flutter       # Flutter integration tests (device=auto)
make e2e-playwright    # Playwright web smoke
make e2e-real          # real-server Flutter E2E (macOS)
make e2e-real-web      # real-server Flutter web full-loop E2E
make e2e-core-phase3   # paw-core Phase 3 live smoke against local server
```

## Database

Migrations live in `paw-server/migrations/`. Use `make migrate` to apply, `make migrate-add name=<name>` to create new ones. SQLx with Postgres; default connection: `postgres://postgres:postgres@localhost:35432/paw`.

## System Context

System-wide architecture, interface contracts, and ADRs live in the `paw-hq` repository. Refer there for cross-project decisions (auth strategy, E2EE library choice, pub/sub roadmap).
