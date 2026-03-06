# Paw — AI-Native Lightweight Messenger

## TL;DR

> **Quick Summary**: OpenClaw 연동에 최적화된 AI-네이티브 메신저 "Paw"를 Rust(서버) + Flutter(클라이언트) 모노레포로 구축한다. "Telegram만큼 쉬운 Agent 연동 + Signal만큼 강한 프라이버시 + Discord만큼 풍부한 메시지 + 어디에도 없는 네이티브 AI 스트리밍"
>
> **Deliverables**:
> - Rust/Axum 기반 메시징 서버 (WebSocket + REST)
> - Flutter 크로스플랫폼 클라이언트 (iOS/Android/Web/Desktop)
> - E2EE 암호화 (vodozemac/MLS 기반, Phase 2)
> - Agent Gateway + 네이티브 스트리밍 프로토콜
> - Python Agent SDK + OpenClaw Channel Adapter
> - Agent 마켓플레이스 (Phase 3)
>
> **Estimated Effort**: XL (38주 / ~9.5개월)
> **Parallel Execution**: YES — 15 waves across 3 phases
> **Critical Path**: T1→T6→T9→T13→T18→T23→T26→T28→T34→T42→T53→F1-F4

---

## Context

### Original Request
OpenClaw가 20+ 메신저에 연동하며 겪는 각 플랫폼의 한계를 분석하고, 그 한계를 모두 해소한 새로운 메신저 "Paw"를 만든다. OpenClaw가 가장 먼저 연동하고 싶어할 메신저.

### Interview Summary
**Key Discussions**:
- 계획 범위: 전체 3 Phase (28주 기획 → Metis 분석 후 38주로 조정)
- E2EE: Phase 1에서 생략 (TLS만), Phase 2에서 vodozemac FFI 추가
- 프로젝트명: "Paw" (paw-server, paw-client, @paw/sdk)
- 레포: 모노레포 (server/ + client/ + sdk/ + adapter/)
- 팀: 소규모 (2-3명), 병렬 Wave 3-4개 동시
- Flutter: 모든 플랫폼 동시 (iOS+Android 필수 QA, Web/Desktop 빌드만)
- 테스트: Tests-after 전략

**Research Findings** (5개 병렬 에이전트):
1. OpenClaw: 23채널, monitor+send+accounts+components+chunk 패턴
2. Rust/Axum: vodozemac(336★), tokio broadcast+DashMap, async-nats
3. Flutter: Flyer Chat UI, Drift+SQLCipher, Riverpod, **Dart Signal Protocol 미존재**
4. NATS/CRDT: JetStream fan-out 성능 이슈, Yjs 채팅 사례 부재
5. E2EE: vodozemac AGPL 라이선스 문제, openmls(MLS) 대안 존재

### Metis Review
**Identified Gaps** (addressed):
- **NATS → PostgreSQL**: JetStream fan-out 18배 성능 저하 (100 sub 기준). Phase 1은 PostgreSQL + tokio broadcast. NATS는 Phase 2 Agent Gateway에서만 도입
- **SRP 제거**: Signal/Telegram/Matrix 모두 기본 인증에 SRP 미사용. Rust/Dart SRP 생태계 저품질 → OTP + Ed25519 디바이스 키 (Signal 모델)
- **flutter_vodozemac AGPL**: 클라이언트 Apache 2.0과 충돌 → 자체 바인딩 작성 또는 openmls(MLS) 대안
- **일정 현실화**: Phase별 2주 dogfooding buffer → 28주→38주
- **미디어 저장소 누락**: S3 호환 오브젝트 스토리지 추가
- **Phase 1 QA 범위**: iOS+Android만 필수 QA, Web/Desktop 빌드만 확인

---

## Work Objectives

### Core Objective
OpenClaw 생태계에서 가장 풍부한 기능을 가장 쉬운 연동으로 제공하는 AI-네이티브 메신저를 구축한다.

### Concrete Deliverables
- `paw-server/`: Rust/Axum WebSocket+REST 서버 (PostgreSQL, S3)
- `paw-client/`: Flutter 크로스플랫폼 클라이언트 (iOS/Android/Web/Desktop)
- `paw-sdk-python/`: Python Agent SDK (스트리밍, 리치 블록)
- `paw-sdk-ts/`: TypeScript Agent SDK (Phase 3)
- `paw-adapter-openclaw/`: OpenClaw Channel Adapter (TypeScript, MIT)
- `paw-crypto/`: Rust E2EE 크레이트 (vodozemac 또는 openmls 기반)

### Definition of Done
- [ ] 1:1 E2EE 채팅이 iOS+Android에서 p95 <200ms로 동작
- [ ] Agent가 네이티브 스트리밍으로 토큰별 응답 (TTFT <1s)
- [ ] OpenClaw Gateway에서 Paw 어댑터로 연동 성공
- [ ] 100명 그룹 채팅 + E2EE 동작
- [ ] 서버 DB에 평문 메시지 부재 확인 (E2EE 검증)

### Must Have
- WebSocket 네이티브 스트리밍 (send+edit 해킹 아님)
- Agent에게 대화 컨텍스트 자동 제공
- 토큰 하나로 Agent 연결 (Telegram만큼 쉬움)
- 마크다운 + 코드 하이라이팅 렌더링
- 로컬 암호화 저장 (SQLCipher)
- 리치 블록 (card + button) (Phase 2)
- E2EE + Agent 참여 (명시적 동의) (Phase 2)
- OpenClaw Channel Adapter 공식 제공 (Phase 2)

### Must NOT Have (Guardrails)
- ❌ SRP 인증 (어떤 Phase에서도) — Signal 모델(OTP+Ed25519) 사용
- ❌ Phase 1에서 NATS JetStream — PostgreSQL + tokio broadcast 사용
- ❌ Phase 1에서 LaTeX/Mermaid 렌더링 — CommonMark + 코드 하이라이팅만
- ❌ Phase 2에서 form/date-picker 블록 — card + button만
- ❌ flutter_vodozemac를 AGPL 라이선스 해결 없이 사용
- ❌ CRDT(Yjs)를 Phase 2 이전에 도입 — 서버 seq 기반 gap-fill 사용
- ❌ Agent SDK 3언어 동시 — Python만 Phase 2 필수
- ❌ 플레이스홀더/모호한 수락 기준 — 모든 기준은 실행 가능한 커맨드
- ❌ 사용자 수동 확인 형태의 QA — 모든 검증 자동화

---

## Verification Strategy (MANDATORY)

> **ZERO HUMAN INTERVENTION** — ALL verification is agent-executed. No exceptions.

### Test Decision
- **Infrastructure exists**: NO (빈 프로젝트)
- **Automated tests**: Tests-after (구현 후 테스트)
- **Framework**: `cargo test` (Rust) + `flutter test` (Dart) + `k6` (부하) + `criterion` (벤치마크)
- **Test setup**: Wave 1에서 CI/CD + 테스트 인프라 구축

### QA Policy
Every task MUST include agent-executed QA scenarios.
Evidence saved to `.sisyphus/evidence/task-{N}-{scenario-slug}.{ext}`.

- **Server API**: Bash (curl) — HTTP/WS 요청, 상태코드+응답 필드 검증
- **Flutter UI**: Playwright (playwright skill) 또는 `flutter test integration_test/`
- **Performance**: k6 WebSocket/HTTP 부하 테스트
- **E2EE**: `cargo test` + 서버 DB 평문 부재 확인 (`psql` 쿼리)

---

## Execution Strategy

### Timeline Overview (38주)

```
Phase 1: Core Messaging (Week 1-12)
├── Wave 1.1 (W1-2):  Foundation + Scaffold ────── 4 parallel
├── Wave 1.2 (W3-4):  Auth + Server Core ────────── 4 parallel
├── Wave 1.3 (W5-7):  Messaging Core ─────────────── 4 parallel
├── Wave 1.4 (W7-9):  Client Integration ──────────── 4 parallel
├── Wave 1.5 (W9-10): Polish + Media ─────────────── 3 parallel
└── Wave 1.6 (W11-12): Dogfooding + Benchmarks ───── 3 parallel

Phase 2: Agent & E2EE (Week 13-26)
├── Wave 2.1 (W13-15): E2EE Foundation ──────────── 3 parallel
├── Wave 2.2 (W15-18): E2EE + Agent Core ─────────── 4 parallel
├── Wave 2.3 (W18-21): Groups + SDK ─────────────── 4 parallel
├── Wave 2.4 (W21-24): OpenClaw + Streaming UI ──── 4 parallel
└── Wave 2.5 (W24-26): Security Audit + Dogfood ─── 3 parallel

Phase 3: Scale & Polish (Week 27-38)
├── Wave 3.1 (W27-30): Scale Infrastructure ──────── 4 parallel
├── Wave 3.2 (W30-33): Developer Platform ─────────── 4 parallel
├── Wave 3.3 (W33-36): Platform QA + Polish ──────── 3 parallel
└── Wave 3.4 (W36-38): Final Dogfooding ──────────── 3 parallel

Wave FINAL (After ALL — 4 parallel review):
├── F1: Plan compliance audit (oracle)
├── F2: Code quality review (unspecified-high)
├── F3: Real QA (unspecified-high + playwright)
└── F4: Scope fidelity check (deep)
```

### Parallel Execution Waves

