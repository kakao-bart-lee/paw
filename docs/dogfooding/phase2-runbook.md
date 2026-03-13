# Paw Phase 2 Dogfooding Runbook

## Purpose

This runbook defines how dogfooders run Paw Phase 2 locally (server + client + agent gateway) and report issues during the Phase 2 dogfooding window.

## Scope

- Phase: **Phase 2 (E2EE, Agent Gateway, Python SDK, OpenClaw adapter)**
- Stack: Rust/Axum server + Flutter client + PostgreSQL + MinIO + NATS JetStream
- Features: E2EE (X25519+AES-GCM), Agent Gateway, Python SDK, OpenClaw adapter, group chat, streaming

## Prerequisites Checklist

- [ ] macOS/Linux with Docker and Docker Compose installed
- [ ] Git clone of `paw/`
- [ ] Rust toolchain installed (`~/.cargo/bin/cargo` available)
- [ ] Python 3.10+ installed (for Python SDK)
- [ ] Node.js 18+ installed (for OpenClaw adapter)
- [ ] `.env` configured from `.env.example`
- [ ] Ports free: `3000`, `5432`, `9000`, `9001`, `4222`, `8222`
- [ ] Flutter SDK available **only for client runners**

## Environment Setup

From repository root:

```bash
cp .env.example .env
```

Required env values (additions for Phase 2):

- `NATS_URL=nats://localhost:34223`
- `PAW_HOST=0.0.0.0`
- `PAW_PORT=38173`
- `DATABASE_URL=postgresql://paw:paw_dev_password@localhost:35432/paw_dev`
- `S3_ENDPOINT=http://localhost:39080`
- `S3_BUCKET=paw-media`
- `S3_ACCESS_KEY=paw_minio`
- `S3_SECRET_KEY=paw_minio_password`
- `JWT_SECRET=<at least 32 chars>`

## Start Infrastructure (PostgreSQL + MinIO + NATS)

From repository root:

```bash
docker-compose up -d
```

Expected exposed services:

- PostgreSQL: `localhost:35432`
- MinIO API: `localhost:39080`
- MinIO Console: `http://localhost:39081`
- NATS JetStream: `localhost:34223`
- NATS Management: `http://localhost:38223`

## Start Paw Server

From repository root:

```bash
~/.cargo/bin/cargo run -p paw-server
```

Server default endpoint: `http://localhost:38173`

## Agent Gateway: Register an Agent Token

To register a new agent and get a token:

```bash
curl -X POST http://localhost:38173/api/v1/agents/register \
  -H "Authorization: Bearer <your_user_session_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Echo Agent",
    "description": "A simple echo agent for testing"
  }'
```

The response will contain a `token` field.

## Python SDK: Run Echo Agent

1. Install the SDK in editable mode:
```bash
pip install -e agents/paw-agent-sdk
```

2. Run the echo agent example:
```bash
export PAW_AGENT_TOKEN="<your_agent_token>"
python agents/paw-agent-sdk/examples/echo_agent.py
```

Note: Update `echo_agent.py` to use the environment variable or replace the placeholder token.

## E2EE Testing

1. **Key Bundle Upload**: Ensure your client uploads a prekey bundle upon registration.
   - `POST /api/v1/keys/bundle`
2. **Conversation**: Start a 1:1 conversation. The client should fetch the recipient's bundle.
   - `GET /api/v1/keys/bundle/:user_id`
3. **Verify 🔒 Banner**: In the Flutter client, verify that the conversation view displays the "End-to-end encrypted" banner.
4. **Message Flow**: Send a message and verify it is encrypted (X25519 ECDH + AES-GCM).

## Group Chat Testing

1. **Create Group**: Create a conversation with multiple members.
```bash
curl -X POST http://localhost:38173/api/v1/conversations \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Test Group",
    "member_ids": ["<user_id_1>", "<user_id_2>"]
  }'
```
2. **Invite Members**: Add more members to an existing group.
   - `POST /api/v1/conversations/:id/members`
3. **Send Messages**: Verify all members receive messages in the group.

## Agent Streaming Testing

1. **Invite Agent**: Invite your registered agent to a conversation.
```bash
curl -X POST http://localhost:38173/conversations/:id/agents \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "agent_id": "<agent_uuid>"
  }'
```
2. **Send Message**: Send a message to the conversation.
3. **Watch Stream**: Verify the agent responds with a stream:
   - `stream_start`
   - `content_delta` (multiple)
   - `stream_end`

## OpenClaw Adapter

To run the TypeScript adapter locally:

```bash
cd adapters/openclaw-adapter
npm install
npm run build
# To run an example (e.g., slack-bot)
node dist/examples/slack-bot.js
```

## Dogfooding Session Checklist (Per Tester)

- [ ] E2EE 1:1 conversation works (🔒 banner visible)
- [ ] Group chat creation and messaging works
- [ ] Agent registration and token retrieval successful
- [ ] Python SDK echo agent connects and responds
- [ ] Agent streaming (start -> delta -> end) verified in client
- [ ] OpenClaw adapter builds and connects
- [ ] At least 1 bug report or UX feedback item submitted

## Performance Expectations (Phase 2 Targets)

- HTTP p95 latency `< 250ms` (slightly higher due to E2EE overhead)
- WS message RTT p95 `< 250ms`
- Agent stream delta latency `< 100ms`
- Max concurrent streams per agent: `10`
- Max delta size: `4096 bytes`

## Known Limitations & Phase 3 Readiness Gates

### Known Phase 2 Limitations
- E2EE is X25519+AES-GCM at transport layer; MLS group E2EE (TreeKEM) is Phase 3.
- OpenClaw adapter E2EE bridge is a stub (`looksLikeCiphertext` detection only).
- Agent streaming backpressure: `MAX_CONCURRENT_STREAMS=10`, `MAX_DELTA_SIZE=4096 bytes`.
- Group max members: `100`.
- No push notifications (Phase 3).
- No web/desktop production builds (build-only in Phase 2).
- Drift `.g.dart` files are stubs until `build_runner` runs.

### Phase 3 Readiness Gates
- MLS TreeKEM group E2EE implementation.
- CRDT/Yjs collaborative editing.
- Push notification service.
- Production deployment (Kubernetes, TLS termination).
- OpenClaw adapter full E2EE bridge.
- Web and Desktop production builds.

## Reporting Rules

- Product/functional defects: use `docs/dogfooding/bug-report-template.md`
- UX/product feedback: add item to `docs/dogfooding/feedback-tracker.md`
- End-of-cycle review: complete `docs/dogfooding/phase2-retrospective.md`

## Stop / Cleanup

From repository root:

```bash
docker-compose down
```
