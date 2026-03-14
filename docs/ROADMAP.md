# Paw Messenger — 실행 로드맵

> 작성일: 2026-03-15
> 기반 문서: `Paw_차세대_메신저_종합분석_및_방향.md`, `PROJECT_SUMMARY.md`, `native-next-steps.md`
> 현재 상태: Phase 1-3 완료, 네이티브 파운데이션 완료, 기본 메시징 + Agent 스트리밍 + E2EE PoC 동작

---

## 현재 자산 요약

지금 Paw가 **이미 가지고 있는 것**을 먼저 명확히 한다.

| 영역 | 완료 항목 | 비고 |
|------|-----------|------|
| 서버 | Rust/Axum, 23개 마이그레이션, WebSocket Hub, pg_notify, Agent Gateway | 87개 Rust 테스트 통과 |
| 프로토콜 | v1 프로토콜 (StreamStart/ContentDelta/ToolStart/ToolEnd/StreamEnd) | 네이티브 스트리밍 — 업계 최선두 |
| 클라이언트 | Flutter (Web/Desktop), Kotlin/Compose (Android), SwiftUI (iOS) | paw-core UniFFI 공유 |
| E2EE | openmls 기반 PoC, prekey bundle 관리, Agent 동의 플로우 | 프로덕션 강화 필요 |
| Agent | Python SDK, TypeScript SDK, OpenClaw 어댑터, 마켓플레이스 | 기본 구조 완성 |
| 인프라 | Fly.io (NRT), CI 5개 파이프라인, Docker, k6 벤치마크 | 수평 확장 미대응 |
| 백오피스 | 스펙 문서만 존재, Admin API 2개 (suspend, reports) | **UI 없음** |

---

## 갭 분석 — 종합분석 대비 부족한 것

### Critical (없으면 "차세대 메신저"가 아닌 것)

| # | 기능 | 현재 상태 | 종합분석 근거 |
|---|------|-----------|---------------|
| G1 | Thread/Topic 에이전트 격리 | 미구현 (대화 단위만 존재) | 5/5 합의, Critical |
| G2 | ContextEngine 호환 인터페이스 | OpenClaw 어댑터 존재하나 v2026.3.7 훅 미구현 | Phase 1 필수 |
| G3 | Per-Agent 권한 스코프 | 에이전트 추가/제거만 존재, 세분화 없음 | 5/5 합의 |
| G4 | E2EE 프로덕션 강화 | PoC 수준, Double Ratchet 미완 | 5/5 합의, 핵심 차별점 |

### High (경쟁력에 직결되는 것)

| # | 기능 | 현재 상태 | 비고 |
|---|------|-----------|------|
| G5 | Durable Channel Binding | 서버 재시작 시 바인딩 손실 | 4/5 합의 |
| G6 | Structured Output 확장 | card + button만 구현 | form, table, date-picker, code-editor 필요 |
| G7 | 추론 가시화 (Reasoning Blocks) | ToolStart/ToolEnd만 존재 | 사고 과정 시각화 없음 |
| G8 | Memory Hot-Swap 인터페이스 | 미구현 | 에이전트 메모리 백엔드 교체 |
| G9 | 백오피스 Admin Dashboard | API만 존재, UI 없음 | 운영 불가 상태 |
| G10 | 관측 가능성 (Observability) | request_id만 존재 | 메트릭, 트레이싱, 알림 필요 |

### Medium (플랫폼화에 필요한 것)

| # | 기능 | 비고 |
|---|------|------|
| G11 | MCP 서버 네이티브 내장 | 현재는 어댑터 방식 |
| G12 | A2A 프로토콜 지원 | 에이전트 간 통신 표준 |
| G13 | Cron 에이전트 스케줄링 | 주기적 에이전트 실행 |
| G14 | Mini App 컨테이너 | 인라인 웹앱 |
| G15 | Rate Limiting | API 보호 미비 |
| G16 | 에이전트 실패 처리 (Graceful Degradation) | circuit breaker, 비용 게이지 |

---

## 실행 계획

### Stream 0: 기반 정비 (2주)

