# Draft: AI-Native Lightweight Messenger

## 연구 결과 종합 (5개 병렬 에이전트)

### 1. OpenClaw 아키텍처 분석
- **23개 채널** 지원, TypeScript 기반, WebSocket Gateway (포트 18789)
- **Channel Adapter 패턴**: `monitor.ts` (인바운드) + `send.ts` (아웃바운드) + `accounts.ts` + `components.ts` + `chunk.ts`
- **InboundContext**: channel, sender, conversation, text, attachments, metadata
- **Capabilities 선언**: text, media, reactions, editing, typing, streaming/draft
- **보안**: DM Pairing, allowlist, 토큰/비밀번호 인증, loopback 바인딩

### 2. Rust/Axum 서버 패턴
- **vodozemac** (336 stars) - Matrix의 E2EE Rust 구현체 (Olm + Megolm), Least Authority 감사 완료
- **커넥션 관리**: `tokio::sync::broadcast` + `DashMap<ConnectionId, ConnectionHandle>`
- **AI 스트리밍**: Fireside Chat (140 stars) 참고 - `tokio::sync::mpsc` 채널로 토큰별 스트리밍
- **NATS 연동**: `async-nats` 크레이트, JetStream으로 영구 메시지
- **인증**: JWT → WebSocket 업그레이드 패턴
- **프로덕션**: heartbeat (30-60s ping), bounded channels (backpressure), graceful shutdown

### 3. Flutter 클라이언트 패턴
- **UI**: Flyer Chat UI (1.6k likes) + flutter_gen_ai_chat_ui (AI 스트리밍)
- **로컬 DB**: Drift (2.34k likes, 반응형) + SQLCipher (암호화)
- **상태 관리**: Riverpod (3.93k likes)
- **WebSocket**: web_socket_channel (1.62k likes)
- **암호화**: cryptography 패키지 (X25519 + AES-256-GCM + Ed25519)
- **DI**: GetIt (4.67k likes)
- **마크다운**: flutter_markdown (1.47k likes) + syntax_highlight
- **⚠️ Dart용 Signal Protocol 포트 없음** → FFI로 libsignal 연동 또는 간소화 E2EE 필요

### 4. NATS/CRDT/Agent SDK 패턴
- **NATS**: JetStream으로 영구 메시지, subject 계층: `chat.rooms.{roomId}.messages`
- **CRDT**: Yjs (21.4k stars) 추천 - Y.Array 메시지 로그, y-websocket 동기화, y-indexeddb 로컬
- **SDK**: Discord.js 패턴 (모듈형, 이벤트 드리븐) + Slack Bolt 미들웨어
- **Python SDK**: `websockets` 라이브러리 + async/await
- **Agent 마켓플레이스**: manifest 기반 레지스트리 (이름, 버전, 권한, 설정 스키마)

### 5. E2EE + Agent 참여 아키텍처
- **추천**: vodozemac (Olm 1:1 + Megolm 그룹) - pure Rust, 감사 완료
- **대안**: libsignal (공식 Signal 구현) - X3DH + Double Ratchet
- **Agent 접근 패턴** (Matrix 참고): Agent = 일반 사용자 취급, 디바이스 키로 참여
  - 사용자가 Agent 초대 → X3DH 키 합의 → Megolm 세션키 Olm으로 공유
  - Agent 제거 시: 새 Megolm 세션 생성, 새 키를 Agent에게 공유 안 함
- **보안**: 주기적 세션 순환 (100 메시지 or 24시간), UI에 "Agent가 읽고 있습니다" 표시
- **⚠️ 보안 감사 필수**: 세션키 공유 메커니즘 프로덕션 전 암호학자 검토 필요

---

## Requirements (확인됨 - 기획서 기반)
- OpenClaw 연동 최적화 메신저 신규 개발
- 클라이언트: Flutter (iOS/Android/Web/Desktop)
- 서버: Rust + Axum
- 메시지 큐: NATS
- E2EE: Signal Protocol 기반 (1:1 Double Ratchet, 그룹 Sender Keys)
- 로컬 저장: SQLCipher
- 동기화: CRDT (Yjs 기반)
- 인증: SRP + Ed25519
- Agent SDK: Python (우선), TypeScript, Go
- MVP 로드맵: Phase 1 (10주) → Phase 2 (8주) → Phase 3 (10주), 총 28주

## Technical Decisions (모두 확인됨)
- **계획 범위**: 전체 3 Phase (28주) — 하나의 계획에 모두 포함
- **프로젝트명**: **Paw** (paw-server, paw-client, @paw/sdk)
- **팀 규모**: 소규모 (2-3명) → 병렬 Wave 3-4개 동시
- **E2EE 전략**: Phase 1에서 E2EE 생략 (TLS만), Phase 2에서 vodozemac FFI로 추가
- **레포 구조**: 모노레포 (server/ + client/ + sdk/ + adapter/)
- **Flutter 타겟**: 모든 플랫폼 동시 (iOS/Android/Web/Desktop)
- **테스트 전략**: Tests-after (구현 후 테스트), 핵심 로직 우선
- **배포**: Docker + Fly.io (기획서 기반 — 확정)
- **CRDT**: Phase 3에서 추가 (기획서 기반)

## Open Questions
- (모두 해결됨)

## Scope Boundaries
- INCLUDE: 전체 3 Phase (28주) — Core Messaging + Agent/Groups + Scale/Polish
- EXCLUDE: 자체 AI 모델 훈련, 결제 시스템 상세 구현, 법적 규정 준수(GDPR 등) 상세
