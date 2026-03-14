# Paw — Agentic Engineering 도입 계획

> 작성일: 2026-03-15
> 참고: OpenAI Codex Harness, Harness Engineering, Agentic Engineering 원칙

---

## 현재 에이전트 준비도 진단

OpenAI의 **Agent Legibility Score** 7개 항목으로 평가:

| # | 항목 | 현재 상태 | 등급 |
|---|------|-----------|------|
| 1 | **Bootstrap self-sufficiency** | `docker-compose.yml` + `Makefile` 존재, 하지만 원커맨드 셋업 아님 | C |
| 2 | **Task entry points** | `make dev`, `make test` 등 존재, 하지만 문서화 부족 | B |
| 3 | **Validation harness** | CI 존재 (5개), 하지만 pre-completion checklist 없음 | C |
| 4 | **Linting & formatting** | `clippy`, `flutter analyze` 존재, 커스텀 룰 없음 | B- |
| 5 | **Codebase map** | `ARCHITECTURE.md` 존재, 하지만 구식 (Phase 1 기준) | D |
| 6 | **Doc structure** | 26개 문서 존재, Progressive disclosure 패턴 없음 | C |
| 7 | **Decision records** | **없음** — 아키텍처 결정 근거가 문서화되지 않음 | F |

**종합: D+ (에이전트가 자율 작업하기에 부족)**

### 치명적 부재

| 항목 | 영향 |
|------|------|
| **CLAUDE.md 없음** | 에이전트 진입점이 없음. 에이전트는 프로젝트 맥락을 매번 스스로 파악해야 함 |
| **AGENTS.md 없음** | 멀티에이전트 작업 시 역할/경계 불명확 |
| **ADR(Architecture Decision Records) 없음** | 에이전트가 "왜 NATS 대신 pg_notify?" 같은 결정을 반복 질문 |
| **구조 테스트 없음** | 에이전트가 의존성 규칙을 위반해도 감지 불가 |
| **에이전트 전용 리뷰 체크리스트 없음** | AI 코드의 전형적 실패 모드 (과도한 추상화, 불필요한 에러 핸들링) 미감지 |

---

## Codex Harness의 4대 기둥과 Paw 적용

### 기둥 1: Constrain (제약 — 행동 공간 축소)

> "제약이 역설적으로 에이전트 생산성을 높인다. 탐색 낭비를 줄여준다."

**현재 부재 → 추가 필요:**

#### 1-1. 의존성 레이어 규칙

Codex는 `Types → Config → Repo → Service → Runtime → UI` 레이어를 강제한다.
Paw에 적용하면:

```
paw-proto (Types)
  ↓ import 가능
paw-crypto (Crypto)
  ↓
paw-core (Core Runtime)
  ↓
paw-server (Service)

※ 역방향 import 금지: paw-proto가 paw-server를 import하면 안 됨
※ paw-server가 paw-core를 import하면 안 됨 (현재도 안 함 — 유지)
```

**구현**:
```rust
// tests/architecture_test.rs (신규)
// Cargo workspace 내 의존성 방향 검증

#[test]
fn proto_has_no_internal_dependencies() {
    let cargo = std::fs::read_to_string("paw-proto/Cargo.toml").unwrap();
    assert!(!cargo.contains("paw-server"));
    assert!(!cargo.contains("paw-core"));
    assert!(!cargo.contains("paw-crypto"));
}

#[test]
fn server_does_not_depend_on_core() {
    let cargo = std::fs::read_to_string("paw-server/Cargo.toml").unwrap();
    assert!(!cargo.contains("paw-core"));
}

#[test]
fn core_does_not_depend_on_server() {
    let cargo = std::fs::read_to_string("paw-core/Cargo.toml").unwrap();
    assert!(!cargo.contains("paw-server"));
}
```

#### 1-2. 커스텀 Clippy/Lint 규칙

에이전트가 생성하는 코드의 전형적 문제를 잡는 규칙:

```toml
# .clippy.toml (강화)
max-fn-params = 5          # 에이전트가 파라미터 과다 함수를 만드는 것 방지
cognitive-complexity-threshold = 15
too-many-lines-threshold = 50
```

```yaml
# .github/workflows/structural-checks.yml (신규)
# AI가 생성한 코드의 패턴 위반 감지
- name: No duplicate utility functions
  run: |
    # 같은 시그니처의 함수가 두 크레이트에 존재하면 실패
    ./scripts/check-duplicate-fns.sh

- name: Migration naming convention
  run: |
    # 마이그레이션 파일이 타임스탬프 형식을 따르는지 검증
    ./scripts/check-migration-names.sh
```