```
Wave 1.1 (Week 1-2 — Foundation, start immediately):
├── T1:  Monorepo scaffold + CI/CD [quick]
├── T2:  PostgreSQL schema + migrations [unspecified-high]
├── T3:  WebSocket protocol spec + shared types [unspecified-high]
└── T4:  Flutter app scaffold + theme [visual-engineering]

Wave 1.2 (Week 3-4 — Auth + Server Core):
├── T5:  Auth service — OTP + Ed25519 (depends: T2) [deep]
├── T6:  WebSocket server — connections + heartbeat (depends: T1,T3) [deep]
├── T7:  Auth UI — login/register flows (depends: T4) [visual-engineering]
└── T8:  E2EE protocol PoC — vodozemac vs openmls eval (depends: T1) [deep]

Wave 1.3 (Week 5-7 — Messaging Core):
├── T9:  Message relay — send/receive/persist (depends: T2,T6) [deep]
├── T10: User profile + contacts API (depends: T2,T5) [unspecified-high]
├── T11: Chat UI — conversation list + message bubbles (depends: T4,T7) [visual-engineering]
└── T12: Media upload service — S3 compatible (depends: T1,T2) [unspecified-high]

Wave 1.4 (Week 7-9 — Client Integration):
├── T13: WebSocket client + real-time sync (depends: T6,T11) [deep]
├── T14: Local storage — Drift + SQLCipher (depends: T4) [unspecified-high]
├── T15: Markdown rendering — CommonMark + code (depends: T11) [visual-engineering]
└── T16: Read receipts + typing indicators (depends: T9,T13) [unspecified-high]

Wave 1.5 (Week 9-10 — Polish):
├── T17: Media send/receive + preview (depends: T12,T13) [visual-engineering]
├── T18: Offline gap-fill + reconnection (depends: T9,T13,T14) [deep]
└── T19: User profile UI (depends: T10,T11) [visual-engineering]

Wave 1.6 (Week 11-12 — Dogfooding):
├── T20: Performance benchmarking — k6 + integration tests (depends: T18) [unspecified-high]
├── T21: Phase 1 test suite + bug fixes (depends: T20) [unspecified-high]
└── T22: Phase 1 dogfooding (2 weeks) (depends: T21) [deep]

Wave 2.1 (Week 13-15 — E2EE Foundation):
├── T23: Rust FFI bridge — flutter_rust_bridge + crypto lib (depends: T8) [deep]
├── T24: E2EE key management — prekey bundles (depends: T2,T5) [deep]
└── T25: Agent Gateway scaffold — Rust WebSocket (depends: T6) [unspecified-high]

Wave 2.2 (Week 15-18 — E2EE + Agent Core):
├── T26: 1:1 E2EE — encrypt/decrypt + key exchange (depends: T23,T24) [deep]
├── T27: E2EE Flutter UI — key verification, consent (depends: T23,T11) [visual-engineering]
├── T28: Agent streaming protocol — content_delta, tool_* (depends: T25) [deep]
└── T29: Agent auth + registration API (depends: T5,T25) [unspecified-high]

Wave 2.3 (Week 18-21 — Groups + SDK):
├── T30: Group chat backend — up to 100, group E2EE (depends: T26) [deep]
├── T31: Group chat UI (depends: T11,T30) [visual-engineering]
├── T32: Python Agent SDK (depends: T28,T29) [unspecified-high]
└── T33: Rich message blocks — card + button (depends: T9,T11,T28) [visual-engineering]

Wave 2.4 (Week 21-24 — OpenClaw + Threading):
├── T34: OpenClaw Channel Adapter — TypeScript (depends: T28,T29) [unspecified-high]
├── T35: Streaming UI — token-by-token + tool indicators (depends: T13,T28) [visual-engineering]
├── T36: Thread support — backend + UI (depends: T9,T11) [unspecified-high]
└── T37: Agent E2EE — key sharing, consent, revocation (depends: T26,T29) [deep]

Wave 2.5 (Week 24-26 — Security + Dogfood):
├── T38: Agent limits + moderation (depends: T28,T37) [unspecified-high]
├── T39: E2EE security audit prep + external review (depends: T26,T30,T37) [deep]
└── T40: Phase 2 dogfooding + benchmarks (depends: T38,T39) [deep]

Wave 3.1 (Week 27-30 — Scale):
├── T41: Channels — broadcast 1→many (depends: T30) [deep]
├── T42: Multi-device sync — server seq gap-fill (depends: T18,T26) [deep]
├── T43: Push notifications + E2EE (depends: T26) [unspecified-high]
└── T44: Cloud backup — encrypted, optional (depends: T26,T14) [unspecified-high]

Wave 3.2 (Week 30-33 — Developer Platform):
├── T45: Agent marketplace — registry + discovery (depends: T29,T32) [unspecified-high]
├── T46: TypeScript Agent SDK (depends: T32,T34) [unspecified-high]
├── T47: Full-text local search — FTS5 (depends: T14) [unspecified-high]
└── T48: Desktop platform QA (depends: T22) [unspecified-high]

Wave 3.3 (Week 33-36 — Polish):
├── T49: Web platform QA + optimization (depends: T22) [unspecified-high]
├── T50: Performance optimization + CDN (depends: T43) [deep]
└── T51: Moderation tools — spam filter, reports (depends: T38) [unspecified-high]

Wave 3.4 (Week 36-38 — Final):
├── T52: Final performance benchmarking (depends: T50) [unspecified-high]
├── T53: Phase 3 dogfooding (2 weeks) (depends: T52) [deep]
└── T54: Release preparation + documentation (depends: T53) [writing]

Wave FINAL (After ALL tasks — 4 parallel reviews):
├── F1: Plan compliance audit (oracle)
├── F2: Code quality review (unspecified-high)
├── F3: Real manual QA (unspecified-high + playwright)
└── F4: Scope fidelity check (deep)

Critical Path: T1→T6→T9→T13→T18→T23→T26→T28→T34→T41→T52→F1-F4
Parallel Speedup: ~65% faster than sequential
Max Concurrent: 4 (team size 2-3)
```

### Dependency Matrix

| Task | Depends On | Blocks | Wave |
|------|-----------|--------|------|
| T1-T4 | — | T5-T12 | 1.1 |
| T5 | T2 | T10,T24,T29 | 1.2 |
| T6 | T1,T3 | T9,T13,T25 | 1.2 |
| T8 | T1 | T23 | 1.2 |
| T9 | T2,T6 | T16,T18,T33,T36 | 1.3 |
| T13 | T6,T11 | T16,T17,T18,T35 | 1.4 |
| T18 | T9,T13,T14 | T20,T42 | 1.5 |
| T23 | T8 | T26,T27 | 2.1 |
| T26 | T23,T24 | T30,T37,T42,T43,T44 | 2.2 |
| T28 | T25 | T32,T33,T34,T35,T38 | 2.2 |
| T34 | T28,T29 | T46 | 2.4 |
| T41 | T30 | — | 3.1 |
| T52 | T50 | T53 | 3.4 |

### Agent Dispatch Summary

| Wave | Tasks | Categories |
|------|-------|-----------|
| 1.1 | 4 | T1→quick, T2→unspecified-high, T3→unspecified-high, T4→visual-engineering |
| 1.2 | 4 | T5→deep, T6→deep, T7→visual-engineering, T8→deep |
| 1.3 | 4 | T9→deep, T10→unspecified-high, T11→visual-engineering, T12→unspecified-high |
| 1.4 | 4 | T13→deep, T14→unspecified-high, T15→visual-engineering, T16→unspecified-high |
| 1.5 | 3 | T17→visual-engineering, T18→deep, T19→visual-engineering |
| 1.6 | 3 | T20→unspecified-high, T21→unspecified-high, T22→deep |
| 2.1 | 3 | T23→deep, T24→deep, T25→unspecified-high |
| 2.2 | 4 | T26→deep, T27→visual-engineering, T28→deep, T29→unspecified-high |
| 2.3 | 4 | T30→deep, T31→visual-engineering, T32→unspecified-high, T33→visual-engineering |
| 2.4 | 4 | T34→unspecified-high, T35→visual-engineering, T36→unspecified-high, T37→deep |
| 2.5 | 3 | T38→unspecified-high, T39→deep, T40→deep |
| 3.1 | 4 | T41→deep, T42→deep, T43→unspecified-high, T44→unspecified-high |
| 3.2 | 4 | T45→unspecified-high, T46→unspecified-high, T47→unspecified-high, T48→unspecified-high |
| 3.3 | 3 | T49→unspecified-high, T50→deep, T51→unspecified-high |
| 3.4 | 3 | T52→unspecified-high, T53→deep, T54→writing |
| FINAL | 4 | F1→oracle, F2→unspecified-high, F3→unspecified-high, F4→deep |

---

## TODOs

> Implementation + Test = ONE Task. Every task has QA Scenarios.
> **A task WITHOUT QA Scenarios is INCOMPLETE. No exceptions.**

### ═══════════════════════════════════════════
### PHASE 1: Core Messaging (Week 1-12)
### ═══════════════════════════════════════════

- [ ] 1. Monorepo Scaffold + CI/CD

  **What to do**:
  - Cargo workspace 생성: `paw-server`, `paw-proto`, `paw-crypto` 크레이트
  - Flutter 프로젝트 생성: `paw-client/` (iOS/Android/Web/macOS/Windows/Linux)
  - GitHub Actions CI: Rust build+test + Flutter build+test + cross-compile
  - `.env.example`, `docker-compose.yml` (PostgreSQL + MinIO), `Makefile`
  - WebSocket 프로토콜 메시지에 `"v": 1` 필드 포함하는 규칙 문서화
  - 모든 Rust 코드를 단일 cargo workspace로 관리 (iOS FRB 심볼 충돌 방지)

  **Must NOT do**:
  - NATS 도입 금지 (Phase 2까지)
  - SRP 관련 코드 금지

  **Recommended Agent Profile**:
  - **Category**: `quick`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1.1 (with T2, T3, T4)
  - **Blocks**: T5, T6, T8, T12
  - **Blocked By**: None

  **References**:
  - `openclaw-plan.md:316-374` — 기술 스택 설계 (Rust+Axum, Flutter, PostgreSQL)
  - Rust workspace: https://doc.rust-lang.org/cargo/reference/workspaces.html
  - Flutter 프로젝트 구조: https://docs.flutter.dev/get-started/install

  **Acceptance Criteria**:
  - [ ] `cargo build --workspace` → 성공
  - [ ] `flutter build apk` + `flutter build ios` → 성공
  - [ ] `docker-compose up -d` → PostgreSQL + MinIO 실행
  - [ ] GitHub Actions CI 그린

  **QA Scenarios**:
  ```
  Scenario: Rust workspace 빌드 성공
    Tool: Bash
    Steps:
      1. `cargo build --workspace`
      2. Assert: exit code 0, no errors
    Expected Result: 모든 크레이트 컴파일 성공
    Evidence: .sisyphus/evidence/task-1-rust-build.txt

  Scenario: Flutter 멀티플랫폼 빌드
    Tool: Bash
    Steps:
      1. `cd paw-client && flutter build apk --debug`
      2. `flutter build web`
      3. Assert: 각 빌드 exit code 0
    Expected Result: APK + Web 빌드 성공
    Evidence: .sisyphus/evidence/task-1-flutter-build.txt
  ```

  **Commit**: YES
  - Message: `feat(init): monorepo scaffold with rust workspace + flutter`
  - Files: `Cargo.toml`, `paw-server/`, `paw-proto/`, `paw-client/`, `.github/`, `docker-compose.yml`

