# Paw Phase 1 — Performance Benchmark Report

## Test Environment

| Component | Spec |
|-----------|------|
| Server | Rust/Axum + Tokio (paw-server) |
| Database | PostgreSQL with pg_notify fan-out |
| Transport | HTTP REST + WebSocket (Protocol v1) |
| Load tool | k6 (Grafana) |
| Auth | OTP → Ed25519 device key → JWT HS256 |

## Performance Targets

| Metric | Target | Rationale |
|--------|--------|-----------|
| HTTP p95 latency | < 200 ms | Responsive feel for all API calls |
| WS message RTT p95 | < 200 ms | Real-time messaging perception |
| WS connect p95 | < 500 ms | Acceptable cold-connect time |
| Cold start (server) | < 2000 ms | Fast deployment/restart cycles |
| RSS memory | < 512 MB | Fits comfortably in 1 GB containers |
| HTTP error rate | < 5% | High reliability under load |
| WS delivery rate | > 99% | Near-perfect message delivery |

## Test Scenarios

### 1. WebSocket Load Test (`k6/ws_load_test.js`)

| Parameter | Value |
|-----------|-------|
| Virtual users | 100 concurrent |
| Messages per VU | 10 |
| Total messages | 1,000 |
| Duration | 60 seconds |
| Message stagger | 500 ms between sends |

**What it measures:**
- WS connection establishment time (JWT validation on upgrade)
- Round-trip latency: `message_send` → `message_received` via pg_notify
- Message delivery success rate
- Connection stability under sustained load

**Protocol flow per VU:**
1. Connect to `ws://host/ws?token=JWT`
2. Receive `hello_ok` frame (server validates JWT on HTTP upgrade)
3. Send 10 `message_send` frames with unique `idempotency_key`
4. Measure RTT for each `message_received` echo
5. Close after 55 seconds

### 2. HTTP API Load Test (`k6/http_load_test.js`)

Three concurrent scenarios with staggered starts:

| Scenario | VUs | Duration | Start |
|----------|-----|----------|-------|
| Auth flow | 20 | 30s | 0s |
| Conversation CRUD | 30 | 30s | 5s |
| Message throughput | 50 | 30s | 10s |

**Endpoints tested:**

| Endpoint | Method | Auth | Scenario |
|----------|--------|------|----------|
| `/auth/request-otp` | POST | No | Auth flow |
| `/auth/verify-otp` | POST | No | Auth flow |
| `/conversations` | GET | Bearer JWT | Conversation CRUD |
| `/conversations` | POST | Bearer JWT | Conversation CRUD |
| `/conversations/:id/messages` | POST | Bearer JWT | Message throughput |
| `/conversations/:id/messages` | GET | Bearer JWT | Message throughput |

### 3. Rust Integration Tests (`paw-server/tests/integration_test.rs`)

| Category | Tests | Requires Server |
|----------|-------|-----------------|
| JWT token logic | 6 | No |
| Protocol frame serialization | 7 | No |
| Auth HTTP API | 4 | Yes (`#[ignore]`) |
| Message relay HTTP | 2 | Yes (`#[ignore]`) |
| WS gap-fill | 2 | Yes (`#[ignore]`) |
| Health check | 1 | Yes (`#[ignore]`) |

**Compilable tests (run without server):** 13 passing
**Integration tests (require running server + DB):** 9 ignored

## Results

> **Status: PLACEHOLDER** — Actual numbers require running server with seeded data.
> Execute with the commands below and fill in measured values.

### How to Run

```bash
# Rust integration tests (compilable tests only)
~/.cargo/bin/cargo test -p paw-server -p paw-proto

# Rust integration tests (with running server)
PAW_TEST_TOKEN=<jwt> PAW_TEST_CONV_ID=<uuid> \
  ~/.cargo/bin/cargo test -p paw-server --test integration_test -- --ignored

# k6 WebSocket load test
k6 run --env TOKEN=<jwt> --env CONVERSATION_ID=<uuid> k6/ws_load_test.js

# k6 HTTP load test
k6 run --env TOKEN=<jwt> --env CONVERSATION_ID=<uuid> k6/http_load_test.js
```

### Placeholder Results Table

| Metric | Target | Measured | Status |
|--------|--------|----------|--------|
| HTTP p95 latency | < 200 ms | _TBD_ | |
| Auth request-otp p95 | < 300 ms | _TBD_ | |
| Auth verify-otp p95 | < 300 ms | _TBD_ | |
| Conversation list p95 | < 200 ms | _TBD_ | |
| Message send p95 | < 200 ms | _TBD_ | |
| Message get p95 | < 150 ms | _TBD_ | |
| WS message RTT p95 | < 200 ms | _TBD_ | |
| WS connect p95 | < 500 ms | _TBD_ | |
| WS delivery rate | > 99% | _TBD_ | |
| HTTP error rate | < 5% | _TBD_ | |
| Cold start | < 2000 ms | _TBD_ | |
| RSS at 100 WS connections | < 512 MB | _TBD_ | |

### Rust Test Results (Compilable — No Server Required)

```
running 23 tests
  14 passed; 0 failed; 9 ignored
  JWT tests: 6/6 passed
  Protocol tests: 7/7 passed (+ 1 paw-proto)
  Integration tests: 9 ignored (require running server)
```

## Architecture Notes

- **pg_notify fan-out**: Messages inserted via HTTP trigger PostgreSQL NOTIFY on channel `new_message`. The `pg_listener` task deserializes the payload and broadcasts to connected WebSocket clients via the in-memory `Hub`.
- **Idempotency**: Messages keyed by `(conversation_id, sender_id, idempotency_key)`. Duplicate sends return the existing message without re-insert.
- **Gap-fill**: Clients send `sync` frame with `last_seq` per conversation. Server responds with up to 100 messages where `seq > last_seq` ordered ascending.
- **JWT hierarchy**: session (15 min) → access (7 day, includes device_id) → refresh (30 day). WebSocket upgrade requires access token.
- **Monotonic seq**: Each conversation maintains an independent monotonic sequence via `next_message_seq()` PostgreSQL function.

## Next Steps

1. Seed test database with users, conversations, and messages
2. Run k6 benchmarks against local server and populate results table
3. Profile memory with `heaptrack` or `/proc/<pid>/status` RSS under load
4. Measure cold start: `time cargo run --release` to first health check response
5. Set up CI pipeline with k6 threshold assertions for regression detection