---

### 기둥 2: Inform (정보 — 에이전트에게 맥락 제공)

> "에이전트가 컨텍스트 내에서 접근할 수 없는 것은 존재하지 않는다."

#### 2-1. CLAUDE.md 생성 (최우선)

```markdown
# CLAUDE.md — Paw Messenger

## 프로젝트 개요
Paw는 E2EE + AI Agent가 공존하는 유일한 메신저 플랫폼이다.
Rust/Axum 서버, Flutter(Web/Desktop), Kotlin(Android), SwiftUI(iOS) 클라이언트.

## 빠른 시작
make local-stack   # PostgreSQL + MinIO + NATS 시작
make dev           # 서버 실행 (localhost:38173)
make test          # 전체 테스트

## 아키텍처 핵심 규칙
- 의존성 방향: paw-proto → paw-crypto → paw-core → paw-server (역방향 금지)
- 모든 WebSocket 메시지에 "v": 1 필수
- 메시지 순서: monotonic seq (next_message_seq PL/pgSQL 함수)
- 인증: OTP + Ed25519 (SRP 사용 금지 — ADR-001 참조)
- E2EE: openmls (vodozemac/AGPL 금지 — ADR-002 참조)
- Pub/Sub: pg_notify (Phase 1-3), NATS JetStream (향후 수평 확장 시)

## 불변 규칙 (에이전트 반드시 준수)
- 새 API 엔드포인트 → openapi.yaml에 반드시 추가
- 새 DB 스키마 변경 → migration 파일 필수 (직접 ALTER 금지)
- paw-proto 변경 → 하위 호환성 필수 유지 (필드 추가만, 제거/변경 금지)
- 에이전트 스트리밍: StreamStart → ContentDelta* → StreamEnd 순서 보장
- 테스트: 새 기능은 반드시 테스트 동반 (80%+ 커버리지)

## 문서 구조
- docs/ARCHITECTURE.md      — 시스템 구조도
- docs/decisions/            — 아키텍처 결정 기록 (ADR)
- docs/api/openapi.yaml      — API 명세
- docs/protocol-v1.md        — WebSocket 프로토콜 상세
- docs/ROADMAP.md            — 실행 로드맵

## 코드 탐색 가이드
paw-server/src/main.rs       — 모든 라우트 등록 (진입점)
paw-server/src/ws/hub.rs     — WebSocket 허브 (실시간 메시지 팬아웃)
paw-server/src/agents/       — Agent 게이트웨이 + 마켓플레이스
paw-proto/src/lib.rs         — 모든 프로토콜 타입 정의
paw-core/src/core.rs         — 모바일 공유 런타임 (UniFFI)
paw-core/src/sync/engine.rs  — 메시지 동기화 엔진
```

#### 2-2. Architecture Decision Records (ADR)

```
docs/decisions/
├── ADR-001-otp-ed25519-over-srp.md
├── ADR-002-openmls-over-vodozemac.md
├── ADR-003-pg-notify-over-nats-phase1.md
├── ADR-004-uniffi-over-flutter-ffi.md
├── ADR-005-cargo-workspace-monorepo.md
├── ADR-006-drift-sqlcipher-client-db.md
└── TEMPLATE.md
```

ADR 템플릿:
```markdown
# ADR-NNN: 제목

## Status: accepted | superseded | deprecated
## Date: YYYY-MM-DD

## Context
결정이 필요했던 배경과 제약 조건.

## Decision
무엇을 결정했는가.

## Consequences
긍정적/부정적 결과. 트레이드오프.

## Alternatives Considered
검토했지만 채택하지 않은 대안과 이유.
```

에이전트가 "왜 이렇게 했지?"를 묻는 대신 ADR을 참조한다.
**이것이 Codex harness에서 말하는 "Slack에 있던 결정을 코드베이스로 옮기는 것"이다.**

#### 2-3. Progressive Disclosure 문서 구조

현재 문서가 하는 문제: 에이전트가 26개 마크다운 중 어디를 읽어야 할지 모른다.

개선: CLAUDE.md → 토픽별 문서 → 상세 코드 순서로 발견 가능하게:

```
CLAUDE.md (진입점, ~100줄)
  ├── → docs/ARCHITECTURE.md (구조도, ~200줄)
  ├── → docs/decisions/       (왜 이렇게? ADR 모음)
  ├── → docs/protocol-v1.md   (WS 프로토콜 상세)
  ├── → docs/api/openapi.yaml (REST API 명세)
  ├── → docs/ROADMAP.md       (향후 계획)
  └── → paw-server/src/main.rs (코드 진입점)
```