- [ ] 2. PostgreSQL Schema + Migrations

  **What to do**:
  - `sqlx` 기반 마이그레이션 시스템 설정
  - Core 테이블: `users`, `devices` (Ed25519 pubkey), `conversations`, `conversation_members`, `messages`, `media_attachments`
  - `messages` 테이블: `id BIGSERIAL`, `conversation_id UUID`, `sender_id UUID`, `content TEXT`, `seq BIGINT` (monotonic per conversation), `created_at TIMESTAMPTZ`
  - `pg_notify` 트리거: 새 메시지 INSERT 시 알림
  - 인덱스: `(conversation_id, seq)`, `(sender_id, created_at)`

  **Must NOT do**:
  - E2EE 관련 컬럼은 Phase 2에서 추가 (prekey_bundles 등)
  - NATS 관련 코드 금지

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1.1 (with T1, T3, T4)
  - **Blocks**: T5, T9, T10, T12, T24
  - **Blocked By**: None

  **References**:
  - Metis 권고: PostgreSQL + pg_notify 패턴 (NATS 대체)
  - `sqlx` 마이그레이션: https://github.com/launchbadge/sqlx
  - Signal 스키마 참고: 메시지 seq 기반 순서 보장

  **Acceptance Criteria**:
  - [ ] `sqlx migrate run` → 성공, 모든 테이블 생성
  - [ ] `psql -c "\dt"` → users, devices, conversations, messages 등 확인
  - [ ] INSERT trigger 테스트: 메시지 삽입 시 pg_notify 이벤트 발생

  **QA Scenarios**:
  ```
  Scenario: 마이그레이션 실행 + 스키마 확인
    Tool: Bash
    Steps:
      1. `docker-compose up -d postgres`
      2. `cd paw-server && sqlx migrate run`
      3. `psql $DATABASE_URL -c "\dt"`
      4. Assert: users, devices, conversations, conversation_members, messages 테이블 존재
    Expected Result: 모든 테이블 생성됨
    Evidence: .sisyphus/evidence/task-2-schema.txt

  Scenario: pg_notify 트리거 동작
    Tool: Bash
    Steps:
      1. 터미널 A: `psql -c "LISTEN new_message;"`
      2. 터미널 B: `psql -c "INSERT INTO messages (conversation_id, sender_id, content, seq) VALUES (...)"`
      3. Assert: 터미널 A에서 NOTIFY 수신
    Expected Result: INSERT 시 알림 발생
    Evidence: .sisyphus/evidence/task-2-notify.txt
  ```

  **Commit**: YES
  - Message: `feat(db): postgresql schema with pg_notify triggers`
  - Files: `paw-server/migrations/`, `paw-server/src/db/`

- [ ] 3. WebSocket Protocol Spec + Shared Types

  **What to do**:
  - `paw-proto` 크레이트: 모든 WebSocket 메시지 타입 정의 (Rust serde)
  - 프로토콜 버전: 모든 메시지에 `"v": 1` 필드 포함
  - 메시지 타입:
    - `connect` / `hello_ok` (인증 + 초기 상태)
    - `message_send` / `message_received` / `message_ack`
    - `typing_start` / `typing_stop`
    - `presence_update`
    - `stream_start` / `content_delta` / `tool_start` / `tool_end` / `stream_end` (Phase 2용 예약)
  - Dart 타입 생성: `paw-client/lib/proto/` (수동 또는 codegen)
  - 프로토콜 스펙 문서: `docs/protocol-v1.md`

  **Must NOT do**:
  - 스트리밍 메시지 구현 (타입 정의만, Phase 2에서 구현)

  **Recommended Agent Profile**:
  - **Category**: `unspecified-high`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1.1 (with T1, T2, T4)
  - **Blocks**: T6, T13
  - **Blocked By**: None

  **References**:
  - `openclaw-plan.md:300-311` — 스트리밍 프로토콜 포맷 (stream_start, content_delta, tool_*)
  - `openclaw-plan.md:256-280` — InboundContext + capabilities 설계
  - OpenClaw Gateway 프로토콜: connect → hello_ok → req/res + events

  **Acceptance Criteria**:
  - [ ] `cargo test -p paw-proto` → 모든 serde 직렬화/역직렬화 테스트 통과
  - [ ] Dart 타입이 Rust 타입과 1:1 매칭 확인
  - [ ] 모든 메시지 타입에 `"v": 1` 필드 포함 확인

  **QA Scenarios**:
  ```
  Scenario: 프로토콜 메시지 직렬화 round-trip
    Tool: Bash
    Steps:
      1. `cargo test -p paw-proto -- --nocapture`
      2. Assert: connect, message_send, typing_start 등 모든 타입 serde 테스트 통과
    Expected Result: 100% 테스트 통과
    Evidence: .sisyphus/evidence/task-3-proto-test.txt

  Scenario: 버전 필드 필수 검증
    Tool: Bash
    Steps:
      1. `cargo test -p paw-proto test_version_field_required`
      2. Assert: `"v"` 필드 없는 JSON 파싱 시 에러 반환
    Expected Result: 버전 필드 누락 시 역직렬화 실패
    Evidence: .sisyphus/evidence/task-3-version-check.txt
  ```

  **Commit**: YES
  - Message: `feat(proto): websocket protocol types v1 with version field`
  - Files: `paw-proto/src/`, `paw-client/lib/proto/`, `docs/protocol-v1.md`

- [ ] 4. Flutter App Scaffold + Theme

  **What to do**:
  - Flutter 프로젝트 구조: `lib/` → `core/`, `features/`, `shared/`
  - Riverpod 상태 관리 설정 (`flutter_riverpod`)
  - GetIt DI 컨테이너 설정
  - 다크 모드 기본 테마 (기획서: "어두운 기본 테마")
  - 3-탭 네비게이션: 채팅 / Agent / 설정
  - GoRouter 라우팅 설정
  - iOS/Android/Web/Desktop 플랫폼별 설정

  **Must NOT do**:
  - 실제 백엔드 연결 (mock 데이터 사용)
  - E2EE 관련 UI

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
  - **Skills**: [`frontend-ui-ux`]
    - `frontend-ui-ux`: Flutter UI 컴포넌트 + 테마 설계

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1.1 (with T1, T2, T3)
  - **Blocks**: T7, T11, T14, T15
  - **Blocked By**: None

  **References**:
  - `openclaw-plan.md:548-584` — UI/UX 원칙, 3-탭 내비게이션, 다크 모드
  - Flyer Chat UI: https://pub.dev/packages/flutter_chat_ui
  - Riverpod: https://pub.dev/packages/flutter_riverpod

  **Acceptance Criteria**:
  - [ ] `flutter run` → 앱 실행, 3-탭 네비게이션 표시
  - [ ] 다크 테마 기본 적용 확인
  - [ ] iOS 시뮬레이터 + Android 에뮬레이터에서 동작

  **QA Scenarios**:
  ```
  Scenario: 앱 실행 + 네비게이션
    Tool: Bash (flutter test)
    Steps:
      1. `cd paw-client && flutter test integration_test/app_scaffold_test.dart`
      2. Assert: 3탭(채팅/Agent/설정) 네비게이션 동작
    Expected Result: 모든 탭 전환 성공
    Evidence: .sisyphus/evidence/task-4-navigation.png

  Scenario: 다크 테마 적용
    Tool: Bash (flutter test)
    Steps:
      1. `flutter test test/theme_test.dart`
      2. Assert: ThemeData.brightness == Brightness.dark
    Expected Result: 다크 모드 기본 활성화
    Evidence: .sisyphus/evidence/task-4-theme.txt
  ```

  **Commit**: YES
  - Message: `feat(client): flutter scaffold with dark theme + 3-tab navigation`
  - Files: `paw-client/lib/`

