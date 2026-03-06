# Paw Phase 1 Dogfooding Runbook

## Purpose

This runbook defines how dogfooders run Paw Phase 1 locally (server + client) and report issues during the 2-week dogfooding window.

## Scope

- Phase: **Phase 1 (TLS-only, no E2EE)**
- Stack: Rust/Axum server + Flutter client + PostgreSQL + MinIO
- Out of scope: Phase 2 features (E2EE, Agent Gateway, OpenClaw adapter)

## Prerequisites Checklist

- [ ] macOS/Linux with Docker and Docker Compose installed
- [ ] Git clone of `paw/`
- [ ] Rust toolchain installed (`~/.cargo/bin/cargo` available)
- [ ] `.env` configured from `.env.example`
- [ ] Ports free: `3000`, `5432`, `9000`, `9001`
- [ ] Flutter SDK available **only for client runners** (if unavailable, server-only dogfooding is still valid)

## Environment Setup

From repository root:

```bash
cp .env.example .env
```

Required env values (default local values from `.env.example`):

- `PAW_HOST=0.0.0.0`
- `PAW_PORT=3000`
- `DATABASE_URL=postgresql://paw:paw_dev_password@localhost:5432/paw_dev`
- `S3_ENDPOINT=http://localhost:9000`
- `S3_BUCKET=paw-media`
- `S3_ACCESS_KEY=paw_minio`
- `S3_SECRET_KEY=paw_minio_password`
- `JWT_SECRET=<at least 32 chars>`

## Start Infrastructure (PostgreSQL + MinIO)

From repository root:

```bash
docker-compose up -d
```

Expected exposed services:

- PostgreSQL: `localhost:5432`
- MinIO API: `localhost:9000`
- MinIO Console: `http://localhost:9001`

## Start Paw Server

From repository root:

```bash
~/.cargo/bin/cargo run -p paw-server
```

Alternative (inside `paw-server/`):

```bash
~/.cargo/bin/cargo run
```

Server default endpoint: `http://localhost:3000`

## Connect Client for Dogfooding

1. Ensure server is running on `localhost:3000`.
2. Launch Paw client with the same backend base URL configuration used in your local setup.
3. Run auth flow: OTP request → OTP verify → device registration.
4. Join/create a conversation and verify:
   - send/receive message
   - typing indicator behavior
   - reconnect gap-fill behavior
5. For media, validate upload path and URL retrieval where available.

## Dogfooding Session Checklist (Per Tester)

- [ ] Login flow successful (OTP + device registration)
- [ ] Conversation list loads
- [ ] Message send/receive works
- [ ] WS reconnect recovers missed messages
- [ ] Presence/typing behaves as expected
- [ ] At least 1 bug report or UX feedback item submitted

## Performance Expectations (Phase 1 Targets)

Use these targets when classifying regressions:

- HTTP p95 latency `< 200ms`
- WS message RTT p95 `< 200ms`
- WS connect p95 `< 500ms`
- HTTP error rate `< 5%`
- WS delivery rate `> 99%`

## Known Issues / Current Limitations

- Flutter may be unavailable in some environments; do not block server-side dogfooding on Flutter setup.
- Drift `*.g.dart` files are currently stubs until `build_runner` is executed in a Flutter-capable environment.
- `MediaPicker` is currently a UI stub (sheet appears, file picker action incomplete).
- `메시지 보내기` button in user profile is a stub (SnackBar only).
- `paw-crypto` has a known compilation issue related to `openmls`; this is outside Phase 1 dogfooding scope.

## Reporting Rules

- Product/functional defects: use `docs/dogfooding/bug-report-template.md`
- UX/product feedback: add item to `docs/dogfooding/feedback-tracker.md`
- End-of-cycle review: complete `docs/dogfooding/phase1-retrospective.md`

## Stop / Cleanup

From repository root:

```bash
docker-compose down
```
