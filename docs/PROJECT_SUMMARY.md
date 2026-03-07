# Paw Messenger — 프로젝트 전체 작업 요약

> 작성일: 2026-03-07  
> 저장소: `/Users/bclaw/workspace/paw/` (git worktree, main branch)  
> 계획 파일: `/Users/bclaw/workspace/zxcv/.sisyphus/plans/paw-messenger.md`

---

## 1. 프로젝트 개요

**Paw**는 OpenClaw 통합에 최적화된 AI-native 경량 메신저입니다.

| 항목 | 내용 |
|------|------|
| 기간 | 38주, 3 Phase |
| 서버 | Rust / Axum |
| 클라이언트 | Flutter (iOS, Android, Web, Desktop) |
| Agent SDK | Python, TypeScript |
| 채널 어댑터 | OpenClaw (TypeScript) |
| 구조 | Cargo workspace 모노레포 |

---

## 2. 기술 스택

### 서버 (paw-server)
- **언어**: Rust (stable)
- **웹 프레임워크**: Axum
- **DB**: PostgreSQL + SQLx + pg_notify
- **실시간**: tokio broadcast (Phase 1), NATS JetStream (Phase 2+)
- **인증**: OTP + Ed25519 device keys (Signal 모델)
- **암호화**: openmls (MIT), X25519-dalek, AES-GCM

### 클라이언트 (paw-client)
- **언어**: Dart 3.11.1
- **프레임워크**: Flutter 3.41.4 stable
- **상태관리**: Riverpod
- **라우팅**: go_router
- **로컬 DB**: Drift (SQLite) + FTS5 전문검색
- **암호화**: flutter_rust_bridge v2 (paw-ffi 바인딩)
- **보안 저장소**: flutter_secure_storage

### 암호화 크레이트 (paw-crypto, paw-ffi)
- openmls 0.7.4 (MLS 프로토콜)
- x25519-dalek (ECDH 키 교환)
- AES-GCM (대칭 암호화)
- flutter_rust_bridge 2.9.0 (Dart ↔ Rust FFI)

### Agent SDK
- **Python**: `paw-agent-sdk` — PawAgent 클래스, 스트리밍 지원
- **TypeScript**: `@paw/sdk` v0.1.0 — agent.ts, streaming.ts

### OpenClaw 어댑터 (adapters/openclaw-adapter)
- TypeScript 기반
- 23개 채널 지원
- E2EE 브릿지 포함

---

## 3. 아키텍처 핵심 결정사항

| 결정 | 이유 |
|------|------|
| ❌ NATS (Phase 1) → ✅ PostgreSQL + pg_notify | NATS fan-out이 100 구독자에서 18배 성능 저하 |
| ❌ SRP 인증 → ✅ OTP + Ed25519 (Signal 모델) | 보안 강화, Signal 프로토콜 준수 |
| ❌ flutter_vodozemac (AGPL) → ✅ openmls (MIT) | 라이선스 충돌 방지 |
| ❌ CRDT/Yjs (Phase 1/2) → ✅ 서버 seq 기반 gap-fill | Phase 3 이전 복잡도 최소화 |
| ❌ LaTeX/Mermaid (Phase 1) → ✅ CommonMark + 코드 하이라이팅 | Phase 1 범위 준수 |
| ✅ 모든 WebSocket 메시지에 `"v": 1` 필드 | 프로토콜 버전 관리 |
| ✅ 단일 Cargo workspace | 크레이트 간 의존성 관리 |

---

## 4. 데이터베이스 마이그레이션

타임스탬프 기반 마이그레이션 (`20260101000001` ~ `20260101000019`):

| 번호 | 내용 |
|------|------|
| 000001 | users, devices, conversations, messages 기본 스키마 |
| 000002 | prekey bundles (E2EE) |
| 000003 | agents, conversation_agents |
| 000004 | media_attachments |
| 000005 | read_receipts |
| 000006 | group 관련 컬럼 추가 |
| 000007 | agent marketplace (published_agents) |
| 000008 | channels (broadcast) |
| 000009 | device_sync_state |
| 000010 | push_tokens |
| 000011 | backup_metadata |
| 000012 | installed_agents |
| 000013 | fts5 전문검색 |
| 000014 | spam_reports, blocked_patterns |
| 000015~019 | 성능 최적화 인덱스, CDN 설정 등 |

---

## 5. WebSocket 프로토콜 (paw-proto)

모든 메시지는 `"v": 1` 필드 필수.

### ClientMessage
- `Connect` — 토큰 인증
- `MessageSend` — 메시지 전송 (blocks 필드 포함)
- `TypingStart` / `TypingStop`
- `MessageAck` — 읽음 확인
- `Sync` — 갭 복구
- `DeviceSync` — 멀티디바이스 동기화