> **목표**: 다음 스트림의 전제조건을 충족하고, 운영 가능 상태로 진입

현재 코드베이스에 남은 기술 부채를 정리하고, 이후 작업의 기반을 놓는다.

```
Week 1-2
├── 0-1. Rust clippy 에러 수정 + Flutter deprecated API 정리
├── 0-2. Rate Limiting 추가 (tower-governor 또는 axum-rate-limit)
│         └── 모든 인증/공개 엔드포인트에 적용
├── 0-3. 관측 가능성 기초 (G10 일부)
│         ├── tracing + tracing-subscriber 구성
│         ├── Prometheus 메트릭 엔드포인트 (/metrics)
│         └── 핵심 지표: WS 연결 수, 메시지 처리량, API 지연, 에러율
├── 0-4. 네이티브 앱 프로덕션화 (native-next-steps.md 항목)
│         ├── Android/iOS: 로딩/빈/에러 상태 UI
│         ├── 프로필/설정 화면 완성
│         └── 푸시 알림 실제 연동 검증
└── 0-5. CI 안정화
          ├── Android lint 환경 수정
          └── 전 플랫폼 CI 그린 확인
```

**완료 기준**:
- `cargo clippy -- -D warnings` 에러 0
- `/metrics` 엔드포인트 동작
- Rate limiting 동작 확인 (429 응답)
- 네이티브 앱 기본 플로우 스크린샷 QA 통과

---

### Stream 1: 에이전트 격리 아키텍처 (4주)

> **목표**: Thread 기반 에이전트 격리 + ContextEngine 호환 → OpenClaw 공식 권장 채널 등재 조건 충족

이 스트림이 Paw의 **존재 이유를 증명**하는 핵심이다.

```
Week 3-4: Thread/Topic 시스템 (G1)
├── 1-1. DB 스키마: threads 테이블 + conversation_threads 관계
│         ├── thread_id, conversation_id, title, creator_id, agent_scope
│         └── 메시지에 thread_id FK 추가
├── 1-2. 서버: Thread CRUD API
│         ├── POST /conversations/{id}/threads
│         ├── GET  /conversations/{id}/threads
│         └── POST /conversations/{id}/threads/{tid}/messages
├── 1-3. WebSocket: thread_id 기반 메시지 라우팅
│         └── 같은 대화의 다른 스레드 메시지는 별도 구독
├── 1-4. 클라이언트: Thread UI
│         ├── Flutter Web/Desktop: 스레드 목록 + 인라인 스레드 뷰
│         ├── Android: Compose 스레드 UI
│         └── iOS: SwiftUI 스레드 UI

Week 5-6: ContextEngine 호환 + Per-Agent 권한 (G2, G3)
├── 1-5. ContextEngine 생명주기 훅 구현 (OpenClaw v2026.3.7)
│         ├── bootstrap: 에이전트 초기화 시 컨텍스트 로드
│         ├── ingest: 새 메시지 수신 시 컨텍스트 업데이트
│         ├── assemble: 에이전트 호출 전 컨텍스트 조립
│         ├── compact: 컨텍스트 압축 (토큰 예산 초과 시)
│         ├── afterTurn: 에이전트 응답 후 후처리
│         ├── prepareSubagentSpawn: 서브에이전트 생성 전 컨텍스트 준비
│         └── onSubagentEnded: 서브에이전트 종료 후 컨텍스트 병합
├── 1-6. OpenClaw 어댑터 v2 — ContextEngine Plugin 규격 준수
│         └── adapters/openclaw-adapter/ 리팩터링
├── 1-7. Per-Agent 권한 스코프 (G3)
│         ├── DB: agent_permissions 테이블
│         │     └── agent_id, conversation_id, thread_id, permissions (READ/WRITE/EXECUTE)
│         ├── 서버: 권한 미들웨어 (에이전트 API 요청 시 검증)
│         └── 클라이언트: 에이전트 권한 설정 UI
└── 1-8. Durable Channel Binding (G5)
          ├── DB: agent_bindings 테이블 (agent_id ↔ channel 매핑 영속화)
          └── 서버 재시작 시 자동 바인딩 복구
```