- [ ] 5. Auth Service — OTP + Ed25519 Device Key

  **What to do**:
  - Phone/Email OTP 요청 + 검증 API (`POST /api/v1/auth/otp/request`, `POST /api/v1/auth/otp/verify`)
  - Ed25519 디바이스 키 등록: 클라이언트가 키페어 생성 → pubkey를 서버에 등록
  - JWT 토큰 발급 (access + refresh)
  - 디바이스 관리 API: 등록, 목록, 제거
  - OTP 레이트 리밋: 5회/분, 10회/시간
  - SMS/Email 발송: MVP에서 콘솔 로그 (프로덕션에서 Twilio/SendGrid)

  **Must NOT do**:
  - SRP 프로토콜 구현 금지 (OTP + 디바이스 키만)
  - 비밀번호 기반 인증 금지

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1.2 (with T6, T7, T8)
  - **Blocks**: T10, T24, T29
  - **Blocked By**: T2

  **References**:
  - Metis 권고: Signal 모델 채택 (OTP + Ed25519), SRP 완전 제거
  - `ed25519-dalek` 크레이트: https://docs.rs/ed25519-dalek
  - `jsonwebtoken` 크레이트: https://docs.rs/jsonwebtoken

  **Acceptance Criteria**:
  - [ ] `curl -X POST /api/v1/auth/otp/request -d '{"phone":"+821012345678"}'` → `{"status":"otp_sent"}`
  - [ ] `curl -X POST /api/v1/auth/otp/verify -d '{"phone":"...","code":"123456"}'` → JWT 반환
  - [ ] 6회 연속 OTP 요청 → 429 Too Many Requests

  **QA Scenarios**:
  ```
  Scenario: OTP 인증 플로우 성공
    Tool: Bash (curl)
    Steps:
      1. `curl -s -X POST http://localhost:3000/api/v1/auth/otp/request -H "Content-Type: application/json" -d '{"phone":"+821012345678"}'`
      2. Assert: status 200, body contains "otp_sent"
      3. 서버 로그에서 OTP 코드 추출
      4. `curl -s -X POST http://localhost:3000/api/v1/auth/otp/verify -H "Content-Type: application/json" -d '{"phone":"+821012345678","code":"<OTP>"}'`
      5. Assert: status 200, body contains "access_token" and "refresh_token"
    Expected Result: JWT 토큰 쌍 발급
    Evidence: .sisyphus/evidence/task-5-otp-flow.txt

  Scenario: OTP 레이트 리밋
    Tool: Bash (curl loop)
    Steps:
      1. 6회 연속 OTP 요청 (`for i in {1..6}; do curl ...; done`)
      2. Assert: 6번째 요청 → HTTP 429
    Expected Result: 레이트 리밋 동작
    Evidence: .sisyphus/evidence/task-5-rate-limit.txt
  ```

  **Commit**: YES
  - Message: `feat(auth): otp + ed25519 device key authentication`
  - Files: `paw-server/src/auth/`

- [ ] 6. WebSocket Server — Connection Manager + Heartbeat

  **What to do**:
  - Axum WebSocket 업그레이드 핸들러 (JWT 인증 후 승인)
  - `DashMap<ConnectionId, ConnectionHandle>` 커넥션 레지스트리
  - `tokio::sync::broadcast` 채널: 대화별 메시지 브로드캐스트
  - `pg_notify` 리스너: PostgreSQL INSERT 알림 → WebSocket 브로드캐스트
  - Heartbeat: 30초 ping/pong, 90초 타임아웃 시 연결 종료
  - Graceful shutdown: SIGTERM 시 모든 연결에 close(1000) 전송
  - 연결 당 bounded mpsc 채널 (backpressure: 100 메시지 버퍼)

  **Must NOT do**:
  - NATS 연동 금지 (pg_notify + tokio broadcast로)
  - 스트리밍 구현 금지 (Phase 2)

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1.2 (with T5, T7, T8)
  - **Blocks**: T9, T13, T25
  - **Blocked By**: T1, T3

  **References**:
  - Axum WebSocket docs: https://docs.rs/axum/latest/axum/extract/ws/
  - Tokio broadcast: https://docs.rs/tokio/latest/tokio/sync/broadcast/
  - DashMap: https://docs.rs/dashmap
  - Metis 권고: PostgreSQL + tokio broadcast 패턴 (NATS 대체)

  **Acceptance Criteria**:
  - [ ] WebSocket 연결 성공 (JWT 인증)
  - [ ] 100 동시 연결 유지 + 메시지 브로드캐스트 동작
  - [ ] Heartbeat: 90초 미응답 시 연결 자동 종료

  **QA Scenarios**:
  ```
  Scenario: WebSocket 인증 + 메시지 수신
    Tool: Bash (websocat)
    Steps:
      1. JWT 토큰 획득 (T5 인증 API)
      2. `websocat "ws://localhost:3000/ws?token=$JWT"`
      3. 다른 연결에서 메시지 전송
      4. Assert: 첫 연결에서 메시지 수신
    Expected Result: 실시간 메시지 수신
    Evidence: .sisyphus/evidence/task-6-ws-message.txt

  Scenario: Heartbeat 타임아웃
    Tool: Bash
    Steps:
      1. WebSocket 연결 후 91초간 pong 미전송
      2. Assert: 서버가 연결 종료 (close code 1000)
    Expected Result: 타임아웃 후 연결 종료
    Evidence: .sisyphus/evidence/task-6-heartbeat-timeout.txt
  ```

  **Commit**: YES
  - Message: `feat(ws): websocket server with connection manager + heartbeat`
  - Files: `paw-server/src/ws/`

- [ ] 7. Auth UI — Login/Register Flows

  **What to do**:
  - 전화번호/이메일 입력 → OTP 요청 화면
  - OTP 입력 → 검증 화면 (6자리, 자동 제출)
  - 프로필 설정 화면 (닉네임, 아바타)
  - Ed25519 키페어 로컬 생성 + 서버 등록
  - Riverpod auth state provider
  - 자동 로그인 (저장된 JWT refresh)

  **Must NOT do**:
  - 비밀번호 입력 필드 없음
  - SRP 관련 UI 없음

  **Recommended Agent Profile**:
  - **Category**: `visual-engineering`
  - **Skills**: [`frontend-ui-ux`]

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1.2 (with T5, T6, T8)
  - **Blocks**: T11
  - **Blocked By**: T4

  **References**:
  - `openclaw-plan.md:593-600` — Phase 1 기능: 회원가입/로그인 (전화번호+이메일)
  - Signal 앱 인증 플로우 참고 (전화번호 → OTP → 프로필)

  **Acceptance Criteria**:
  - [ ] OTP 입력 화면 → 6자리 입력 시 자동 제출
  - [ ] 성공 시 메인 화면으로 전환
  - [ ] 앱 재시작 시 자동 로그인

  **QA Scenarios**:
  ```
  Scenario: 회원가입 플로우 완료
    Tool: Bash (flutter test)
    Steps:
      1. `cd paw-client && flutter test integration_test/auth_flow_test.dart`
      2. Assert: 전화번호 입력 → OTP 화면 → 프로필 설정 → 메인 화면
    Expected Result: 전체 인증 플로우 통과
    Evidence: .sisyphus/evidence/task-7-auth-flow.png
  ```

  **Commit**: YES
  - Message: `feat(auth-ui): login/register with otp + profile setup`
  - Files: `paw-client/lib/features/auth/`

- [ ] 8. E2EE Protocol PoC — vodozemac vs openmls Evaluation

  **What to do**:
  - vodozemac (Olm/Megolm) PoC: Rust에서 1:1 키 교환 + 메시지 암/복호화
  - openmls (MLS, RFC 9420) PoC: Rust에서 그룹 키 교환 + 메시지 암/복호화
  - flutter_rust_bridge 통합 PoC: Dart에서 Rust 함수 호출 가능 확인
  - 라이선스 분석: vodozemac(Apache-2.0) vs flutter_vodozemac(AGPL-3.0) vs openmls 라이선스
  - iOS 심볼 충돌 테스트: FRB 기반 라이브러리 2개 동시 사용
  - **결정 보고서**: `docs/e2ee-evaluation.md` — 프로토콜 선택 + 라이선스 해결 방안

  **Must NOT do**:
  - 프로덕션 E2EE 구현 (PoC만)
  - flutter_vodozemac AGPL 패키지 직접 사용 금지

  **Recommended Agent Profile**:
  - **Category**: `deep`
  - **Skills**: []

  **Parallelization**:
  - **Can Run In Parallel**: YES
  - **Parallel Group**: Wave 1.2 (with T5, T6, T7)
  - **Blocks**: T23 (E2EE FFI bridge — PoC 결과에 의존)
  - **Blocked By**: T1

  **References**:
  - vodozemac: https://github.com/matrix-org/vodozemac (Apache-2.0, 336★)
  - openmls: https://github.com/openmls/openmls (MIT)
  - flutter_rust_bridge: https://github.com/aspect-build/aspect
  - Metis 권고: Phase 1 Week 4까지 E2EE 프로토콜 선택 확정

  **Acceptance Criteria**:
  - [ ] vodozemac PoC: `cargo test test_olm_session` → 메시지 암/복호화 성공
  - [ ] openmls PoC: `cargo test test_mls_group` → 그룹 키 교환 성공
  - [ ] FRB PoC: Dart에서 Rust crypto 함수 호출 성공
  - [ ] `docs/e2ee-evaluation.md` 작성 완료 (추천 프로토콜 + 근거)

  **QA Scenarios**:
  ```
  Scenario: vodozemac 1:1 암호화 round-trip
    Tool: Bash
    Steps:
      1. `cargo test -p paw-crypto test_vodozemac_olm_roundtrip`
      2. Assert: 평문 → 암호화 → 복호화 → 원문 일치
    Expected Result: 메시지 무결성 확인
    Evidence: .sisyphus/evidence/task-8-vodozemac-poc.txt

  Scenario: Flutter → Rust FFI 호출
    Tool: Bash
    Steps:
      1. `cd paw-client && flutter test test/ffi_bridge_test.dart`
      2. Assert: Dart에서 Rust 함수 호출 + 결과 반환
    Expected Result: FFI 브릿지 동작 확인
    Evidence: .sisyphus/evidence/task-8-ffi-bridge.txt
  ```

  **Commit**: YES
  - Message: `docs(crypto): e2ee protocol evaluation — vodozemac vs openmls`
  - Files: `paw-crypto/src/poc/`, `docs/e2ee-evaluation.md`

- [ ] 9. Message Relay Service — Send/Receive/Persist

  **What to do**:
  - `POST /api/v1/messages/send` REST API + WebSocket `message_send` 핸들러
  - PostgreSQL INSERT → `pg_notify` → 해당 대화 참여자의 WebSocket 브로드캐스트
  - 메시지 seq 발급 (conversation별 monotonic BIGSERIAL)
  - 메시지 포맷: `{"v":1,"type":"message","content":"...","format":"markdown","blocks":[]}`
  - 메시지 수정/삭제 API (soft delete)
  - 대화 목록 API: `GET /api/v1/conversations` (최근 메시지, 안읽은 수)

  **Recommended Agent Profile**: `deep`
  **Parallelization**: Wave 1.3 | Blocks: T16,T18,T33,T36 | Blocked By: T2,T6

  **Acceptance Criteria**:
  - [ ] 메시지 전송 → 수신 p95 <200ms (로컬 테스트)
  - [ ] `psql` 에서 메시지 확인 + seq 순서 검증

  **QA Scenarios**:
  ```
  Scenario: 메시지 전송 + 실시간 수신
    Tool: Bash (curl + websocat)
    Steps:
      1. User A WebSocket 연결
      2. User B가 `POST /api/v1/messages/send` 로 메시지 전송
      3. Assert: User A WebSocket에서 메시지 수신, seq 순서 올바름
    Expected Result: 실시간 메시지 전달
    Evidence: .sisyphus/evidence/task-9-message-relay.txt
  ```
  **Commit**: YES — `feat(msg): message relay with pg_notify broadcast`

- [ ] 10. User Profile + Contacts API

  **What to do**:
  - `GET/PUT /api/v1/users/me` — 프로필 조회/수정 (닉네임, 아바타 URL, 상태 메시지)
  - `POST /api/v1/contacts/add` — 전화번호/이메일로 연락처 추가
  - `GET /api/v1/contacts` — 연락처 목록
  - `POST /api/v1/conversations/create` — 1:1 대화 생성 (상대방 user_id)
  - 아바타 업로드: S3에 저장, URL 반환

  **Recommended Agent Profile**: `unspecified-high`
  **Parallelization**: Wave 1.3 | Blocks: T19 | Blocked By: T2,T5

  **Acceptance Criteria**:
  - [ ] 프로필 수정 후 조회 시 변경 내용 반영
  - [ ] 연락처 추가 → 대화 생성 → 메시지 전송 가능

  **QA Scenarios**:
  ```
  Scenario: 프로필 수정 + 연락처 + 대화 생성
    Tool: Bash (curl)
    Steps:
      1. `PUT /api/v1/users/me` — 닉네임 "TestUser"
      2. `POST /api/v1/contacts/add` — 전화번호로 추가
      3. `POST /api/v1/conversations/create` — 1:1 대화 생성
      4. Assert: 대화 ID 반환, 참여자 2명
    Expected Result: 대화 생성 성공
    Evidence: .sisyphus/evidence/task-10-contacts.txt
  ```
  **Commit**: YES — `feat(api): user profile + contacts + conversation creation`

- [ ] 11. Chat UI — Conversation List + Message Bubbles

  **What to do**:
  - 대화 목록 화면: 최근 대화, 마지막 메시지 미리보기, 안읽은 뱃지
  - 대화 상세 화면: 메시지 버블 (보낸 메시지 오른쪽, 받은 메시지 왼쪽)
  - `flutter_chat_ui` 기반 또는 커스텀 메시지 버블 위젯
  - Agent 메시지: 일반 버블 + 🤖 배지
  - 무한 스크롤 (이전 메시지 로드)
  - 기획서 UI 디자인 참고 (`openclaw-plan.md:557-584`)

  **Recommended Agent Profile**: `visual-engineering` + `frontend-ui-ux` skill
  **Parallelization**: Wave 1.3 | Blocks: T13,T15,T16,T19,T27,T31,T33 | Blocked By: T4,T7

  **Acceptance Criteria**:
  - [ ] 대화 목록에서 대화 선택 → 메시지 목록 표시
  - [ ] 메시지 입력 + 전송 버튼 동작 (mock 데이터)

  **QA Scenarios**:
  ```
  Scenario: 대화 목록 + 메시지 표시
    Tool: Bash (flutter test)
    Steps:
      1. `flutter test integration_test/chat_ui_test.dart`
      2. Assert: 대화 목록 → 대화 선택 → 메시지 버블 표시
    Expected Result: UI 렌더링 정상
    Evidence: .sisyphus/evidence/task-11-chat-ui.png
  ```
  **Commit**: YES — `feat(chat-ui): conversation list + message bubbles`

- [ ] 12. Media Upload Service — S3 Compatible

  **What to do**:
  - `POST /api/v1/media/upload` — presigned URL 발급 (클라이언트 직접 업로드)
  - `GET /api/v1/media/{id}` — presigned download URL
  - MinIO 로컬 개발, Cloudflare R2/S3 프로덕션
  - 파일 타입 제한: 이미지(jpg/png/gif/webp), 비디오(mp4), 파일(최대 100MB)
  - 이미지 썸네일 자동 생성 (서버사이드)
  - `media_attachments` 테이블: message_id, url, type, size, thumbnail_url

  **Recommended Agent Profile**: `unspecified-high`
  **Parallelization**: Wave 1.3 | Blocks: T17 | Blocked By: T1,T2

  **Acceptance Criteria**:
  - [ ] presigned URL로 10MB 이미지 업로드 성공
  - [ ] 썸네일 자동 생성 확인

  **QA Scenarios**:
  ```
  Scenario: 이미지 업로드 + 썸네일
    Tool: Bash (curl)
    Steps:
      1. `POST /api/v1/media/upload` → presigned URL
      2. `PUT <presigned_url>` — 10MB 이미지 업로드
      3. `GET /api/v1/media/{id}` → download URL + thumbnail URL
      4. Assert: 이미지 + 썸네일 접근 가능
    Expected Result: 업로드 + 썸네일 생성 성공
    Evidence: .sisyphus/evidence/task-12-media-upload.txt
  ```
  **Commit**: YES — `feat(media): s3 compatible upload with thumbnail generation`

- [ ] 13. WebSocket Client + Real-time Sync

  **What to do**:
  - `web_socket_channel` 기반 WebSocket 클라이언트 (Flutter)
  - 자동 재연결 (지수 백오프: 1s → 2s → 4s → 최대 30s)
  - 재연결 시 gap-fill: `last_seq` 전송 → 누락 메시지 수신
  - Riverpod provider: `messageStreamProvider(conversationId)` — 실시간 메시지 스트림
  - 연결 상태 표시 (연결됨/재연결 중/오프라인)

  **Recommended Agent Profile**: `deep`
  **Parallelization**: Wave 1.4 | Blocks: T16,T17,T18,T35 | Blocked By: T6,T11

  **Acceptance Criteria**:
  - [ ] 메시지 전송 → 실시간 UI 업데이트
  - [ ] 네트워크 끊김 → 재연결 → 누락 메시지 자동 수신

  **QA Scenarios**:
  ```
  Scenario: 실시간 메시지 수신
    Tool: Bash (flutter test)
    Steps:
      1. `flutter test integration_test/realtime_sync_test.dart`
      2. Assert: 서버에서 메시지 전송 → 클라이언트 UI 즉시 업데이트
    Expected Result: 실시간 동기화 동작
    Evidence: .sisyphus/evidence/task-13-realtime.txt
  ```
  **Commit**: YES — `feat(ws-client): realtime sync with auto-reconnection`

- [ ] 14. Local Storage — Drift + SQLCipher

  **What to do**:
  - `drift` + `sqflite_sqlcipher` 통합 설정
  - 테이블: messages, conversations, contacts, media_cache, sync_state
  - 암호화 키: 앱 첫 실행 시 랜덤 생성, Keychain/Keystore에 저장
  - 반응형 쿼리: 대화 목록, 메시지 목록 → Stream으로 UI 자동 업데이트
  - 마이그레이션 전략: 스키마 버전 관리

  **Recommended Agent Profile**: `unspecified-high`
  **Parallelization**: Wave 1.4 | Blocks: T18,T44,T47 | Blocked By: T4

  **Acceptance Criteria**:
  - [ ] 앱 종료 → 재시작 → 메시지 목록 로컬에서 로드
  - [ ] DB 파일 직접 열기 시도 → 암호화로 읽기 불가

  **QA Scenarios**:
  ```
  Scenario: 로컬 영속성 + 암호화 확인
    Tool: Bash (flutter test)
    Steps:
      1. 메시지 저장 → 앱 프로세스 종료 → 재시작
      2. Assert: 이전 메시지 로컬에서 로드
      3. `sqlite3 paw.db .tables` → 에러 (암호화됨)
    Expected Result: 메시지 영속 + DB 암호화
    Evidence: .sisyphus/evidence/task-14-local-storage.txt
  ```
  **Commit**: YES — `feat(storage): drift + sqlcipher encrypted local db`

- [ ] 15. Markdown Rendering — CommonMark + Code Highlight

  **What to do**:
  - `flutter_markdown` 기반 마크다운 렌더링 위젯
  - 코드 블록 구문 하이라이팅 (`highlight` 또는 `syntax_highlight` 패키지)
  - 지원 범위: **bold**, *italic*, `inline code`, ```code block```, [링크], 테이블
  - Graceful degradation: 파싱 실패 시 평문 폴백 (Telegram과 차별화)
  - 글자 수 제한 없음 (긴 AI 응답 지원)
  - 하이퍼링크 일반 메시지에서도 지원 (Discord와 차별화)

  **Must NOT do**: LaTeX 수식, Mermaid 다이어그램 (Phase 3+)

  **Recommended Agent Profile**: `visual-engineering` + `frontend-ui-ux` skill
  **Parallelization**: Wave 1.4 | Blocks: — | Blocked By: T11

  **Acceptance Criteria**:
  - [ ] 마크다운 메시지 → 올바르게 렌더링
  - [ ] 잘못된 마크다운 → 평문으로 표시 (에러 없음)

  **QA Scenarios**:
  ```
  Scenario: 마크다운 렌더링 + graceful degradation
    Tool: Bash (flutter test)
    Steps:
      1. `flutter test test/markdown_render_test.dart`
      2. Assert: bold, code, link, code block 렌더링 확인
      3. 깨진 마크다운 입력 → 평문 폴백 확인
    Expected Result: 정상 마크다운 렌더링 + 에러 시 폴백
    Evidence: .sisyphus/evidence/task-15-markdown.png
  ```
  **Commit**: YES — `feat(md): markdown rendering with code highlighting`

- [ ] 16. Read Receipts + Typing Indicators

  **What to do**:
  - 읽음 확인: 메시지 표시 시 `message_read` WebSocket 이벤트 전송 → 상대에게 전달
  - UI: ✓(전송) → ✓✓(읽음) 표시
  - 타이핑 인디케이터: `typing_start`/`typing_stop` WebSocket 이벤트
  - UI: "상대방이 입력 중..." 표시 (3초 타임아웃)
  - 서버 로직: `pg_notify` 또는 직접 WebSocket 릴레이

  **Recommended Agent Profile**: `unspecified-high`
  **Parallelization**: Wave 1.4 | Blocks: — | Blocked By: T9,T13

  **Acceptance Criteria**:
  - [ ] 메시지 읽음 → 상대 화면에 ✓✓ 표시
  - [ ] 타이핑 시작 → 상대 화면에 "입력 중..." 표시

  **QA Scenarios**:
  ```
  Scenario: 읽음 확인 + 타이핑 인디케이터
    Tool: Bash (flutter test)
    Steps:
      1. `flutter test integration_test/read_receipt_test.dart`
      2. User A 메시지 전송 → User B 읽음 → User A에 ✓✓ 표시
      3. User B 타이핑 → User A에 "입력 중..." 표시 → 3초 후 사라짐
    Expected Result: 읽음 확인 + 타이핑 동작
    Evidence: .sisyphus/evidence/task-16-receipts.txt
  ```
  **Commit**: YES — `feat(msg): read receipts + typing indicators`

- [ ] 17. Media Send/Receive + Preview

  **What to do**:
  - 이미지/파일 선택 → presigned URL 업로드 → 메시지에 첨부
  - 이미지 미리보기 (썸네일 → 탭 시 전체 크기)
  - 파일 다운로드 + 공유 (share_plus 패키지)
  - 이미지 뷰어 (줌/패닝), 비디오 플레이어 (video_player)
  - 미디어 갤러리 뷰 (대화 내 모든 미디어 모아보기)

  **Recommended Agent Profile**: `visual-engineering` + `frontend-ui-ux` skill
  **Parallelization**: Wave 1.5 | Blocked By: T12,T13
  **Commit**: YES — `feat(media-ui): media send/receive with preview`

  **QA Scenarios**:
  ```
  Scenario: 이미지 전송 + 미리보기
    Tool: Bash (flutter test)
    Steps:
      1. `flutter test integration_test/media_send_test.dart`
      2. 이미지 선택 → 업로드 → 상대에게 썸네일 표시 → 탭 → 전체 이미지
    Expected Result: 미디어 전송 + 미리보기 동작
    Evidence: .sisyphus/evidence/task-17-media.png
  ```

- [ ] 18. Offline Gap-fill + Reconnection

  **What to do**:
  - 서버 seq 기반 gap-fill: 재연결 시 `{"type":"sync","last_seq":12345}` 전송
  - 서버: `SELECT * FROM messages WHERE conversation_id = $1 AND seq > $2 ORDER BY seq`
  - 벌크 전달 후 실시간 스트림 전환
  - 오프라인 메시지 큐: 로컬에 저장 → 연결 복구 시 순차 전송
  - conflict resolution: 서버 seq가 진실의 원천

  **Recommended Agent Profile**: `deep`
  **Parallelization**: Wave 1.5 | Blocks: T20,T42 | Blocked By: T9,T13,T14
  **Commit**: YES — `feat(sync): offline gap-fill with server seq`

  **QA Scenarios**:
  ```
  Scenario: 오프라인 → 재연결 → 갭 채우기
    Tool: Bash (flutter test)
    Steps:
      1. User A 오프라인 상태에서 User B가 3개 메시지 전송
      2. User A 온라인 복구
      3. Assert: 3개 누락 메시지 자동 수신, seq 순서 올바름
    Expected Result: 갭 없이 메시지 동기화
    Evidence: .sisyphus/evidence/task-18-gap-fill.txt
  ```

- [ ] 19. User Profile UI

  **What to do**:
  - 프로필 화면: 아바타, 닉네임, 상태 메시지, 디바이스 목록
  - 프로필 수정 화면: 아바타 업로드, 닉네임 변경
  - 연락처 프로필 보기 화면
  - 설정 탭 내 계정 설정 (로그아웃, 디바이스 관리)

  **Recommended Agent Profile**: `visual-engineering` + `frontend-ui-ux` skill
  **Parallelization**: Wave 1.5 | Blocked By: T10,T11
  **Commit**: YES — `feat(profile-ui): user profile + settings screens`

- [ ] 20. Performance Benchmarking — k6 + Integration Tests

  **What to do**:
  - k6 WebSocket 부하 테스트: 100 동시 연결, 메시지 전송/수신 p95
  - k6 HTTP 부하 테스트: 인증 API, 대화 API, 미디어 API
  - Flutter integration test 스위트: 인증 → 대화 → 메시지 → 미디어 E2E
  - 서버 메모리/CPU 프로파일링 (100 동시 연결)
  - 벤치마크 결과 리포트: `docs/benchmarks/phase1.md`

  **Recommended Agent Profile**: `unspecified-high`
  **Parallelization**: Wave 1.6 | Blocks: T21 | Blocked By: T18
  **Commit**: YES — `test(perf): k6 benchmarks + integration test suite`

  **Acceptance Criteria**:
  - [ ] 메시지 p95 < 200ms (100 동시)
  - [ ] cold start < 2000ms (iOS + Android)
  - [ ] 서버 RSS < 512MB (100 동시)

- [ ] 21. Phase 1 Test Suite + Bug Fixes

  **What to do**:
  - T20 벤치마크 결과에서 발견된 버그 수정
  - 핵심 로직 단위 테스트 추가: 인증, 메시지 릴레이, gap-fill
  - Flutter golden test: 주요 화면 스냅샷 (대화 목록, 채팅, 프로필)
  - CI에서 전체 테스트 스위트 실행 확인

  **Recommended Agent Profile**: `unspecified-high`
  **Parallelization**: Wave 1.6 | Blocks: T22 | Blocked By: T20
  **Commit**: YES — `fix(phase1): bug fixes + test coverage`

- [ ] 22. Phase 1 Dogfooding (2 Weeks)

  **What to do**:
  - 팀 내 실제 사용 (2주간 Paw로 일상 대화)
  - 버그 리포트 수집 + 우선순위 분류
  - UX 피드백 반영 (사소한 개선 즉시 적용)
  - Phase 1 회고 + Phase 2 준비사항 정리
  - 성능 벤치마크 재측정 (dogfooding 버그 수정 후)

  **Recommended Agent Profile**: `deep`
  **Parallelization**: Wave 1.6 | Blocks: T23,T24,T25 (Phase 2 시작) | Blocked By: T21
  **Commit**: YES — `fix(dogfood): phase 1 dogfooding fixes`

### ═══════════════════════════════════════════
### PHASE 2: Agent & E2EE (Week 13-26)
### ═══════════════════════════════════════════

- [ ] 23. Rust FFI Bridge — flutter_rust_bridge + Crypto Library

  **What to do**:
  - T8 PoC 결과에 따라 선택된 crypto 라이브러리 통합
  - `flutter_rust_bridge` 설정: `paw-crypto` 크레이트 → Dart 바인딩 자동 생성
  - iOS/Android/Desktop/Web 크로스 컴파일 설정
  - 모든 Rust 코드를 단일 workspace 크레이트로 관리 (iOS 심볼 충돌 방지)
  - API: `encrypt(plaintext, session) → ciphertext`, `decrypt(ciphertext, session) → plaintext`
  - `create_account() → AccountKeys`, `create_session(their_identity_key) → Session`

  **Recommended Agent Profile**: `deep`
  **Parallelization**: Wave 2.1 | Blocks: T26,T27 | Blocked By: T8
  **Commit**: YES — `feat(ffi): flutter_rust_bridge crypto bindings`

  **QA Scenarios**:
  ```
  Scenario: Dart → Rust 암호화 round-trip
    Tool: Bash (flutter test)
    Steps:
      1. `flutter test test/crypto_ffi_test.dart`
      2. Dart에서: createSession → encrypt("hello") → decrypt → "hello"
    Expected Result: FFI 경유 암/복호화 성공
    Evidence: .sisyphus/evidence/task-23-ffi-crypto.txt
  ```

- [ ] 24. E2EE Key Management — Prekey Bundles

  **What to do**:
  - 서버 측 prekey 번들 저장: `prekey_bundles` 테이블 (identity_key, signed_prekey, one_time_prekeys)
  - `POST /api/v1/keys/upload` — prekey 번들 업로드
  - `GET /api/v1/keys/{user_id}` — 상대방 prekey 번들 조회 (X3DH 시작용)
  - One-time prekey 소진 모니터링 + 클라이언트에 보충 요청
  - 키 교체 API: signed prekey 주기적 갱신

  **Recommended Agent Profile**: `deep`
  **Parallelization**: Wave 2.1 | Blocks: T26 | Blocked By: T2,T5
  **Commit**: YES — `feat(e2ee): prekey bundle management`

- [ ] 25. Agent Gateway Scaffold — Rust WebSocket

  **What to do**:
  - Agent 전용 WebSocket 엔드포인트: `ws://host/agent/ws?token=agent_xxx`
  - Agent 인증: 토큰 기반 (Telegram Bot API처럼 단순)
  - Agent → 서버: 메시지 수신 구독, 응답 전송
  - 서버 → Agent: 컨텍스트 포함 인바운드 메시지 전달
  - NATS 도입: Agent Gateway ↔ Message Relay 사이 메시지 라우팅
  - `InboundContext` 구조: message + conversation.recent_messages + capabilities

  **Recommended Agent Profile**: `unspecified-high`
  **Parallelization**: Wave 2.1 | Blocks: T28,T29 | Blocked By: T6
  **Commit**: YES — `feat(agent): gateway scaffold with nats routing`