### ServerMessage
- `HelloOk` / `HelloError`
- `MessageReceived` (blocks 필드 포함)
- `DeviceSyncResponse`
- `TypingStart` / `TypingStop`
- `PresenceUpdate`
- `StreamStart` / `ContentDelta` / `ToolStart` / `ToolEnd` / `StreamEnd` (Agent 스트리밍)

### Agent Gateway
- `InboundContext` — 메시지 수신 시 에이전트에 전달 (최근 10개 메시지 포함)
- `AgentResponseMsg` — 에이전트 응답
- `AgentStreamMsg` — 스트리밍 프레임 열거형

---

## 6. Phase별 완료 내역

### Phase 1 (T1–T22) ✅ 완료

| Wave | 커밋 | 내용 |
|------|------|------|
| 1.1 | `c52d7895` | Cargo workspace, DB 스키마, 프로토콜 타입, Flutter 스캐폴드 |
| 1.2 | `3c73dc5e` | 인증 서비스, WebSocket 서버, 인증 UI, E2EE PoC |
| 1.3 | `26f140ca` | 메시지 릴레이, 사용자 프로필 API, 채팅 UI, 미디어 업로드 |
| 1.4 | `28d668a6` | WS 클라이언트, 로컬 DB, 마크다운 렌더링, 읽음 확인 |
| 1.5 | `58aafb97` | 미디어 UI, 오프라인 갭 복구, 사용자 프로필 UI |
| T20 | `ba385dbe` | k6 벤치마크 + 통합 테스트 |
| T21 | `f0da7009` | 버그 수정 + 테스트 커버리지 (23개 테스트) |
| T22 | `3450b764` | 도그푸딩 인프라 문서 |

### Phase 2 (T23–T40) ✅ 완료

| 커밋 | 내용 |
|------|------|
| `e180425c` | T24: prekey bundle 관리 |
| `5cc46521` | T23a: paw-crypto mls.rs 수정 |
| `120b11d2` | T23b: paw-ffi 크레이트 (X25519+AES-GCM, 3개 테스트) |
| `86637275` | T25: Agent 게이트웨이 스캐폴드 (NATS, agent WS, docker-compose, 28개 테스트) |
| `1819045c` | T23c: flutter_rust_bridge v2 Dart 바인딩 + CI 업데이트 |
| `223017df` | T27: E2EE UI (e2ee_banner, agent_consent_banner, key_verification_screen) |
| `7acdaffe` | T29: Agent 인증 API |
| `00ee8dfb` | fix: sqflite_sqlcipher ^3.4.0 |
| `0b688de1` | T26a+b: E2eeService, KeyStorageService, ApiClient 키 번들 메서드 |
| `3bba7fd0` | T28b: Agent 스트리밍 릴레이 |
| `d6717a4a` | T26c: RustLib.init() in main.dart, DI 등록 |
| `c9eacc33` | T30: 그룹 채팅 백엔드 |
| `63fc3b7e` | T32: Python Agent SDK |
| `86cdd65f` | T33: Agent 스트리밍 Flutter UI |
| `217c7260` | T31: 그룹 채팅 UI |
| `c6a76a1f` | T34: Agent 초대/제거 |
| `6cb315d0` | T35: Agent 메시지 렌더링 |
| `3a4cd164` | T36: OpenClaw 어댑터 스캐폴드 |
| `d475851f` | T37: OpenClaw E2EE 브릿지 |
| `940f6847` | T38: 스트리밍 백프레셔 |
| `e435880c` | T39: Phase 2 통합 테스트 |
| `0549b3ee` | T40: Phase 2 도그푸딩 런북 |

### Phase 3 (T41–T54) ✅ 완료

| 커밋 | 내용 |
|------|------|
| `cba4b211` | T42: 멀티디바이스 seq 기반 동기화 |
| `61aaf799` | T41: 브로드캐스트 채널 (소유자 전용 전송) |
| `9b4b2393` | T44: 암호화 클라우드 백업 |
| `f8940753` | T43: E2EE 호환 푸시 알림 |
| `e7bff3c4` | T45: Agent 마켓플레이스 레지스트리 + 검색 |
| `d135fe45` | T46: TypeScript Agent SDK (@paw/sdk) |
| `6c91a52b` | T47: FTS5 로컬 전문검색 |
| `745c3762` | T48: 데스크톱 플랫폼 수정 + 반응형 레이아웃 |
| `f6aacd26` | T49: 웹 최적화 + PWA 지원 |
| `0772deb8` | T52: 최종 벤치마크 리포트 |
| `58fbab3a` | T51: 스팸 필터 + 신고 시스템 |
| `0b6edce2` | T54: 릴리즈 문서 (OpenAPI, README, CHANGELOG, 배포 스크립트) |
| `1b56b3d2` | T53: Phase 3 도그푸딩 런북 |

