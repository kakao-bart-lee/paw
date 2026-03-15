# AI Agent Messenger Landscape — 조사 리포트

> 작성일: 2026-03-15
> 목적: OpenClaw 연동 메신저 플랫폼 현황 분석 및 Paw 필요 기능 식별
> 조사 기간: 2026년 1–3월 (최근 2개월) 기준

---

## 1. Telegram — 가장 많이 쓰이는 플랫폼

### 최근 주요 변화

| 기능 | 내용 |
|------|------|
| **Bot API 9.5** (2026-03-01) | `sendMessageDraft` — AI 스트리밍 응답 네이티브 지원. 이전엔 `editMessageText` 해킹에 의존 |
| **OpenClaw 호환** | OpenClaw가 이 API 최초 완전 지원 프레임워크로 공식 발표됨 |
| **Mini Apps** | 풀스크린, 가로모드, 홈화면 바로가기, Telegram Stars 결제, Web3/TON 통합 |
| **Topic 라우팅** | 동일 그룹 안에서 토픽별로 다른 AI 에이전트 격리 배정 가능 |
| **Inline Query** | `@botname 키워드` 방식으로 채팅 이탈 없이 쿼리 |
| **Stars 결제** | 봇 내 구독/일회성 구매 지원 |

### AI Agent 관점 강점
- 글로벌 최대 규모 봇 생태계
- Bot API 9.5로 드디어 네이티브 스트리밍 지원
- Mini Apps으로 메신저 안에서 풍부한 UI 구현 가능
- Topic 단위 에이전트 격리 (그룹 채팅)

### 한계
- 스트리밍은 Bot API 9.5 이전까지 edit-message 해킹으로만 가능
- 그룹에서는 여전히 `sendMessage + editMessageText` 조합 필요 (DM만 sendMessageDraft)
- Mini App은 별도 WebApp 서버 필요 — 메신저 자체와 분리된 UX
- 에이전트 권한 스코프 관리가 취약

---

## 2. Discord — OpenClaw 기능·목적 최적합 플랫폼

OpenClaw 아키텍처 목적(채널 기반 에이전트 라우팅)에 가장 잘 맞는 플랫폼.

### 주요 기능

| 기능 | 내용 |
|------|------|
| **슬래시 커맨드** | `/ask`, `/summarize` 등 — 채널별 다른 시스템 프롬프트 배정 가능 |
| **채널별 에이전트 격리** | 채널/스레드 단위로 에이전트 라우팅 분리 |
| **인터랙티브 컴포넌트** | 버튼, Select Menu로 에이전트 워크플로우 GUI 진행 |
| **스트리밍** | `partial` 모드: 토큰 도착 시마다 단일 메시지 실시간 편집. `block` 모드: 덩어리 단위 |
| **Durable Channel Binding** | 채널 바인딩이 서버 재시작 후에도 유지됨 (OpenClaw v2026.3.7 반영) |
| **MessageID 컨텍스트** | 에이전트가 특정 메시지를 타겟팅 가능 |

### 한계
- 스트리밍이 네이티브가 아니라 edit 기반
- 커뮤니티 메신저 중심 — 1:1 개인 채팅보다 채널 구조

---

## 3. 기타 플랫폼 현황

| 플랫폼 | AI Agent 현황 | 비고 |
|--------|--------------|------|
| **WhatsApp** | 2026년 1월부터 범용 AI 챗봇 **전면 금지** | 구조화된 비즈니스 봇만 허용. EU 반독점 조사 진행 중 |
| **Slack** | 42M+ DAU. OpenAI/Anthropic/Google 에이전트 네이티브 탑재. Canvas AI 작성, Workflow 자동 생성 | 기업용 클로즈드 플랫폼 |
| **Signal** | 공식 Bot API 없음 | 프라이버시 우선 설계상 에이전트 연동 사실상 불가 |
| **LINE, Zalo 등** | OpenClaw 채널 지원되나 AI 에이전트 기능 기초 수준 | — |

---

## 4. OpenClaw 최신 동향 (v2026.3.7)

GitHub 280,000+ stars. "에이전트 운영체제"로 진화 중.