- [ ] 26. 1:1 E2EE — Encrypt/Decrypt + Key Exchange

  **What to do**:
  - X3DH 키 교환 구현 (또는 MLS 초기 그룹 생성)
  - Double Ratchet 세션 관리 (또는 MLS 트리 라쳇)
  - 메시지 암호화: 평문 → E2EE 암호문 → 서버 전송 (서버는 암호문만 저장)
  - 메시지 복호화: 암호문 수신 → 로컬 세션으로 복호화 → UI 표시
  - 세션 상태 로컬 저장 (SQLCipher에 암호화)
  - Phase 1 기존 비암호화 대화: "이 대화에 E2EE를 활성화하시겠습니까?" 프롬프트

  **Recommended Agent Profile**: `deep`
  **Parallelization**: Wave 2.2 | Blocks: T30,T37,T42,T43,T44 | Blocked By: T23,T24
  **Commit**: YES — `feat(e2ee): 1:1 end-to-end encryption`

  **Acceptance Criteria**:
  - [ ] `psql -c "SELECT content FROM messages WHERE id='test'" → 암호문만 존재`
  - [ ] 두 클라이언트 간 메시지 암/복호화 성공
  - [ ] 세션 상태 로컬 영속 (앱 재시작 후 복호화 성공)

- [ ] 27. E2EE Flutter UI — Key Verification + Consent

  **What to do**:
  - E2EE 대화 표시: 🔒 아이콘 + "종단간 암호화됨" 라벨
  - 키 검증 화면: QR 코드 비교 또는 안전 번호 비교 (Signal 스타일)
  - Agent 동의 UI: "🤖 Agent가 이 대화를 읽고 있습니다" 배너
  - Agent 초대/제거: 대화 설정에서 Agent 관리
  - E2EE 활성화 프롬프트 (기존 비암호화 대화용)

  **Recommended Agent Profile**: `visual-engineering` + `frontend-ui-ux` skill
  **Parallelization**: Wave 2.2 | Blocked By: T23,T11
  **Commit**: YES — `feat(e2ee-ui): key verification + agent consent indicators`