**완료 기준**:
- 하나의 대화에서 여러 스레드 생성, 각 스레드에 다른 에이전트 바인딩 가능
- OpenClaw ContextEngine 7개 훅 모두 동작
- 에이전트별 READ/WRITE/EXECUTE 권한 분리 동작
- 서버 재시작 후 에이전트-채널 바인딩 유지 확인

---

### Stream 2: E2EE 프로덕션 + 백오피스 (5주)

> **목표**: E2EE를 PoC에서 프로덕션으로 올리고, 운영 가능한 Admin Dashboard를 구축

두 트랙을 **병렬**로 진행한다.

```
Track A: E2EE 프로덕션 (G4) — 서버/코어 팀
Week 7-8:
├── 2A-1. openmls → 프로덕션 통합
│         ├── paw-crypto: MLS 그룹 생성/참여/업데이트/제거
│         ├── paw-core: E2EE 래퍼 (UniFFI 노출)
│         └── paw-server: 암호화된 메시지 저장/릴레이 (서버는 복호화 불가)
├── 2A-2. Key Verification 강화
│         ├── Safety Number 비교 UI
│         └── QR 코드 기반 검증
└── 2A-3. E2EE + Agent 공존 메커니즘
          ├── 에이전트 참여 시 별도 MLS 서브그룹 생성
          ├── 사용자 동의 후에만 에이전트에게 복호화 키 전달
          └── 에이전트 제거 시 키 로테이션

Week 9:
└── 2A-4. E2EE 전환 마이그레이션
          ├── 기존 평문 대화 → E2EE 대화 전환 플로우
          ├── 디바이스별 키 동기화
          └── 감사 로그 (E2EE 상태 변경 기록)

Track B: 백오피스 Dashboard — 프론트 팀
Week 7-9:
├── 2B-1. 기술 선정
│         └── React + Vite + TailwindCSS (별도 SPA, /admin 경로)
│             또는 Flutter Web 재활용 (기존 투자 활용)
│
├── 2B-2. 인증/권한
│         ├── Admin 역할 DB 스키마 (users.role = 'admin' | 'moderator' | 'user')
│         ├── Admin JWT 발급 + 미들웨어
│         └── 역할별 접근 제어 (RBAC)
│
├── 2B-3. Dashboard 핵심 화면
│         ├── 📊 Overview
│         │     ├── 활성 사용자 수 (DAU/WAU/MAU)
│         │     ├── 메시지 처리량 (실시간 그래프)
│         │     ├── WS 연결 수 (현재/피크)
│         │     ├── 에러율 (API 4xx/5xx)
│         │     └── 시스템 상태 (DB/MinIO/NATS 헬스)
│         │
│         ├── 👤 사용자 관리
│         │     ├── 사용자 검색/조회
│         │     ├── 정지/해제 (기존 API 활용)
│         │     ├── 디바이스 목록/강제 로그아웃
│         │     ├── 신고 이력 조회
│         │     └── 사용자별 활동 로그
│         │
│         ├── 🤖 에이전트 관리
│         │     ├── 등록된 에이전트 목록
│         │     ├── 마켓플레이스 게시 승인/거부
│         │     ├── 에이전트별 사용 통계 (호출 수, 토큰 소비, 에러율)
│         │     ├── 에이전트 강제 비활성화
│         │     └── 에이전트 권한 감사 로그
│         │
│         ├── 🛡️ 모더레이션
│         │     ├── 신고 대기열 (기존 API 활용)
│         │     ├── 스팸 패턴 관리
│         │     ├── 콘텐츠 검토 큐
│         │     └── 조치 이력 (정지/경고/해제)
│         │
│         ├── 📈 분석
│         │     ├── 메시지 유형별 분포 (텍스트/미디어/에이전트)
│         │     ├── 에이전트 응답 시간 분포
│         │     ├── E2EE 채택률
│         │     ├── 플랫폼별 사용자 분포 (iOS/Android/Web/Desktop)
│         │     └── Retention 지표
│         │
│         └── ⚙️ 시스템 설정
│               ├── 서버 설정 조회 (읽기 전용)
│               ├── Rate Limit 설정 변경
│               ├── 공지사항 브로드캐스트
│               └── 서비스 점검 모드 on/off

Week 10-11:
├── 2B-4. 실시간 연동
│         ├── Admin WebSocket: 실시간 메트릭 스트림
│         └── 알림: 스팸 급증, 서버 에러 스파이크, WS 연결 급감 시 알림
├── 2B-5. 감사 로그 시스템
│         ├── DB: audit_logs 테이블 (actor, action, target, metadata, timestamp)
│         ├── 모든 Admin 액션 자동 기록
│         └── Dashboard에서 조회/필터링
└── 2B-6. SLA/SLO 대시보드
          ├── API 응답시간 P50/P95/P99
          ├── WS 연결 성공률
          ├── 메시지 전달 지연 (send → receive)
          └── 에이전트 응답 성공률/지연
```