---

### 기둥 3: Verify (검증 — 에이전트 산출물 자동 확인)

> "harness 없는 피드백은 cage(우리), guide가 아니다."

#### 3-1. Pre-completion Checklist

에이전트가 "완료"를 선언하기 전 자동 실행:

```yaml
# scripts/verify-completion.sh (신규)
#!/bin/bash
set -e

echo "=== Pre-completion Verification ==="

# 1. 빌드
echo "[1/6] Building..."
cargo build --workspace 2>&1

# 2. Lint
echo "[2/6] Linting..."
cargo clippy --workspace -- -D warnings 2>&1

# 3. 테스트
echo "[3/6] Testing..."
cargo test --workspace 2>&1

# 4. 프로토콜 호환성
echo "[4/6] Proto backward compat..."
cargo test -p paw-proto 2>&1

# 5. 구조 테스트
echo "[5/6] Architecture tests..."
cargo test --test architecture_test 2>&1

# 6. OpenAPI 동기화
echo "[6/6] OpenAPI sync check..."
./scripts/check-openapi-sync.sh 2>&1

echo "=== All checks passed ==="
```

#### 3-2. 에이전트 전용 리뷰 체크리스트

AI 코드의 전형적 실패 모드를 잡는 전용 리뷰:

```markdown
## AI 코드 리뷰 체크리스트

### 과도한 추상화
- [ ] 한 번만 사용되는 헬퍼/유틸이 새로 생겼는가?
- [ ] 불필요한 trait/interface가 추가되었는가?
- [ ] "미래를 위한" 코드가 있는가?

### 에러 핸들링 과잉
- [ ] 발생 불가능한 에러를 처리하고 있는가?
- [ ] 내부 코드에 불필요한 입력 검증이 있는가?
- [ ] match 분기가 과도하게 세분화되어 있는가?

### 일관성
- [ ] 기존 패턴과 다른 새 패턴을 도입했는가?
- [ ] 동일 기능의 다른 구현이 이미 존재하는가?
- [ ] 네이밍이 기존 코드베이스와 일관적인가?

### 보안/불변 규칙
- [ ] paw-proto 하위 호환성이 유지되는가?
- [ ] 새 엔드포인트가 openapi.yaml에 추가되었는가?
- [ ] 새 테이블/컬럼이 migration으로 추가되었는가?
- [ ] 인증 미들웨어가 적용되었는가?
```

#### 3-3. Sub-Agent Reviewer 패턴

Codex의 "서브에이전트 리뷰어" 패턴을 적용:

```
[구현 에이전트] → 코드 작성
      ↓
[리뷰 에이전트] → AI 코드 체크리스트 검증
      ↓
[구조 에이전트] → 의존성/아키텍처 규칙 검증
      ↓
[보안 에이전트] → OWASP Top 10 검증
      ↓
  CI/CD 통과 → PR 생성 → Human 리뷰
```

이미 `.agents/skills/`에 12개 스킬이 정의되어 있다. 하지만 이것들은 **범용 템플릿**이지 Paw 특화가 아니다.

**필요한 조치**: 스킬을 Paw 코드베이스에 특화:

```markdown
# .agents/skills/backend-architect/SKILL.md (Paw 특화 버전 발췌)

## Paw 서버 규칙
- 새 라우트: paw-server/src/main.rs의 Router에 등록
- DB 접근: sqlx prepared query만 사용 (문자열 보간 금지)
- 실시간: pg_notify 트리거 패턴 준수 (ws/pg_listener.rs 참조)
- Agent Gateway: NATS 미연결 시 in-process 폴백 (agents/service.rs)
- 인증: auth/middleware.rs의 require_auth 미들웨어 적용
```

---

### 기둥 4: Correct (교정 — 피드백 루프)

> "에이전트 실패 시 자동 교정 메커니즘이 있어야 한다."

#### 4-1. 자동 교정 루프

```
에이전트 코드 작성
  ↓
cargo clippy 실패
  ↓ (자동)
에이전트가 clippy 에러 메시지를 컨텍스트로 받음
  ↓
에이전트가 수정
  ↓
재검증 (최대 3회)
  ↓ 3회 초과 시
Human escalation
```

#### 4-2. Garbage Collection 에이전트 (정기 실행)

Codex의 "엔트로피 에이전트" 패턴:

```yaml
# 주기적 실행 (cron 또는 수동)
- 사용하지 않는 import 정리
- deprecated API 사용 감지 및 마이그레이션 제안
- 테스트 커버리지 하락 감지 및 보완 PR 생성
- docs/ARCHITECTURE.md와 실제 구조의 불일치 감지
```

---

## 저장소 분리 시 각 repo의 에이전트 준비

저장소가 분리되면 **각 저장소가 독립적으로 에이전트 친화적**이어야 한다.

### 각 저장소에 반드시 필요한 것

```
각 저장소/
├── CLAUDE.md               # 에이전트 진입점 (필수)
│     ├── 프로젝트 개요 (3줄)
│     ├── 빠른 시작 (원커맨드 셋업)
│     ├── 불변 규칙
│     ├── 코드 탐색 가이드
│     └── 의존성 관계 (다른 repo와의 관계)
├── docs/decisions/          # ADR (필수)
├── scripts/verify.sh        # Pre-completion check (필수)
├── Makefile 또는 justfile   # Task entry points (필수)
└── .github/workflows/       # CI (필수)
```

### 저장소별 특수 고려사항

| 저장소 | CLAUDE.md 핵심 내용 | 에이전트 특수 제약 |
|--------|---------------------|-------------------|
| **paw** (Rust) | Cargo workspace 구조, 의존성 레이어, proto 호환성 규칙 | proto 변경 시 하위 호환성 필수 |
| **paw-android** | paw-core 아티팩트 버전 고정, UniFFI 바인딩 규칙 | `.paw-core-version` 변경 시 CI 검증 |
| **paw-ios** | 동일 | 동일 |
| **paw-flutter** | 서버 API 호환성, Drift 마이그레이션 규칙 | openapi.yaml 기반 타입 검증 |
| **paw-sdk** | 프로토콜 v1 준수, 하위 호환성 | 서버 API 변경 시 SDK 동기화 |
| **paw-admin** | 서버 API 소비자, RBAC 규칙 | Admin 전용 엔드포인트만 사용 |

---

## 멀티에이전트 오케스트레이션 패턴

저장소가 분리되면 **병렬 에이전트 실행**이 핵심이 된다.

### Work Tree 패턴

Codex가 사용하는 worktree 기반 병렬 실행:

```
main branch
  ├── worktree-1: 에이전트 A → paw-server Thread API 구현
  ├── worktree-2: 에이전트 B → paw-android Thread UI 구현
  └── worktree-3: 에이전트 C → paw-sdk Thread 지원 추가
```

각 에이전트는 독립 worktree(또는 독립 저장소)에서 작업하므로 **병합 충돌 없이 병렬 진행**.

### Task Decomposition 규칙

Agentic Engineering의 "15분 유닛 규칙" 적용:

```markdown
## 태스크 분해 기준

좋은 에이전트 태스크:
- ✅ 단일 파일 또는 2-3개 관련 파일 수정
- ✅ 독립적으로 검증 가능 (테스트 실행으로 확인)
- ✅ 명확한 완료 조건 ("X 테스트가 통과하면 완료")
- ✅ 하나의 주요 리스크만 포함

나쁜 에이전트 태스크:
- ❌ "Thread 시스템 전체 구현" (너무 큼)
- ❌ "코드 품질 개선" (완료 조건 불명확)
- ❌ "여러 서비스에 걸친 리팩터링" (리스크 다수)
```

예시 분해:
```
"Thread/Topic 시스템 구현" (큰 태스크)
  → Unit 1: threads 테이블 마이그레이션 추가 (DB만)
  → Unit 2: Thread CRUD 핸들러 (paw-server, 테스트 포함)
  → Unit 3: WS 프로토콜에 thread_id 추가 (paw-proto, 호환성 테스트)
  → Unit 4: Thread 라우팅 로직 (ws/hub.rs)
  → Unit 5: Flutter Thread 목록 UI
  → Unit 6: Flutter Thread 인라인 뷰
  → Unit 7: Android Thread UI
  → Unit 8: iOS Thread UI
  → Unit 9: 통합 테스트 (서버 + 클라이언트)
```

Unit 1-4는 순차, 5-8은 **병렬**, 9는 전체 완료 후.

### Model Routing