- [ ] 28. Agent Streaming Protocol — content_delta + tool indicators

  **What to do**:
  - `stream_start` → `content_delta` → `tool_start/end` → `stream_end` 프레임 구현
  - Agent가 토큰별로 `content_delta` 전송 → 서버가 사용자에게 릴레이
  - Tool 인디케이터: `{"type":"tool_start","tool":"web_search","label":"검색 중..."}`
  - 서버 측 강제 제한: `max_stream_duration: 300s`, `max_stream_bytes: 1MB`
  - 스트리밍 메타데이터: `stream_end`에 토큰 수, 소요 시간 포함
  - 네트워크 끊김 시: `stream_end` 없이 연결 끊기면 "응답 중단됨" 표시

  **Recommended Agent Profile**: `deep`
  **Parallelization**: Wave 2.2 | Blocks: T32,T33,T34,T35,T38 | Blocked By: T25
  **Commit**: YES — `feat(agent): streaming protocol with content_delta + tool indicators`

  **Acceptance Criteria**:
  - [ ] Agent 스트리밍 TTFT p95 < 1000ms
  - [ ] 300초 초과 스트리밍 시 서버가 강제 종료

- [ ] 29. Agent Auth + Registration API

  **What to do**:
  - `POST /api/v1/agents/register` — Agent 등록 (이름, 설명, 아바타)
  - Agent 토큰 발급: `msg_xxx` 형식 (Telegram Bot API처럼)
  - `GET /api/v1/agents/{id}` — Agent 프로필 조회
  - Agent 토큰 갱신/폐기 API
  - KYC 없는 등록 (Discord와 차별화)

  **Recommended Agent Profile**: `unspecified-high`
  **Parallelization**: Wave 2.2 | Blocks: T32,T34 | Blocked By: T5,T25
  **Commit**: YES — `feat(agent): token-based registration + auth api`

- [ ] 30. Group Chat Backend — Up to 100 + Group E2EE

  **What to do**:
  - 그룹 생성/관리 API (최대 100명)
  - Sender Keys 기반 그룹 E2EE (또는 MLS 그룹)
  - 그룹 멤버 추가/제거 → 키 재배포
  - 그룹 관리자 권한 (멤버 추가/제거, 그룹 설정)
  - 그룹 메시지 라우팅: 모든 멤버에게 전달

  **Recommended Agent Profile**: `deep`
  **Parallelization**: Wave 2.3 | Blocks: T31,T41 | Blocked By: T26
  **Commit**: YES — `feat(group): group chat with e2ee (sender keys)`

  **Acceptance Criteria**:
  - [ ] 5명 그룹 E2EE: 모든 참여자 메시지 복호화 성공
  - [ ] 멤버 제거 후 새 메시지 복호화 불가 확인

- [ ] 31. Group Chat UI

  **What to do**:
  - 그룹 생성 화면 (연락처에서 멤버 선택)
  - 그룹 대화 화면: 멤버별 아바타 + 닉네임 표시
  - 그룹 설정: 이름, 아바타, 멤버 목록, 관리자 설정
  - 그룹 내 Agent 초대/제거 UI
  - 멤버 변경 시스템 메시지 표시

  **Recommended Agent Profile**: `visual-engineering` + `frontend-ui-ux` skill
  **Parallelization**: Wave 2.3 | Blocked By: T11,T30
  **Commit**: YES — `feat(group-ui): group chat interface`