**완료 기준**:
- E2EE 1:1 대화 + 그룹 대화 동작 (openmls 기반)
- 에이전트 참여 E2EE 대화에서 동의 → 키 공유 → 통신 → 제거 → 키 로테이션 전 과정 동작
- Admin Dashboard 로그인 → Overview → 사용자 관리 → 에이전트 관리 기본 플로우 동작
- 감사 로그 기록 + 조회 동작

---

### Stream 3: 차별화 UX (4주)

> **목표**: "Paw에서만 가능한 에이전트 경험" 구축

```
Week 12-13: Structured Output + Reasoning (G6, G7)
├── 3-1. 메시지 블록 타입 확장
│         ├── FormBlock: 텍스트 입력, 드롭다운, 체크박스, 날짜 선택
│         ├── TableBlock: 데이터 테이블 (정렬/필터)
│         ├── CodeBlock: 구문 강조 + 복사 + 실행(선택적)
│         ├── ChartBlock: 간단한 차트 (bar, line, pie)
│         ├── ApprovalGateBlock: 승인/거부 버튼 + 타임아웃
│         └── ProgressBlock: 진행률 표시
├── 3-2. 프로토콜 v2 블록 타입 정의
│         └── paw-proto: 새 블록 enum 추가 (하위 호환 유지)
├── 3-3. 클라이언트 렌더러
│         ├── Flutter: 각 블록 위젯
│         ├── Android: 각 블록 Compose 컴포넌트
│         └── iOS: 각 블록 SwiftUI 뷰
└── 3-4. 에이전트 SDK 업데이트
          ├── Python: Block 빌더 API
          └── TypeScript: Block 빌더 API

Week 14-15: Reasoning Visualization + Agent UX (G7, G16)
├── 3-5. 추론 가시화
│         ├── 프로토콜: ReasoningStart/ReasoningStep/ReasoningEnd 이벤트 추가
│         ├── 서버: 에이전트 추론 이벤트 릴레이
│         ├── 클라이언트: 접이식(collapsible) 추론 트리 UI
│         │     ├── 단계별 Tool 호출 시각화
│         │     ├── 소요 시간 표시
│         │     └── 토큰/비용 게이지
│         └── SDK: 추론 이벤트 emit API
├── 3-6. 에이전트 실패 처리 (Graceful Degradation, G16)
│         ├── 서버: Circuit Breaker (에이전트 연속 실패 시 자동 차단)
│         ├── 서버: 에이전트 타임아웃 (설정 가능, 기본 30초)
│         ├── 서버: 토큰/비용 한도 (대화별, 사용자별)
│         ├── 클라이언트: 실패 시 즉시 사용자 제어권 반환 UI
│         └── 클라이언트: "에이전트가 응답하지 않습니다" + 재시도/취소 선택
├── 3-7. 인라인 에이전트 호출
│         ├── @agentname 멘션 파싱
│         ├── 에이전트 자동완성 제안
│         └── 채팅 이탈 없이 즉시 호출
└── 3-8. Memory Hot-Swap 인터페이스 (G8)
          ├── 서버: 에이전트 메모리 백엔드 플러그인 인터페이스
          │     └── trait AgentMemoryBackend { load, save, swap }
          ├── 기본 구현: PostgreSQL 기반 (내장)
          └── 확장 포인트: 벡터 DB, 외부 LLM 메모리 연결
```