```
태스크 유형                    → 적합 모델
─────────────────────────────────────────────
마이그레이션 파일 생성           → Haiku (보일러플레이트)
단일 파일 버그 수정              → Haiku
API 핸들러 구현                  → Sonnet (구현 + 테스트)
Flutter/Android/iOS UI 구현      → Sonnet
proto 호환성 설계                → Opus (불변 규칙 준수 판단)
의존성 레이어 변경               → Opus (아키텍처 판단)
E2EE + Agent 공존 설계          → Opus (멀티파일 불변 조건)
보안 리뷰                       → Opus (엣지 케이스 추론)
```

---

## 실행 체크리스트

### 즉시 (저장소 분리 전)

- [ ] **CLAUDE.md 생성** — 프로젝트 진입점 (2시간)
- [ ] **docs/decisions/ 생성** — 기존 6개 주요 결정 ADR 작성 (3시간)
- [ ] **ARCHITECTURE.md 업데이트** — Phase 1 기준 → 현재 상태 반영 (1시간)
- [ ] **scripts/verify.sh 생성** — Pre-completion 체크 스크립트 (1시간)
- [ ] **구조 테스트 추가** — 의존성 방향 검증 테스트 (1시간)

### 저장소 분리 시

- [ ] 각 저장소에 CLAUDE.md 생성
- [ ] 각 저장소에 Makefile/justfile 정비 (원커맨드 셋업)
- [ ] 각 저장소에 scripts/verify.sh 생성
- [ ] paw-core 아티팩트 CI/CD 파이프라인 구축
- [ ] Cross-repo 의존성 버전 관리 체계 (.paw-core-version)

### 운영 단계

- [ ] `.agents/skills/` Paw 특화 (현재 범용 → 프로젝트 규칙 반영)
- [ ] 주간 Garbage Collection 에이전트 설정
- [ ] Agent Legibility Score 주기적 평가 (월 1회)
- [ ] 에이전트 성공률 메트릭 수집 (태스크별 모델, 토큰, 재시도, 성공/실패)

---

## Codex Harness 핵심 교훈 — Paw에 적용

### 1. "문서가 가장 영향력 있는 harness 개선이다"

> 가장 임팩트 있는 harness 개선은 종종 가장 단순하다: **더 나은 문서**.

CLAUDE.md 하나가 에이전트 생산성에 미치는 영향은 도구 10개를 추가하는 것보다 크다.
LangChain은 harness만 변경하여 벤치마크를 52.8% → 66.5%로 올렸다 (모델 변경 없이).

### 2. "제거 가능한(rippable) harness를 만들어라"

> 다음 모델 업데이트가 당신의 시스템을 깨뜨릴 것이다.

지금 추가하는 제약/가드레일이 모델이 발전하면 불필요해질 수 있다.
각 제약에 "이것이 왜 필요한가"를 주석으로 남겨서, 불필요해졌을 때 안전하게 제거할 수 있게 한다.

### 3. "머릿속의 지식은 존재하지 않는 것이다"

> Slack, Google Docs, 사람 머릿속에만 있는 아키텍처 결정은 에이전트에게 보이지 않는다.

모든 결정을 ADR로, 모든 규칙을 CLAUDE.md로, 모든 관습을 lint 규칙으로 코드화한다.

### 4. "아키텍처가 선결 조건이다"

> 깨끗한 구조일수록 에이전트가 제한된 컨텍스트 내에서 더 잘 수행한다.

저장소 분리 자체가 에이전트 성능을 높인다 — 각 저장소가 작고 집중적이면 에이전트의 컨텍스트 부담이 줄어든다.

### 5. "일일 싱크업이 더 중요해진다"

> 코드 속도가 높아질수록, 핵심 아키텍처 패턴이 변경된 것을 몇 주 후에야 알게 될 수 있다.

에이전트가 대량의 코드를 생성하는 환경에서는 짧은 일일 동기화가 오히려 더 중요하다.

---

Sources:
- [Unlocking the Codex harness (OpenAI)](https://openai.com/index/unlocking-the-codex-harness/)
- [Unrolling the Codex agent loop (OpenAI)](https://openai.com/index/unrolling-the-codex-agent-loop/)
- [Harness engineering: leveraging Codex (OpenAI)](https://openai.com/index/harness-engineering/)
- [Harness Engineering Complete Guide (NxCode)](https://www.nxcode.io/resources/news/harness-engineering-complete-guide-ai-agent-codex-2026)
- [OpenAI Harness Engineering Playbook (The Neuron)](https://www.theneuron.ai/explainer-articles/openais-harness-engineering-playbook-how-to-ship-1m-lines-of-code-without-writing-any/)
- [OpenAI Introduces Harness Engineering (InfoQ)](https://www.infoq.com/news/2026/02/openai-harness-engineering-codex/)
