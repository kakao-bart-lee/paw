# Paw Phase 3 Dogfooding Runbook

## Purpose & Scope

### Purpose
This runbook defines how dogfooders run Paw Phase 3 locally and report issues during the Phase 3 dogfooding window.

### Scope
- Phase: **Phase 3 (Scale & Polish)**
- Features: Channels, Multi-device Sync, Push Notifications, Cloud Backup, Agent Marketplace, TypeScript SDK, Full-text Search, Desktop/Web QA, Performance, and Moderation.

## Prerequisites

- [ ] macOS/Linux with Docker and Docker Compose installed
- [ ] Git clone of `paw/`
- [ ] Rust toolchain installed (`~/.cargo/bin/cargo` available)
- [ ] Python 3.10+ installed (for Python SDK)
- [ ] Node.js 18+ installed (for TypeScript SDK and OpenClaw adapter)
- [ ] `.env` configured from `.env.example`
- [ ] Ports free: `3000`, `5432`, `9000`, `9001`, `4222`, `8222`
- [ ] Flutter SDK available **only for client runners**

## Environment Setup

From repository root:

```bash
cp .env.example .env
```

Required env values (same as Phase 2):

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

## Channels

1. **Create Channel**:
```bash
curl -X POST http://localhost:38173/api/v1/channels \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Announcements",
    "description": "Official announcements channel"
  }'
```

2. **Subscribe to Channel**:
```bash
curl -X POST http://localhost:38173/api/v1/channels/:id/subscribe \
  -H "Authorization: Bearer <token>"
```

3. **Send Message as Owner**:
```bash
curl -X POST http://localhost:38173/api/v1/channels/:id/messages \
  -H "Authorization: Bearer <owner_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "content": "Hello subscribers!"
  }'
```

4. **Verify**: Ensure subscribers receive the message via WebSocket or `GET /api/v1/channels/:id/messages`.

## Multi-device Sync

1. Connect two devices (e.g., Mobile and Desktop) using the same account.
2. Send a message from Device A.
3. Verify Device B receives a `device_sync` event via WebSocket:
```json
{"v":1,"type":"device_sync","conversations":[{"conversation_id":"...","last_seq":0}]}
```
4. Verify Device B updates its local state to match Device A.

## Push Notifications

1. **Register Token**:
```bash
curl -X POST http://localhost:38173/api/v1/push/register \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "token": "FCM_OR_APNS_TOKEN",
    "platform": "android"
  }'
```

2. **Verify**: Send a message to the user and verify a push notification is triggered. Note: Payload should not contain message content for privacy.

3. **Mute Conversation**:
```bash
curl -X POST http://localhost:38173/api/v1/conversations/:id/mute \
  -H "Authorization: Bearer <token>"
```

## Cloud Backup

1. **Initiate Backup**:
```bash
curl -X POST http://localhost:38173/api/v1/backup/initiate \
  -H "Authorization: Bearer <token>"
```

2. **Upload Encrypted File**: (Follow the multi-part upload flow to S3).

3. **List Backups**:
```bash
curl -X GET http://localhost:38173/api/v1/backup/list \
  -H "Authorization: Bearer <token>"
```

4. **Restore Backup**:
```bash
curl -X POST http://localhost:38173/api/v1/backup/:id/restore \
  -H "Authorization: Bearer <token>"
```

## Agent Marketplace

1. **Search Agents**:
```bash
curl -X GET http://localhost:38173/api/v1/marketplace/agents \
  -H "Authorization: Bearer <token>"
```

2. **Install Agent**:
```bash
curl -X POST http://localhost:38173/api/v1/marketplace/agents/:id/install \
  -H "Authorization: Bearer <token>"
```

3. **Verify Installed**:
```bash
curl -X GET http://localhost:38173/api/v1/marketplace/installed \
  -H "Authorization: Bearer <token>"
```

## TypeScript SDK

1. **Build SDK**:
```bash
cd adapters/paw-sdk-ts
npm install
npm run build
```

2. **Run Echo Bot**:
```bash
export PAW_AGENT_TOKEN="<your_agent_token>"
node dist/examples/echo-bot.js
```

## Full-text Search

1. Open the search screen in the Flutter app.
2. Search for a specific keyword from a previous message.
3. Verify the result is found and the keyword is highlighted in the UI.

## Desktop QA

1. Run the Paw client on macOS:
```bash
flutter run -d macos
```
2. Expand the window width to >768px.
3. Verify the two-panel layout (sidebar + conversation view) is active.

## Web QA

1. Build for web:
```bash
flutter build web
```
2. Open in a browser and verify the PWA manifest is correctly loaded.
3. Verify WebSocket URL conversion (e.g., `http` -> `ws`).

## Moderation

1. **Submit Report**:
```bash
curl -X POST http://localhost:38173/api/v1/reports \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "target_id": "<user_or_message_id>",
    "reason": "spam"
  }'
```

2. **Block User**:
```bash
curl -X POST http://localhost:38173/api/v1/users/:id/block \
  -H "Authorization: Bearer <token>"
```

3. **Verify Spam Detection**: Attempt to send a known spam pattern and verify a `422 Unprocessable Entity` response.

## Performance

1. Run the final benchmark (requires `k6`):
```bash
k6 run k6/final-benchmark.js
```
2. Verify p95 targets:
   - HTTP latency < 200ms
   - WS RTT < 200ms

## Dogfooding Session Checklist

- [ ] Channel creation and subscription works
- [ ] Multi-device sync verified (device_sync event received)
- [ ] Push token registration successful
- [ ] Cloud backup initiated and listed
- [ ] Agent installed from marketplace
- [ ] TypeScript SDK builds and echo bot runs
- [ ] Full-text search returns highlighted results
- [ ] Desktop two-panel layout verified
- [ ] Web PWA manifest and WS connection verified
- [ ] User blocking and spam detection works
- [ ] Performance p95 targets met

## Known Limitations & Phase 3 Readiness

### Known Limitations
- Production deployment requires Kubernetes and real TLS termination.
- Push notifications require real FCM/APNs certificates.
- Cloud backup requires a production-grade S3/CDN setup.

### Phase 3 Readiness
- All Phase 3 features implemented and verified locally.
- Performance targets met under simulated load.
- Security audit for E2EE and Moderation APIs completed.

## Reporting Rules

- Product/functional defects: use `docs/dogfooding/bug-report-template.md`
- UX/product feedback: add item to `docs/dogfooding/feedback-tracker.md`
- End-of-cycle review: complete `docs/dogfooding/phase3-retrospective.md`

## Stop / Cleanup

From repository root:

```bash
docker-compose down
```