**완료 기준**:
- 에이전트가 FormBlock을 보내면 사용자가 폼 안에서 응답 가능
- 에이전트 추론 과정이 실시간으로 접이식 트리로 표시
- 에이전트 30초 타임아웃 → 자동 중단 + 사용자 알림
- @agentname으로 채팅 내 즉시 호출 동작

---

### Stream 4: 플랫폼 인프라 (4주)

> **목표**: 수평 확장 + MCP 네이티브 + 스케줄링 기반 구축

```
Week 16-17: 인프라 확장
├── 4-1. NATS JetStream 프로덕션 전환
│         ├── pg_notify → NATS 전환 (수평 확장 대응)
│         ├── 멀티 서버 인스턴스 지원
│         └── 메시지 영속성 보장 (JetStream)
├── 4-2. CDN + 미디어 파이프라인
│         ├── CloudFront/Cloudflare CDN 연동
│         ├── 이미지 리사이징 (썸네일 자동 생성)
│         └── 미디어 만료 정책
└── 4-3. 데이터베이스 최적화
          ├── Read Replica 구성
          ├── 파티셔닝 (messages 테이블 — 월별)
          └── 커넥션 풀 튜닝

Week 18-19: MCP + 스케줄링 (G11, G13)
├── 4-4. MCP 서버 네이티브 내장 (G11)
│         ├── paw-server에 MCP 엔드포인트 추가
│         ├── 에이전트가 세션 시작 시 사용 가능한 도구 동적 탐색
│         ├── Tool 등록/검색 API
│         └── Tool 실행 샌드박스 (WASM 또는 프로세스 격리)
├── 4-5. Cron 에이전트 스케줄링 (G13)
│         ├── DB: scheduled_agents 테이블
│         ├── 서버: 크론 스케줄러 (tokio-cron-scheduler)
│         ├── API: CRUD for scheduled tasks
│         ├── 클라이언트: 스케줄 등록/관리 UI
│         │     └── "매일 오전 6시 브리핑", "매주 월요일 리포트" 등
│         └── 실행 이력 + 실패 알림
└── 4-6. Webhook 시스템
          ├── 외부 서비스 연동 (GitHub, Jira, Slack 등)
          ├── Webhook 등록/관리 API
          └── 수신 이벤트 → 대화 메시지 변환
```

**완료 기준**:
- 2대 이상의 서버 인스턴스에서 메시지 정상 라우팅
- MCP 도구 동적 탐색 + 실행 동작
- Cron 에이전트 등록 → 예약 시간에 자동 실행 → 결과 대화에 전송
- Webhook 수신 → 지정 대화에 알림 전송

---

### Stream 5: 생태계 확장 (6주)

> **목표**: Paw를 "에이전트가 배포·발견·거래되는 플랫폼"으로 전환