### Wave FINAL — 감사 및 수정

| 커밋 | 내용 |
|------|------|
| `1c3eb62b` | fix(flutter): widget_test 테마/렌더 테스트 수정 |
| `43b9131f` | feat(blocks): card + button 리치 메시지 블록 (Flutter 클라이언트) |
| `74d70d91` | feat(agent-ui): Agent 참여 명시적 동의 플로우 |
| `050f693d` | feat(agent): 메시지 수신 시 InboundContext를 Agent에 전송 |

---

## 7. 주요 파일 구조

```
/Users/bclaw/workspace/paw/
├── Cargo.toml                          # Cargo workspace 루트
├── docker-compose.yml                  # 개발 환경 (PostgreSQL, NATS)
├── README.md                           # 프로젝트 소개
├── CONTRIBUTING.md                     # 기여 가이드
├── CHANGELOG.md                        # 변경 이력
│
├── paw-proto/src/lib.rs                # 모든 WS 프로토콜 타입
├── paw-crypto/src/{lib.rs,mls.rs}      # MLS 암호화
├── paw-ffi/src/api.rs                  # Dart FFI API (createAccount, encrypt, decrypt)
│
├── paw-server/
│   ├── src/
│   │   ├── main.rs                     # 모든 라우트 등록
│   │   ├── auth/mod.rs                 # AppState, JWT 인증
│   │   ├── keys/                       # prekey bundle 관리
│   │   ├── agents/                     # Agent 게이트웨이, 마켓플레이스
│   │   ├── messages/                   # 메시지 CRUD, InboundContext 전송
│   │   ├── channels/                   # 브로드캐스트 채널
│   │   ├── devices/                    # 멀티디바이스 동기화
│   │   ├── push/                       # E2EE 푸시 알림
│   │   ├── backup/                     # 암호화 백업
│   │   ├── moderation/                 # 스팸 필터, 신고
│   │   ├── db/mod.rs                   # 커넥션 풀 (max_connections=20)
│   │   └── ws/                         # WebSocket hub, pg_listener
│   ├── migrations/                     # 20260101000001 ~ 20260101000019
│   └── tests/integration_test.rs       # 56개 통합 테스트
│
├── paw-client/
│   ├── pubspec.yaml
│   ├── web/manifest.json               # PWA 매니페스트
│   └── lib/
│       ├── main.dart                   # RustLib.init(), ProviderScope
│       ├── core/
│       │   ├── crypto/                 # E2eeService, KeyStorageService
│       │   ├── di/service_locator.dart # GetIt DI 컨테이너
│       │   ├── http/api_client.dart    # REST API 클라이언트
│       │   ├── platform/              # DesktopService, WebService
│       │   ├── search/search_service.dart  # FTS5 검색
│       │   ├── db/app_database.dart    # Drift DB + FTS5 가상 테이블
│       │   ├── proto/messages.dart     # WS 메시지 타입
│       │   ├── router/app_router.dart  # go_router 설정
│       │   └── ws/ws_service.dart      # WebSocket 클라이언트
│       └── features/chat/
│           ├── models/
│           │   ├── message.dart        # Message + MessageBlock (CardBlock, ActionButtonsBlock)
│           │   └── conversation.dart
│           ├── screens/               # 채팅, 대화목록, 그룹, 검색, 키 검증
│           ├── providers/chat_provider.dart
│           ├── services/
│           │   └── agent_consent_service.dart  # 동의 상태 영구 저장
│           └── widgets/
│               ├── message_bubble.dart         # 리치 블록 렌더링 포함
│               ├── stream_bubble.dart          # Agent 스트리밍 UI
│               ├── tool_indicator.dart
│               ├── e2ee_banner.dart
│               └── agent_consent_banner.dart   # 허용/거부 동의 플로우
│
├── agents/
│   └── paw-agent-sdk/                  # Python Agent SDK
│       └── paw_agent_sdk/
│           ├── agent.py                # PawAgent 클래스
│           ├── models.py
│           └── streaming.py
│
├── adapters/
│   ├── openclaw-adapter/               # OpenClaw 채널 어댑터 (TypeScript)
│   │   └── src/{adapter,types,channel,e2ee}.ts
│   └── paw-sdk-ts/                     # @paw/sdk v0.1.0
│       └── src/{agent,streaming,types,index}.ts
│
├── deploy/
│   ├── fly.toml                        # Fly.io 배포 설정
│   └── docker-compose.prod.yml
│
├── docs/
│   ├── api/openapi.yaml                # OpenAPI 3.0 명세
│   ├── sdk/python-quickstart.md
│   ├── sdk/typescript-quickstart.md
│   ├── protocol-v1.md
│   ├── benchmarks/final.md
│   ├── operations/cdn-setup.md
│   └── dogfooding/
│       ├── phase1-runbook.md
│       ├── phase2-runbook.md
│       └── phase3-runbook.md
│
└── k6/final-benchmark.js               # 성능 벤치마크 스크립트
```