- [ ] 32. Python Agent SDK

  **What to do**:
  - `paw-sdk-python` 패키지: PyPI 배포
  - WebSocket 클라이언트 (`websockets` 라이브러리)
  - 이벤트 드리븐 API: `@agent.on_message`, `@agent.on_button_click`
  - 스트리밍 지원: `await ctx.stream(token)` — 토큰별 전송
  - 리치 블록 빌더: `ctx.card(title, fields)`, `ctx.buttons([...])`
  - 대화 컨텍스트: `ctx.conversation.recent` — 최근 메시지 자동 제공
  - 기획서의 예시 코드와 동일한 API (`openclaw-plan.md:483-541`)
  - README + 퀵스타트 가이드

  **Recommended Agent Profile**: `unspecified-high`
  **Parallelization**: Wave 2.3 | Blocks: T45 | Blocked By: T28,T29
  **Commit**: YES — `feat(sdk): python agent sdk with streaming + rich blocks`

  **Acceptance Criteria**:
  - [ ] 기획서의 echo-bot 예시 코드 실행 → 정상 동작
  - [ ] 기획서의 claude-bot 스트리밍 예시 → TTFT < 1s
  - [ ] `pip install paw-sdk` 성공

- [ ] 33. Rich Message Blocks — Card + Button

  **What to do**:
  - 서버: `blocks` 필드 처리 (`type: "card"`, `type: "action_buttons"`)
  - Flutter: 카드 렌더링 (색상, 제목, 필드, 이미지, 푸터)
  - Flutter: 버튼 렌더링 (라벨, 액션 ID → 서버로 전달)
  - 기획서 메시지 포맷 참고 (`openclaw-plan.md:406-443`)
  - Agent가 보낸 블록만 렌더링 (일반 사용자는 마크다운만)

  **Must NOT do**: form, date-picker, dropdown 블록 (Phase 3+)

  **Recommended Agent Profile**: `visual-engineering` + `frontend-ui-ux` skill
  **Parallelization**: Wave 2.3 | Blocked By: T9,T11,T28
  **Commit**: YES — `feat(blocks): card + button rich message blocks`

- [ ] 34. OpenClaw Channel Adapter — TypeScript

  **What to do**:
  - OpenClaw 패턴 따르기: `monitor.ts`, `send.ts`, `accounts.ts`, `components.ts`, `chunk.ts`
  - `monitor.ts`: Paw WebSocket 연결 → 인바운드 메시지 수신 → `InboundContext` 변환
  - `send.ts`: OutboundMessage → Paw 스트리밍 프로토콜로 변환 (content_delta 지원)
  - `accounts.ts`: 토큰 인증, 건강 확인
  - `components.ts`: 리치 블록 매핑 (card → Paw card, button → Paw button)
  - `chunk.ts`: 사실상 불필요 (Paw는 글자 제한 없음) — passthrough
  - MIT 라이선스, npm 패키지 (`@paw/openclaw-adapter`)
  - OpenClaw 커뮤니티 기여용 README + 설정 가이드

  **Recommended Agent Profile**: `unspecified-high`
  **Parallelization**: Wave 2.4 | Blocks: T46 | Blocked By: T28,T29
  **Commit**: YES — `feat(openclaw): channel adapter for openclaw gateway`

  **Acceptance Criteria**:
  - [ ] OpenClaw Gateway에서 Paw 어댑터 로드 성공
  - [ ] OpenClaw → Paw 메시지 전송 → 응답 수신 (스트리밍 포함)
  - [ ] `npm test --prefix paw-adapter-openclaw` → 모든 테스트 통과

- [ ] 35. Streaming UI — Token-by-token + Tool Indicators

  **What to do**:
  - `content_delta` 수신 → 글자 단위 실시간 표시 (ChatGPT/Claude처럼)
  - Tool 인디케이터: "🔍 검색 중..." → "✓ 검색 완료" 접이식 블록
  - 스트리밍 커서: `█` 깜빡이는 커서 표시
  - 스트리밍 중 스크롤 자동 따라가기
  - 스트리밍 완료 후 마크다운 최종 렌더링
  - 기획서 UI 참고 (`openclaw-plan.md:557-584`)

  **Recommended Agent Profile**: `visual-engineering` + `frontend-ui-ux` skill
  **Parallelization**: Wave 2.4 | Blocked By: T13,T28
  **Commit**: YES — `feat(stream-ui): token streaming with tool indicators`

- [ ] 36. Thread Support — Backend + UI

  **What to do**:
  - `threads` 테이블: parent_message_id, conversation_id
  - 메시지에 thread 생성 → thread 내 메시지 별도 목록
  - Agent가 thread 단위로 대화 (컨텍스트 격리)
  - Flutter UI: 메시지 스와이프 → thread 열기, thread 뷰

  **Recommended Agent Profile**: `unspecified-high`
  **Parallelization**: Wave 2.4 | Blocked By: T9,T11
  **Commit**: YES — `feat(thread): threaded conversations`

- [ ] 37. Agent E2EE — Key Sharing + Consent + Revocation

  **What to do**:
  - Agent 초대 시: 사용자가 Megolm/MLS 세션키를 Agent에게 공유
  - Agent 디바이스 키 등록 + 검증
  - 세션키 공유: 현재 ratchet index 이후만 (과거 메시지 접근 불가)
  - Agent 제거 시: 새 세션 생성, Agent에게 새 키 미공유
  - UI: "🤖 Agent가 이 대화를 읽고 있습니다" 지속 배너
  - 감사 로그: Agent 초대/제거 이벤트 기록

  **Recommended Agent Profile**: `deep`
  **Parallelization**: Wave 2.4 | Blocks: T38 | Blocked By: T26,T29
  **Commit**: YES — `feat(agent-e2ee): agent key sharing with consent + revocation`

- [ ] 38. Agent Streaming Limits + Moderation

  **What to do**:
  - 서버 측 강제: `max_stream_duration: 300s`, `max_stream_bytes: 1MB`, `max_tokens: configurable`
  - 악의적 Agent 차단: 과도한 요청 시 레이트 리밋 + 일시 중지
  - 기본 모더레이션: 스팸 필터 (단순 키워드), 신고 API
  - Agent 토큰 폐기 API (관리자용)

  **Recommended Agent Profile**: `unspecified-high`
  **Parallelization**: Wave 2.5 | Blocked By: T28,T37
  **Commit**: YES — `feat(moderation): agent limits + basic spam filter`

- [ ] 39. E2EE Security Audit Preparation + External Review

  **What to do**:
  - E2EE 코드 문서화: 프로토콜 결정, 키 관리 흐름, 위협 모델
  - 보안 감사 준비: 코드 정리, 테스트 커버리지 확인
  - 외부 보안 감사 의뢰 (Cure53, Trail of Bits 등 리스트업)
  - 자체 보안 테스트: 알려진 공격 벡터 확인 (B=0, 키 재사용 등)
  - 감사 리포트 템플릿 준비

  **Recommended Agent Profile**: `deep`
  **Parallelization**: Wave 2.5 | Blocked By: T26,T30,T37
  **Commit**: YES — `docs(security): e2ee audit preparation`

- [ ] 40. Phase 2 Dogfooding + Benchmarks (2 Weeks)

  **What to do**:
  - E2EE 실사용 테스트 (팀 내 2주간 E2EE 활성화 대화)
  - Agent 스트리밍 실사용 테스트 (실제 Claude/GPT Agent 연결)
  - 그룹 채팅 실사용 테스트
  - 성능 벤치마크: 메시지 p95, 스트리밍 TTFT, E2EE 오버헤드
  - Phase 2 버그 수정 + Phase 3 준비

  **Recommended Agent Profile**: `deep`
  **Parallelization**: Wave 2.5 | Blocks: T41-T44 (Phase 3) | Blocked By: T38,T39
  **Commit**: YES — `fix(dogfood): phase 2 fixes + benchmarks`

### ═══════════════════════════════════════════
### PHASE 3: Scale & Polish (Week 27-38)
### ═══════════════════════════════════════════

- [ ] 41. Channels — Broadcast (1→Many)

  **What to do**:
  - 채널 생성/관리 API (1→다 브로드캐스트)
  - 채널 소유자만 메시지 전송, 구독자는 읽기만
  - 대규모 채널 (100명+): 암호화 수준 선택 (공개/비공개/보안)
  - 채널 검색 + 구독 API
  - Agent가 채널에서 알림 발송 가능

  **Recommended Agent Profile**: `deep`
  **Parallelization**: Wave 3.1 | Blocked By: T30
  **Commit**: YES — `feat(channels): broadcast channels`

- [ ] 42. Multi-device Sync — Server Seq-based Gap-fill

  **What to do**:
  - 다중 디바이스 동시 접속 지원
  - 각 디바이스: 독립 WebSocket 연결 + 독립 `last_seq` 추적
  - 새 디바이스 추가: 서버에서 전체 대화 목록 + 최근 N개 메시지 전달
  - E2EE 키 공유: 기존 디바이스에서 새 디바이스로 Olm/MLS 세션키 전달
  - 디바이스 간 읽음 상태 동기화

  **Must NOT do**: CRDT(Yjs) 사용 금지 — 서버 seq 기반만

  **Recommended Agent Profile**: `deep`
  **Parallelization**: Wave 3.1 | Blocked By: T18,T26
  **Commit**: YES — `feat(sync): multi-device seq-based sync`

- [ ] 43. Push Notifications + E2EE

  **What to do**:
  - FCM (Android) + APNs (iOS) 설정
  - E2EE 호환: 무음 데이터 푸시 → 앱 내 복호화 → 로컬 알림
  - iOS: Notification Service Extension에서 Rust C-ABI 복호화
  - Android: FCM data-only 메시지 → WorkManager에서 복호화
  - 알림 설정: 대화별 음소거, 전체 무음 모드

  **Recommended Agent Profile**: `unspecified-high`
  **Parallelization**: Wave 3.1 | Blocked By: T26
  **Commit**: YES — `feat(push): e2ee-compatible push notifications`

- [ ] 44. Cloud Backup — Encrypted, Optional

  **What to do**:
  - 암호화된 메시지 백업: 사용자 키로 AES-256 암호화 → S3 업로드
  - 백업 복원: 키 입력 → 다운로드 → 복호화 → 로컬 DB 복원
  - 자동 백업 옵션 (일간/주간)
  - 백업 삭제 API

  **Recommended Agent Profile**: `unspecified-high`
  **Parallelization**: Wave 3.1 | Blocked By: T26,T14
  **Commit**: YES — `feat(backup): encrypted cloud backup`

- [ ] 45. Agent Marketplace — Registry + Discovery

  **What to do**:
  - Agent 마켓플레이스 API: 검색, 카테고리, 인기 순, 평점
  - Agent manifest 스키마: name, version, description, capabilities, permissions, config
  - Flutter UI: 마켓플레이스 탭 → Agent 검색 → 설치 → 대화에 추가
  - Agent 설치/제거 플로우

  **Recommended Agent Profile**: `unspecified-high`
  **Parallelization**: Wave 3.2 | Blocked By: T29,T32
  **Commit**: YES — `feat(marketplace): agent registry + discovery ui`