```
Week 20-22: 에이전트 마켓플레이스 v2
├── 5-1. 마켓플레이스 강화
│         ├── 에이전트 카테고리/태그 시스템
│         ├── 평점/리뷰
│         ├── 설치 수 통계
│         ├── 개발자 대시보드
│         └── 에이전트 버전 관리 (업데이트 알림)
├── 5-2. A2A 프로토콜 지원 (G12)
│         ├── Agent Card 규격 구현 (역량/입출력/프로토콜 기술)
│         ├── 에이전트 간 발견 (discovery) API
│         ├── 에이전트 간 메시지 라우팅
│         └── 멀티에이전트 오케스트레이션 기초
│              ├── 우선순위 체계
│              ├── 잠금(locking) 메커니즘
│              └── 충돌 시 사용자 선택 UI
└── 5-3. 에이전트 경제 모델 기초
          ├── 사용량 추적 (토큰, API 호출, 실행 시간)
          ├── 개발자별 사용량 리포트
          └── 과금 인터페이스 (추후 결제 연동 준비)

Week 23-25: 고급 기능
├── 5-4. Mini App 컨테이너 (G14)
│         ├── iframe 기반 인라인 웹앱 컨테이너
│         ├── 보안 샌드박스 (CSP, postMessage 기반 API)
│         ├── 호스트-게스트 통신 프로토콜
│         └── Mini App 등록/관리 API
├── 5-5. 음성/영상 통화 기초
│         ├── WebRTC 시그널링 서버
│         ├── 1:1 음성 통화
│         └── 화면 공유 (에이전트 협업 시나리오)
├── 5-6. 연합(Federation) 기초 준비
│         ├── 서버 간 프로토콜 설계
│         ├── 사용자 ID 연합 주소 체계 (user@server.paw)
│         └── 메시지 라우팅 (로컬 → 원격 서버)
└── 5-7. 고급 검색
          ├── 시맨틱 검색 (임베딩 기반)
          ├── 에이전트 대화 필터
          └── 날짜/발신자/유형 복합 필터
```

**완료 기준**:
- 마켓플레이스에서 에이전트 카테고리 탐색 → 설치 → 리뷰 전 과정 동작
- A2A: 두 에이전트가 같은 대화에서 협업하며 충돌 시 사용자에게 선택지 제시
- Mini App이 대화 내에서 렌더링되고 호스트와 데이터 교환

---

## 종합분석 문서가 누락한 항목 — 보충

### 1. 백오피스/운영 도구 (종합분석에서 완전 누락)

종합분석 문서는 사용자/에이전트 관점에만 집중하고, **운영자 관점**을 전혀 다루지 않았다.
Stream 2에서 이를 보충했지만, 추가로 필요한 항목:

| 항목 | 필요 이유 |
|------|-----------|
| **CS 티켓 시스템** | 사용자 문의 추적/대응 (초기에는 외부 도구 연동으로 대체 가능) |
| **장애 대응 플레이북** | 토큰 대량 만료, WS 장애, DB 장애 시 절차 |
| **A/B 테스트 인프라** | 기능 점진적 롤아웃 (feature flag 시스템) |
| **컴플라이언스 도구** | GDPR 데이터 삭제 요청, 데이터 내보내기, 보존 정책 |

### 2. 개발자 경험 (DX) — 과소평가

| 항목 | 현재 상태 | 필요 |
|------|-----------|------|
| **Agent SDK 문서** | quickstart만 존재 | 전체 API 레퍼런스, 튜토리얼, 예제 |
| **에이전트 로컬 개발 환경** | docker-compose 수동 설정 | `paw dev` CLI로 원커맨드 시작 |
| **에이전트 테스트 프레임워크** | 없음 | 모의 서버, 대화 시뮬레이션 |
| **OpenAPI 클라이언트 생성** | openapi.yaml 존재 | 자동 SDK 생성 파이프라인 |

### 3. 사용자 경험 기본기 — 종합분석이 "당연히 있다"고 가정한 것들

| 항목 | 현재 상태 | 우선순위 |
|------|-----------|----------|
| **메시지 편집/삭제** | 미구현 | Stream 0-1 사이에 추가 |
| **답장(Reply)** | 미구현 | Thread와 함께 구현 |
| **메시지 반응(Reaction)** | 미구현 | Stream 1 이후 |
| **파일 공유 개선** | 기본 업로드만 | 미리보기, 드래그&드롭 |
| **알림 세분화** | 뮤트만 존재 | 대화별/에이전트별/키워드별 알림 |
| **다국어 지원** | i18n 기초 존재 | 전체 UI 번역 (한/영/일 최소) |

### 4. 보안 강화 — 종합분석이 E2EE 외에 누락한 것

| 항목 | 설명 |
|------|------|
| **에이전트 코드 서명** | 마켓플레이스 에이전트의 무결성 검증 |
| **CSP (Content Security Policy)** | 웹 클라이언트 XSS 방어 |
| **에이전트 샌드박싱** | 악의적 에이전트의 서버 자원 남용 방지 |
| **IP 기반 이상 탐지** | 비정상 로그인 패턴 감지 |
| **2FA** | OTP 외 TOTP/WebAuthn 추가 인증 |