---

## 8. 테스트 현황

| 구분 | 수량 | 상태 |
|------|------|------|
| Rust 통합 테스트 (paw-server) | 56개 | ✅ 전체 통과 |
| Rust 단위 테스트 (paw-proto, paw-crypto 등) | 31개 | ✅ 전체 통과 |
| Flutter 위젯/단위 테스트 | 28개 | ✅ 전체 통과 |
| TypeScript Agent SDK 테스트 | 전체 | ✅ 전체 통과 |

---

## 9. Wave FINAL 감사 결과

### F1 — Plan Compliance Audit
| 항목 | 결과 |
|------|------|
| Must Have (19개 중 16개) | ✅ PASS |
| Must Have #2: InboundContext 자동 전송 | ✅ 수정 완료 (`050f693d`) |
| Must Have #6: 리치 블록 (card+button) | ✅ 수정 완료 (`43b9131f`) |
| Must Have #7: Agent 명시적 동의 플로우 | ✅ 수정 완료 (`74d70d91`) |
| Must NOT Have (6개) | ✅ 전체 PASS |
| **재검증 상태** | 🔄 재검증 필요 |

### F2 — Code Quality
| 항목 | 결과 |
|------|------|
| Flutter 테스트 | ✅ 28개 통과 (수정 완료) |
| Rust 테스트 | ✅ 87개 통과 |
| Flutter analyze | ✅ 에러 없음 (info/warning만) |
| Rust clippy (`-D warnings`) | ⚠️ 재검증 필요 (9개 에러 보고됨) |
| **재검증 상태** | 🔄 재검증 필요 |

### F3 — Manual QA
| 시나리오 | 결과 |
|----------|------|
| 메시지 전송 | ✅ PASS |
| Agent 스트리밍 | ✅ PASS |
| E2EE 키 교환 | ✅ PASS |
| 채널 브로드캐스트 | ✅ PASS |
| 마켓플레이스 | ✅ PASS |
| E2EE 푸시 | ✅ PASS |
| FTS5 검색 | ✅ PASS |
| 스팸 필터 | ✅ PASS |
| **전체** | ✅ **8/8 PASS** |

### F4 — Scope Fidelity
| 항목 | 결과 |
|------|------|
| Phase 3 태스크 준수 | ✅ 11/11 COMPLIANT |
| Must NOT 위반 | ✅ 0건 |
| 오염 검사 | ✅ CLEAN |
| **전체** | ✅ **PASS** |

---

## 10. 남은 작업

### 즉시 필요
1. **Rust clippy 에러 수정** — `cargo clippy -- -D warnings` 실행 후 에러 목록 확인 및 수정
2. **F1 재검증** — oracle 에이전트로 3개 수정 항목 재감사
3. **F2 재검증** — clippy 수정 후 코드 품질 재감사
4. **최종 커밋** — `fix(final): clippy warnings + wave final sign-off`

### 완료 기준
- F1: PASS (3개 수정 항목 모두 확인)
- F2: PASS (clippy 에러 0개, 테스트 전체 통과)
- F3: ✅ 이미 PASS
- F4: ✅ 이미 PASS

---

## 11. 알려진 기술 부채 / 주의사항

| 항목 | 내용 |
|------|------|
| paw-ffi frb_expand 경고 | `flutter_rust_bridge = "=2.9.0"` 생성 경고 3개 — 에러 아님 |
| Flutter deprecated API | `withOpacity` → `withValues()`, `surfaceVariant` → `surfaceContainerHighest` (기존 파일) |
| Drift .g.dart 스텁 | `flutter pub run build_runner build` 실행 전까지 스텁 상태 |
| TypeScript 툴링 | `tsc`, `typescript-language-server` 미설치 — `npm run build`로 검증 |
| Rust clippy | `notify_agents_of_message` 추가 후 불필요한 클로저 경고 가능성 |

---

*이 문서는 `/Users/bclaw/workspace/paw/docs/PROJECT_SUMMARY.md`에 저장됩니다.*