- [ ] 46. TypeScript Agent SDK

  **What to do**:
  - `@paw/sdk` npm 패키지
  - Python SDK와 동일한 API 패턴 (이벤트 드리븐, 스트리밍)
  - TypeScript 타입 정의
  - OpenClaw이 TypeScript 기반이므로 생태계 호환 중요
  - README + 퀵스타트

  **Recommended Agent Profile**: `unspecified-high`
  **Parallelization**: Wave 3.2 | Blocked By: T32,T34
  **Commit**: YES — `feat(sdk-ts): typescript agent sdk`

- [ ] 47. Full-text Local Search — FTS5

  **What to do**:
  - SQLCipher FTS5 확장 활용
  - 로컬 메시지 전문 검색 (한국어, 영어, 일본어)
  - 검색 결과 하이라이팅 + 대화 컨텍스트 표시
  - Flutter 검색 UI: 상단 검색바 → 결과 목록 → 탭 시 해당 대화 이동

  **Recommended Agent Profile**: `unspecified-high`
  **Parallelization**: Wave 3.2 | Blocked By: T14
  **Commit**: YES — `feat(search): full-text local search with fts5`

- [ ] 48. Desktop Platform QA

  **What to do**:
  - macOS, Windows, Linux 빌드 + 실행 테스트
  - 플랫폼별 네이티브 통합: 시스템 트레이, 알림, 키보드 단축키
  - 레이아웃 반응형 확인 (데스크톱 넓은 화면 대응)
  - 데스크톱 전용 버그 수정

  **Recommended Agent Profile**: `unspecified-high`
  **Parallelization**: Wave 3.2 | Blocked By: T22
  **Commit**: YES — `fix(desktop): platform-specific fixes + responsive layout`

- [ ] 49. Web Platform QA + Optimization

  **What to do**:
  - Web 빌드 최적화 (번들 사이즈 축소, 트리 셰이킹)
  - Web 전용 이슈 수정 (WebSocket 호환, 로컬 스토리지)
  - PWA 설정 (서비스 워커, 오프라인 지원)
  - 브라우저 호환성 테스트 (Chrome, Firefox, Safari)

  **Recommended Agent Profile**: `unspecified-high`
  **Parallelization**: Wave 3.3 | Blocked By: T22
  **Commit**: YES — `fix(web): web optimization + pwa support`

- [ ] 50. Performance Optimization + CDN

  **What to do**:
  - 미디어 CDN 설정 (Cloudflare R2 또는 동등)
  - 서버 최적화: 커넥션 풀링, 쿼리 최적화
  - 클라이언트 최적화: 이미지 캐싱, 가상 스크롤
  - 최종 벤치마크: p95 <200ms (1000 동시)

  **Recommended Agent Profile**: `deep`
  **Parallelization**: Wave 3.3 | Blocks: T52 | Blocked By: T43
  **Commit**: YES — `perf(all): cdn + server/client optimization`

- [ ] 51. Moderation Tools — Spam Filter + Reports

  **What to do**:
  - 향상된 스팸 필터 (패턴 매칭 + 레이트 리밋)
  - 신고 시스템: 메시지/사용자/Agent 신고 → 관리자 대시보드
  - 사용자 차단/뮤트 기능
  - 관리자 API: 사용자/Agent 일시중지/영구차단

  **Recommended Agent Profile**: `unspecified-high`
  **Parallelization**: Wave 3.3 | Blocked By: T38
  **Commit**: YES — `feat(moderation): spam filter + report system`

- [ ] 52. Final Performance Benchmarking

  **What to do**:
  - k6 전체 부하 테스트: 1000 동시 연결, 메시지/스트리밍/미디어
  - E2EE 오버헤드 측정 (암호화/복호화 지연)
  - 모바일 배터리/메모리 프로파일링
  - KPI 달성 확인: p95 <200ms, TTFT <1s, cold start <2s
  - 최종 벤치마크 리포트: `docs/benchmarks/final.md`

  **Recommended Agent Profile**: `unspecified-high`
  **Parallelization**: Wave 3.4 | Blocks: T53 | Blocked By: T50
  **Commit**: YES — `docs(perf): final benchmark report`

- [ ] 53. Phase 3 Dogfooding (2 Weeks)

  **What to do**:
  - 전체 기능 실사용 (멀티 디바이스, 채널, 마켓플레이스)
  - 외부 베타 테스터 초대 (10-20명)
  - 최종 버그 수정 + UX 폴리시
  - 출시 준비: 앱스토어 메타데이터, 스크린샷, 설명

  **Recommended Agent Profile**: `deep`
  **Parallelization**: Wave 3.4 | Blocks: T54 | Blocked By: T52
  **Commit**: YES — `fix(dogfood): final round of fixes`

- [ ] 54. Release Preparation + Documentation

  **What to do**:
  - API 문서: Swagger/OpenAPI spec
  - Agent SDK 문서: 퀵스타트, API 레퍼런스, 예제 코드
  - 프로토콜 스펙 문서 최종화
  - 오픈소스 라이선스 정리: 클라이언트 Apache 2.0, 서버 AGPL, SDK MIT
  - README, CONTRIBUTING, CHANGELOG
  - 배포 자동화: Docker 이미지 빌드 + Fly.io 배포 스크립트

  **Recommended Agent Profile**: `writing`
  **Parallelization**: Wave 3.4 | Blocked By: T53
  **Commit**: YES — `docs(release): api docs + sdk guides + deployment scripts`

---

## Final Verification Wave (MANDATORY)

> 4 review agents run in PARALLEL. ALL must APPROVE.

- [ ] F1. **Plan Compliance Audit** — `oracle`
  Read plan end-to-end. For each "Must Have": verify implementation exists. For each "Must NOT Have": search codebase for forbidden patterns. Check evidence files in `.sisyphus/evidence/`. Compare deliverables against plan.
  Output: `Must Have [N/N] | Must NOT Have [N/N] | Tasks [N/N] | VERDICT`

- [ ] F2. **Code Quality Review** — `unspecified-high`
  Run `cargo clippy`, `flutter analyze`, `cargo test`, `flutter test`. Review all changed files for: `unwrap()` without comment, empty catches, `print!` in prod, commented-out code. Check AI slop: excessive comments, over-abstraction.
  Output: `Build [PASS/FAIL] | Lint [PASS/FAIL] | Tests [N pass/N fail] | VERDICT`

- [ ] F3. **Real Manual QA** — `unspecified-high` (+ `playwright` skill for web)
  Execute EVERY QA scenario from EVERY task. Test cross-task integration. Test edge cases: empty state, invalid input, network disconnect. Save to `.sisyphus/evidence/final-qa/`.
  Output: `Scenarios [N/N pass] | Integration [N/N] | Edge Cases [N tested] | VERDICT`

- [ ] F4. **Scope Fidelity Check** — `deep`
  For each task: read spec, read actual code. Verify 1:1 match. Check "Must NOT do" compliance. Detect cross-task contamination. Flag unaccounted changes.
  Output: `Tasks [N/N compliant] | Contamination [CLEAN/N issues] | VERDICT`

---

## Commit Strategy

Phase 1:
- `feat(server): init monorepo + CI/CD` — T1
- `feat(db): postgresql schema + migrations` — T2
- `feat(proto): websocket protocol types v1` — T3
- `feat(client): flutter scaffold + theme` — T4
- `feat(auth): otp + ed25519 device key service` — T5
- `feat(ws): websocket server + connection manager` — T6
- `feat(auth-ui): login/register flows` — T7
- `docs(crypto): e2ee protocol evaluation report` — T8
- `feat(msg): message relay service` — T9
- `feat(api): user profile + contacts` — T10
- `feat(chat-ui): conversation list + message bubbles` — T11
- `feat(media): s3 compatible upload service` — T12
- `feat(ws-client): realtime sync + reconnection` — T13
- `feat(storage): drift + sqlcipher local db` — T14
- `feat(md): markdown rendering + code highlight` — T15
- `feat(msg): read receipts + typing indicators` — T16
- `feat(media-ui): send/receive + preview` — T17
- `feat(sync): offline gap-fill + reconnection` — T18
- `feat(profile-ui): user profile screen` — T19
- `test(perf): k6 benchmarks + integration tests` — T20
- `fix(phase1): bug fixes from dogfooding` — T21

Phase 2:
- `feat(ffi): flutter_rust_bridge + crypto bindings` — T23
- `feat(e2ee): key management + prekey bundles` — T24
- `feat(agent): gateway scaffold` — T25
- `feat(e2ee): 1:1 encrypt/decrypt + key exchange` — T26
- `feat(e2ee-ui): key verification + consent` — T27
- `feat(agent): streaming protocol` — T28
- `feat(agent): auth + registration api` — T29
- `feat(group): group chat + group e2ee` — T30
- `feat(group-ui): group chat interface` — T31
- `feat(sdk): python agent sdk` — T32
- `feat(blocks): rich card + button blocks` — T33
- `feat(openclaw): channel adapter` — T34
- `feat(stream-ui): token streaming + tool indicators` — T35
- `feat(thread): thread support` — T36
- `feat(agent-e2ee): agent key sharing + revocation` — T37
- `feat(moderation): agent limits + tools` — T38

Phase 3:
- `feat(channels): broadcast channels` — T41
- `feat(sync): multi-device seq-based sync` — T42
- `feat(push): notifications + e2ee` — T43
- `feat(backup): encrypted cloud backup` — T44
- `feat(marketplace): agent registry + discovery` — T45
- `feat(sdk-ts): typescript agent sdk` — T46
- `feat(search): full-text local search` — T47
- `perf(all): optimization + cdn` — T50
- `feat(moderation): spam filter + reports` — T51

---

## Success Criteria

### Performance KPIs
```bash
# 메시지 전송 p95
k6 run --vus 100 --duration 60s test/load/message-send.js
# Expected: p95 < 200ms

# AI 스트리밍 TTFT
k6 run test/load/agent-streaming.js
# Expected: p95 TTFT < 1000ms

# 클라이언트 cold start
flutter drive --target=test_driver/cold_start_test.dart
# Expected: 대화 목록 표시 < 2000ms

# 서버 메모리 (1000 동시 연결)
# Expected: RSS < 512MB
```

### Security Checklist
- [ ] 서버 DB에 평문 메시지 부재: `psql -c "SELECT count(*) FROM messages WHERE content NOT LIKE 'enc:%'" → 0`
- [ ] E2EE 키 교환 성공: `cargo test test_x3dh_key_exchange → PASS`
- [ ] Agent 세션키 폐기: Agent 제거 후 새 메시지 복호화 불가 확인
- [ ] 외부 보안 감사 완료 (Phase 2 완료 후)

### Final Checklist
- [ ] All "Must Have" present
- [ ] All "Must NOT Have" absent
- [ ] All tests pass (`cargo test` + `flutter test`)
- [ ] Performance benchmarks met
- [ ] OpenClaw adapter integration verified
- [ ] 3 phases dogfooding completed