### 5. 모니터링/SRE — 서비스 안정성

| 항목 | 필요 이유 |
|------|-----------|
| **분산 트레이싱** | OpenTelemetry — 요청 흐름 전체 추적 |
| **알림 체계** | PagerDuty/Slack 연동, 에스컬레이션 정책 |
| **로그 집계** | ELK/Loki — 구조화된 로그 수집/검색 |
| **카나리 배포** | 트래픽 일부에만 새 버전 적용 |
| **자동 스케일링** | CPU/메모리 기반 인스턴스 자동 확장 |

---

## 타임라인 요약

```
Month 1          Month 2          Month 3          Month 4          Month 5-6
──────────────── ──────────────── ──────────────── ──────────────── ────────────────
 Stream 0         Stream 1         Stream 2                          Stream 5
 기반 정비         에이전트 격리     ┌─ E2EE 프로덕션   Stream 3         생태계 확장
 (2w)             Thread/Topic    │  (3w)           차별화 UX         A2A, 마켓v2
                  ContextEngine   └─ 백오피스 v1     (4w)             Mini App
                  권한/바인딩        (5w 병렬)                         Federation
                  (4w)                              Stream 4
                                                    플랫폼 인프라
                                                    MCP, NATS, Cron
                                                    (4w)

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  ▲ Phase 1 목표                    ▲ Phase 2 목표                 ▲ Phase 3 시작
  "OpenClaw 권장 채널 등재"          "Paw에서만 가능한 경험"         "에이전트 생태계 허브"
```

---

## 우선순위 판단 근거

1. **Stream 0 (기반 정비) 선행 이유**: Rate limiting/관측 가능성 없이 에이전트 격리를 구축하면, 악의적/오동작 에이전트가 서버를 마비시킬 수 있다.

2. **Stream 1 (에이전트 격리) 최우선 이유**: 종합분석 5/5 합의 항목이 3개(Thread격리, 권한, E2EE) 포함. 이 중 Thread와 권한은 E2EE보다 구현 복잡도가 낮고, OpenClaw 공식 등재라는 명확한 성공 지표가 있다.

3. **Stream 2 E2EE + 백오피스 병렬 이유**: E2EE는 서버/코어 팀, 백오피스는 프론트 팀 — 자원 충돌 없이 병렬 가능. 백오피스 없이는 서비스 운영 자체가 불가능.

4. **Stream 3이 Stream 4 앞인 이유**: Structured Output과 Reasoning Blocks는 기존 인프라 위에서 구현 가능하지만, 사용자에게 "Paw만의 차이"를 체감시키는 핵심 기능이다. 인프라 확장은 트래픽이 실제로 증가한 후에도 충분하다.

5. **Stream 5 (생태계) 마지막인 이유**: 플랫폼화는 1-4의 모든 기반 위에서만 의미가 있다. 에이전트 격리, 권한, E2EE, 구조화 메시지가 없는 마켓플레이스는 껍데기다.

---

## 리스크 및 완화

| 리스크 | 영향 | 완화 전략 |
|--------|------|-----------|
| openmls 프로덕션 성숙도 | E2EE 일정 지연 | vodozemac 대안 준비, 커뮤니티 이슈 모니터링 |
| Thread 도입 시 기존 메시지 호환 | 마이그레이션 복잡도 | thread_id를 nullable로 추가, 기존 메시지는 "기본 스레드" |
| 백오피스 기술 선정 | 개발 속도 vs 일관성 | Flutter Web 재활용 시 기존 투자 활용, React 선택 시 속도 우위 |
| ContextEngine 스펙 변경 | 어댑터 재작업 | 인터페이스 추상화, OpenClaw 릴리즈 노트 추적 |
| 1인/소규모 팀 병목 | 병렬 트랙 실행 불가 | Stream 2 병렬을 포기하고 순차 실행, 총 일정 +3주 |