| 신기능 | 내용 |
|--------|------|
| **Memory Hot-Swapping** | 에이전트 실행 중 메모리 백엔드(로컬/벡터DB/LLM 메모리) 교체 가능. A/B 테스트, 롤링 업그레이드, 페일오버 지원 |
| **ContextEngine Plugin** | RAG, 무손실 압축 알고리즘을 플러그인으로 장착. 긴 대화의 "망각" 문제 해결. 전체 생명주기 훅 제공 (bootstrap/ingest/assemble/compact/afterTurn/prepareSubagentSpawn/onSubagentEnded) |
| **Durable Channel Binding** | Discord/Telegram 채널 바인딩이 재시작 후에도 유지. CLI/Docs 관리 |
| **Topic-level Agent Routing** | 그룹 내 토픽별 에이전트 격리 라우팅 |
| **GPT-5.4 지원** | 최신 모델 즉시 연동 |

---

## 5. Paw가 갖춰야 할 기능

### 5-1. 필수 (현재 미구현 또는 보강 필요)

| 기능 | 근거 | 우선순위 |
|------|------|---------|
| **ContextEngine 호환 인터페이스** | OpenClaw v2026.3.7이 플러그인 슬롯을 표준화. Paw 어댑터가 생명주기 훅을 구현해야 OpenClaw와 완전 호환 | 🔴 Critical |
| **Thread/Topic 단위 에이전트 격리** | Discord의 핵심 강점. 현재 Paw의 Thread 지원(T36)이 실제 커밋에서 확인 안 됨 | 🔴 Critical |
| **Durable Channel Binding** | 재시작 후에도 에이전트-채널 바인딩 유지. OpenClaw 2026.3.7 표준 | 🟠 High |
| **Memory Hot-Swap 지원 인터페이스** | 에이전트 실행 중 메모리 백엔드 교체 가능한 인터페이스 노출 | 🟠 High |
| **에이전트별 채널 권한 스코프** | 어떤 에이전트가 어느 채널을 읽고 쓸 수 있는지 세밀한 권한 설정 | 🟠 High |
| **Mini App / WebApp 컨테이너** | Telegram Mini Apps처럼 메신저 내부에서 풍부한 UI 실행. 현재 Paw엔 없음 | 🟡 Medium |
| **Cron 에이전트 등록 UI** | OpenClaw가 cron 기반 자율 태스크 지원. "매일 오전 6시 브리핑" 같은 스케줄 에이전트를 메신저 UI에서 등록 | 🟡 Medium |

### 5-2. 차별화 강화 (경쟁 플랫폼 대비 우위)

| 기능 | 근거 |
|------|------|
| **진짜 네이티브 스트리밍** | Telegram은 9.5에서야 DM에서만 도달. Discord는 여전히 edit 기반. Paw는 이미 전용 WebSocket 이벤트 보유 — 가장 강력한 차별점 |
| **E2EE + Agent 공존** | WhatsApp은 에이전트 금지, Signal은 API 없음. Paw만 E2EE 환경에서 Agent 허용 — 독보적 포지션 |
| **인라인 에이전트 호출** | Telegram의 `@botname` 방식처럼 채팅 중 `@agentname` 태그로 즉시 에이전트 호출 |
| **에이전트 Tool 실행 시각화** | Tool Start/End 이벤트를 단계별 실행 트리로 표시. 현재는 indicator만 존재 |
| **Structured Output 블록 확장** | card + button 이후 — form, date-picker, table 블록 (Phase 4 대상) |

---

## 6. 요약

```
Telegram  : 규모 최대, 스트리밍은 Bot API 9.5에서야 해결, Mini Apps 강점
Discord   : 에이전트 라우팅 구조 최적, 스트리밍은 여전히 edit 기반
WhatsApp  : 2026년부터 범용 AI 에이전트 금지
Slack     : 기업용 AI 통합 최강, 클로즈드 플랫폼
Signal    : Bot API 없음

Paw 포지션:
  ✅ 유일하게 E2EE + Agent 공존
  ✅ 진짜 네이티브 WebSocket 스트리밍 (모든 채팅 유형)
  🎯 ContextEngine + Durable Binding 구현 시 OpenClaw 레퍼런스 메신저 등극 가능
```

---
