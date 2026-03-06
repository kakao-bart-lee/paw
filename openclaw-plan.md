# AI-Native Lightweight Messenger — 기획서

> OpenClaw가 연동하는 20+ 메신저 플랫폼의 장점을 분석하고, AI Agent 연동에 최적화된 새로운 메신저를 설계한다.

---

## 1. 프로젝트 배경

### 1.1 OpenClaw란 무엇인가

[OpenClaw](https://github.com/openclaw/openclaw)는 2025년 11월 Peter Steinberger가 만든 오픈소스 개인 AI 에이전트 플랫폼이다. 2026년 1월 바이럴을 타며 GitHub 20만+ 스타를 달성, 역사상 가장 빠르게 성장한 오픈소스 프로젝트 중 하나가 되었다.

핵심 구조는 **Hub-and-Spoke Gateway 아키텍처**다:

```
                    ┌─────────────┐
                    │  OpenClaw   │
  WhatsApp ────────►│  Gateway    │◄──────── CLI
  Telegram ────────►│  (ws://     │◄──────── Web UI
  Discord  ────────►│  127.0.0.1: │◄──────── macOS App
  Signal   ────────►│  18789)     │◄──────── iOS/Android
  Slack    ────────►│             │
  Matrix   ────────►│  ┌───────┐  │
  LINE     ────────►│  │Agent  │  │
  iMessage ────────►│  │Runtime│  │
  ...20+ more ─────►│  └───────┘  │
                    └─────────────┘
```

각 메신저마다 **Channel Adapter**가 플랫폼별 메시지를 정규화(normalize)하여 `InboundContext`로 변환하고, Agent Runtime이 응답을 생성하면 다시 해당 채널 포맷으로 변환하여 전송한다.

### 1.2 우리의 목표

OpenClaw가 20+ 메신저에 연동하며 겪는 **각 플랫폼의 한계**를 분석하고, 그 한계를 모두 해소한 새로운 메신저를 만든다. 즉, **"OpenClaw가 가장 연동하고 싶어하는 메신저"**를 직접 만드는 것이다.

---

## 2. OpenClaw가 연동하는 메신저 플랫폼 분석

### 2.1 플랫폼별 AI Agent 연동 평가

OpenClaw가 실제로 각 메신저에 연결할 때 경험하는 장단점을 정리한다.

#### Tier 1: 쉬운 연동 (토큰만으로 연결)

**Telegram**
- 연결: Bot API 토큰 하나면 끝. 공인 IP, 도메인, SSL 불필요 (long-polling)
- 강점: 공식 Bot API (grammY 프레임워크), 인라인 키보드, 음성 메시지, 마크다운 지원
- **AI Agent 관점 한계:**
  - 봇이 대화 기록을 볼 수 없음 → 매번 컨텍스트 손실
  - 스트리밍 불가 → 2026.3 API 9.5에서 `sendMessageDraft` 도입했으나 DM 텍스트만 지원, 그 전엔 send+edit 해킹 (30msg/s 제한에 걸림)
  - 마크다운 파싱이 엄격 — 문법 오류 시 메시지 전체 거부
  - E2EE가 기본이 아님 (Secret Chat은 봇 사용 불가)

**Discord**
- 연결: Bot 토큰으로 간편 연결
- 강점: 풍부한 Embed 시스템 (색상, 필드, 이미지, 푸터), 웹훅, 리액션, 스레드
- **AI Agent 관점 한계:**
  - 스트리밍 불가 — edit-in-place가 유일한 방법
  - 일반 메시지에서 하이퍼링크 불가 (Embed 내에서만 가능)
  - 2,000자 제한 (Nitro 4,000)
  - 100서버 이상 봇에 KYC 요구, 점점 제한 강화 추세
  - E2EE 없음

#### Tier 2: 중간 난이도 (OAuth/QR 필요)

**WhatsApp**
- 연결: Baileys 라이브러리로 QR 코드 페어링 (비공식)
- 강점: 가장 널리 쓰이는 메신저, 모바일 네이티브 경험
- **AI Agent 관점 한계:**
  - **비공식 라이브러리 의존** → WhatsApp 업데이트 시 수시로 깨짐
  - Meta의 메타데이터 수집
  - 봇 전용 API가 없음 (Business API는 유료+제한적)
  - 리치 메시지 포맷 제한

**Slack**
- 연결: OAuth + Socket Mode
- 강점: 기업용 워크플로 통합, 스레드, 풍부한 Block Kit UI
- **AI Agent 관점 한계:**
  - 유료 (무료 플랜 메시지 히스토리 제한)
  - 개인용 부적합
  - 무거운 앱

#### Tier 3: 어려운 연동 (추가 시스템 설정 필요)

**Signal**
- 연결: `signal-cli` 커맨드라인 도구 통해 브릿지
- 강점: **최강 E2EE** (기본 활성), 메타데이터 최소 수집, 오픈소스, 비영리
- **AI Agent 관점 한계:**
  - **공식 Bot API가 아예 없음** — signal-cli는 우회 방법
  - 암호화 상태 관리가 복잡
  - 음성 메시지 노출 제한
  - 리치 메시지 포맷 거의 없음

**Matrix/Element**
- 연결: matrix-bot-sdk (TypeScript/Python/Rust/Kotlin/Go 등 다양한 SDK)
- 강점: **완전 오픈 프로토콜**, E2EE (Olm/Megolm), 페더레이션, 셀프호스팅, 브릿지로 다른 메신저 연결
- **AI Agent 관점 한계:**
  - 서버(Synapse) 설정이 무거움
  - E2EE 상태에서 봇 연동 복잡
  - UI(Element)가 기술 지향적이고 무거움
  - 사용자 기반이 작음

**LINE**
- 연결: Messaging API 토큰
- 강점: 아시아 시장 (일본, 태국, 대만) 지배적 점유율, 리치 메뉴
- **AI Agent 관점 한계:**
  - Push 메시지 유료 과금 모델
  - 마크다운 미지원
  - 그룹에서 봇 기능 제한

**iMessage**
- 연결: BlueBubbles/macOS 네이티브 (macOS 필수)
- 강점: Apple 생태계 통합, E2EE
- **AI Agent 관점 한계:**
  - **macOS 필수** (cross-platform 불가)
  - 공식 API 없음, 우회만 가능
  - Apple 정책 변경 시 언제든 차단 가능

### 2.2 종합 비교 매트릭스

| 평가 항목 | Telegram | Discord | WhatsApp | Signal | Matrix | Slack | LINE |
|-----------|----------|---------|----------|--------|--------|-------|------|
| **연동 난이도** | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐ |
| **공식 Bot API** | ✅ 공식 | ✅ 공식 | ❌ 비공식 | ❌ 없음 | ✅ SDK | ✅ 공식 | ✅ 공식 |
| **스트리밍 가능** | △ (2026.3~) | ❌ edit만 | ❌ | ❌ | ❌ | ❌ | ❌ |
| **컨텍스트 접근** | ❌ | 제한적 | ❌ | ❌ | ✅ Room API | ✅ 일부 | ❌ |
| **마크다운** | ✅ (엄격) | ✅ (제한) | ❌ | ❌ | ✅ | ✅ Block Kit | ❌ |
| **리치 UI (버튼/카드)** | ✅ 인라인KB | ✅ Embed | 제한적 | ❌ | 제한적 | ✅ Block Kit | ✅ Flex |
| **E2EE 기본** | ❌ | ❌ | ✅ | ✅ | ✅ (옵션) | ❌ | ❌ |
| **로컬 저장** | ❌ 서버 | ❌ 서버 | ✅ | ✅ | △ 서버+로컬 | ❌ 서버 | ❌ 서버 |
| **오픈 프로토콜** | ❌ (클라이언트만) | ❌ | ❌ | ✅ | ✅ | ❌ | ❌ |
| **대규모 그룹/채널** | ✅ 20만명 | ✅ 서버 | ✅ 1,024명 | △ 1,000명 | ✅ | ✅ | ✅ |

---

## 3. 흡수해야 할 장점 — 플랫폼별 베스트 프랙티스

### 3.1 Telegram에서 가져올 것

| 장점 | 우리 메신저에 적용 방법 |
|------|----------------------|
| **토큰 하나로 봇 연결** | Agent 등록 시 토큰 발급 → 즉시 연동. 공인 IP/도메인 불필요 |
| **Long-polling 지원** | WebSocket 기본이되, long-polling fallback도 제공. 홈서버 배포 용이 |
| **인라인 키보드** | 더 발전시켜 인터랙티브 카드/폼/버튼 블록으로 확장 |
| **대규모 채널/그룹** | 채널(1→다 브로드캐스트) + 슈퍼그룹 구조 흡수 |
| **Bot API의 단순함** | 5줄 코드로 Agent 동작하는 SDK. grammY처럼 프레임워크 수준 편의성 |

**Telegram의 한계를 개선:**
- 스트리밍을 네이티브 지원 (send+edit 해킹 불필요)
- Agent에게 대화 컨텍스트 자동 제공
- E2EE를 기본 활성화

### 3.2 Discord에서 가져올 것

| 장점 | 우리 메신저에 적용 방법 |
|------|----------------------|
| **풍부한 Embed 시스템** | 카드형 메시지 블록 — 색상, 필드, 이미지, 푸터 모두 지원 |
| **스레드** | 메시지별 스레드로 컨텍스트 격리 (Agent가 스레드 단위로 대화) |
| **리액션 트리거** | 리액션으로 Agent 액션 트리거 가능 |
| **웹훅 지원** | 외부 서비스 → 메신저 메시지 전송 웹훅 |

**Discord의 한계를 개선:**
- 일반 메시지에서도 마크다운 링크 지원
- 2,000자 제한 없앰 (긴 AI 응답 가능)
- E2EE 추가
- KYC 없는 Agent 등록

### 3.3 Signal에서 가져올 것

| 장점 | 우리 메신저에 적용 방법 |
|------|----------------------|
| **기본 E2EE (Signal Protocol)** | Double Ratchet + X3DH 기반 E2EE를 기본 활성화 |
| **로컬 저장 (SQLCipher)** | 메시지를 로컬 암호화 DB에 저장, 서버는 중계만 |
| **Sealed Sender** | 메타데이터 보호 — 서버도 누가 누구에게 보냈는지 모름 |
| **메타데이터 최소 수집** | 서버 로그 최소화, IP 프록시 지원 |

**Signal의 한계를 개선:**
- 공식 Agent/Bot API 제공 (signal-cli 우회 불필요)
- 풍부한 마크다운 + 리치 메시지 지원
- Agent가 E2EE 대화에 참여할 수 있는 프로토콜 수준 설계

### 3.4 Matrix에서 가져올 것

| 장점 | 우리 메신저에 적용 방법 |
|------|----------------------|
| **완전 오픈 프로토콜** | 프로토콜 스펙 공개 → 서드파티 클라이언트/Agent SDK 생태계 |
| **다중 SDK (TS/Python/Rust/Kotlin/Go)** | Agent SDK를 Python, TypeScript, Rust, Go로 제공 |
| **Room API로 히스토리 접근** | Agent에게 대화 히스토리 API 접근 제공 (권한 기반) |
| **셀프호스팅** | 서버 코드 공개, 기업/개인이 자체 인스턴스 운영 가능 |
| **브릿지 아키텍처** | 다른 메신저와의 브릿지 가능성 열어둠 |

**Matrix의 한계를 개선:**
- 가벼운 서버 (Synapse의 무거움 해결)
- 단순한 UI (Element의 복잡함 탈피)
- E2EE 환경에서도 쉬운 Agent 연동

### 3.5 Slack에서 가져올 것

| 장점 | 우리 메신저에 적용 방법 |
|------|----------------------|
| **Block Kit UI** | 구조화된 메시지 블록 (카드, 입력 폼, 날짜 피커, 드롭다운) |
| **앱 디렉토리** | Agent 마켓플레이스 — 검색, 설치, 평가 |
| **스레드 + 대화 히스토리 API** | Agent가 스레드 컨텍스트를 API로 조회 가능 |

**Slack의 한계를 개선:**
- 무료 (기본 기능 모두 무료)
- 개인용으로도 적합
- 가벼운 앱

### 3.6 LINE에서 가져올 것 (아시아 시장 참고)

| 장점 | 우리 메신저에 적용 방법 |
|------|----------------------|
| **Flex Message** | JSON 기반 유연한 레이아웃 메시지 → 우리의 블록 시스템에 영감 |
| **리치 메뉴** | 대화 하단 고정 메뉴 → Agent가 자주 쓰는 명령을 퀵 액션으로 |

---

## 4. 설계 원칙 — OpenClaw 연동 최적화

### 4.1 OpenClaw의 Channel Adapter가 가장 쉬운 메신저

OpenClaw는 각 메신저마다 Channel Adapter를 구현해야 한다. 우리 메신저는 OpenClaw가 **가장 적은 코드로, 가장 풍부한 기능을 활용할 수 있는** 채널이 되어야 한다.

```
OpenClaw Channel Adapter 구현 난이도 (현재):

  쉬움 ──────────────────────────── 어려움
  Telegram  Discord  Slack  Matrix  WhatsApp  Signal  iMessage
     │         │       │      │        │         │        │
     ▼         ▼       ▼      ▼        ▼         ▼        ▼
  토큰 인증  토큰 인증  OAuth  SDK     비공식    signal   macOS
  grammY    discord.js        복잡    Baileys   -cli    전용
  단순       Embed           서버     불안정    우회
             풍부            무거움

우리 메신저 목표:

  [우리 메신저] ◄── 가장 왼쪽, 가장 쉬움
     │
     ▼
  토큰 인증 (Telegram만큼 쉬움)
  + 공식 WebSocket API (안정적)
  + 네이티브 스트리밍 (유일)
  + 컨텍스트 자동 제공 (유일)
  + 풍부한 리치 블록 (Discord+Slack 수준)
  + E2EE 지원 (Signal 수준)
```

### 4.2 Agent Context Protocol — OpenClaw가 원하는 것

OpenClaw의 Gateway가 우리 메신저에 연결할 때, 다른 채널에서는 불가능한 것들을 제공한다:

```
기존 메신저에서 OpenClaw가 받는 것:
  InboundContext {
    message: "오늘 날씨 어때?"    // 단일 메시지만
    sender: "user_123"
    channel: "telegram"
    // 이전 대화? 없음. 직접 세션 DB에서 로드해야 함
  }

우리 메신저에서 OpenClaw가 받는 것:
  InboundContext {
    message: "오늘 날씨 어때?"
    sender: "user_123"
    channel: "our_messenger"
    conversation: {
      recent_messages: [...최근 50개],     // 프로토콜이 제공
      thread: { id, title, participants },
      pinned_context: [...],               // 사용자가 핀한 맥락
    }
    capabilities: {
      streaming: true,          // 토큰 스트리밍 가능
      rich_blocks: true,        // 카드/폼/버튼 사용 가능
      tool_indicators: true,    // "검색 중..." 표시 가능
      long_messages: true,      // 글자 수 제한 없음
    }
  }
```

### 4.3 스트리밍 응답 — 어떤 메신저에도 없는 기능

현재 **어떤 주요 메신저도 AI 응답 스트리밍을 네이티브 지원하지 않는다.**

| 메신저 | 스트리밍 방법 | 문제 |
|--------|-------------|------|
| Telegram | send → 반복 edit (2026.3부터 Draft) | 레이트 리밋, DM 텍스트만 |
| Discord | send → 반복 edit | 레이트 리밋, 깜빡임 |
| WhatsApp | 불가 | — |
| Signal | 불가 | — |
| Matrix | 불가 | — |
| Slack | 불가 (Block Kit 업데이트만) | — |
| **우리 메신저** | **WebSocket 네이티브 스트리밍** | **없음 — 프로토콜 수준 지원** |

우리 메신저의 스트리밍 프로토콜:

```
WebSocket Frame Format (JSON over WS):

→ { "type": "stream_start", "conv_id": "...", "agent": "..." }
→ { "type": "content_delta", "delta": "안녕" }
→ { "type": "content_delta", "delta": "하세요, " }
→ { "type": "tool_start", "tool": "web_search", "label": "날씨 검색 중..." }
→ { "type": "tool_end", "tool": "web_search" }
→ { "type": "content_delta", "delta": "서울 현재 기온은 " }
→ { "type": "content_delta", "delta": "12°C입니다." }
→ { "type": "stream_end", "metadata": { "tokens": 156 } }
```

클라이언트 UI에서는 ChatGPT/Claude처럼 글자가 실시간으로 흘러나오고, 도구 호출 시 인디케이터가 표시된다.

---

## 5. 시스템 아키텍처

### 5.1 전체 구조

```
┌─────────────────────────────────────────────────────────┐
│                      Clients                            │
│  ┌──────────┐  ┌──────────┐  ┌───────────┐             │
│  │ Mobile   │  │ Desktop  │  │ Web       │             │
│  │ (Flutter)│  │ (Flutter)│  │ (Flutter  │             │
│  │          │  │          │  │  Web)     │             │
│  └────┬─────┘  └────┬─────┘  └─────┬─────┘             │
│       └──────────────┼──────────────┘                   │
│              ┌───────▼────────┐                         │
│              │ Client SDK     │                         │
│              │ (Dart/Flutter) │                         │
│              └───────┬────────┘                         │
└──────────────────────┼──────────────────────────────────┘
                       │ WebSocket (primary) + REST (fallback)
┌──────────────────────┼──────────────────────────────────┐
│              ┌───────▼────────┐        Server           │
│              │ API Gateway    │                         │
│              │ (Rust/Axum)    │                         │
│              └───┬────┬───┬──┘                         │
│    ┌─────────────┤    │   ├─────────────┐              │
│    ▼             ▼    │   ▼             ▼              │
│ ┌──────┐  ┌──────┐   │ ┌──────┐  ┌──────────┐        │
│ │Auth  │  │Msg   │   │ │Group │  │Agent     │        │
│ │Svc   │  │Relay │   │ │/Chan │  │Gateway   │        │
│ └──────┘  └──────┘   │ └──────┘  └──────────┘        │
│                       │                                │
│              ┌────────▼───────┐                        │
│              │ NATS (MQ)     │                        │
│              └────────────────┘                        │
└───────────────────────────────────────────────────────┘
                       │
          ┌────────────┼────────────┐
          ▼            ▼            ▼
     ┌─────────┐ ┌─────────┐ ┌──────────┐
     │OpenClaw │ │Custom   │ │Community │
     │Gateway  │ │Agents   │ │Agents    │
     │Adapter  │ │(SDK)    │ │          │
     └─────────┘ └─────────┘ └──────────┘
```

### 5.2 기술 스택

| 계층 | 기술 | 근거 |
|------|------|------|
| **클라이언트** | Flutter (Dart) | 단일 코드베이스 → iOS/Android/Web/Desktop |
| **로컬 DB** | SQLCipher | Signal이 검증. 사용자 패스프레이즈로 암호화 |
| **동기화** | CRDT (Yjs 기반) | 오프라인-퍼스트 멀티디바이스. 메시지는 append-only라 CRDT 복잡도 낮음 |
| **서버** | Rust + Axum | 고성능, 메모리 안전, WebSocket 네이티브 |
| **메시지 큐** | NATS | 경량 고성능 Pub/Sub |
| **인증** | SRP + Ed25519 | 서버에 비밀번호 저장 안 함 |
| **E2EE** | Signal Protocol 기반 | 1:1은 Double Ratchet, 그룹은 Sender Keys |
| **Agent Gateway** | Rust + WebSocket | 스트리밍 중계, 컨텍스트 주입 |
| **Agent SDK** | Python (우선), TypeScript, Go | OpenClaw이 TypeScript 기반이므로 TS SDK 중요 |
| **인프라** | Docker + Fly.io (MVP) | 빠른 배포, 글로벌 에지 |

### 5.3 E2E 암호화 + Agent 참여 설계

기존 메신저들의 가장 큰 딜레마: **E2EE를 하면 봇/Agent가 대화를 읽을 수 없다.**

우리의 해법:

```
1:1 대화 + Agent:
  User ←──E2EE──→ User
       ←──별도 세션키──→ Agent
  └─ Agent는 사용자가 명시적으로 초대해야 참여
  └─ Agent 세션 키는 대화에서 제거 시 즉시 폐기
  └─ UI에 "🤖 Agent가 이 대화를 읽고 있습니다" 표시

그룹 대화 (소규모):
  Sender Keys + Agent 전용 키 슬롯

대규모 채널 (100명+):
  채널 소유자가 암호화 수준 선택:
  ├─ 공개 — 서명만
  ├─ 비공개 — 서버 암호화
  └─ 보안 — E2EE (Agent 참여 시 사용자 동의 필요)
```

---

## 6. 메시지 포맷 — 모든 플랫폼의 장점 통합

### 6.1 Markdown-First + 구조화 블록

```json
{
  "type": "message",
  "content": "검색 결과입니다:\n\n**요약**: 서울 현재 맑음, 12°C",
  "format": "markdown",
  "blocks": [
    {
      "type": "card",
      "color": "#4A90D9",
      "title": "서울 날씨",
      "fields": [
        { "name": "기온", "value": "12°C", "inline": true },
        { "name": "습도", "value": "45%", "inline": true }
      ],
      "image": "https://..."
    },
    {
      "type": "action_buttons",
      "buttons": [
        { "label": "주간 예보 보기", "action": "weekly_forecast" },
        { "label": "알림 설정", "action": "set_alert" }
      ]
    },
    {
      "type": "form",
      "fields": [
        { "name": "도시", "type": "text", "placeholder": "다른 도시 검색..." }
      ]
    }
  ]
}
```

흡수 원천:
- **Discord Embed** → `card` 블록 (색상, 필드, 이미지, 푸터)
- **Telegram Inline Keyboard** → `action_buttons` 블록
- **Slack Block Kit** → `form` 블록 (입력 필드, 날짜 피커, 드롭다운)
- **LINE Flex Message** → JSON 기반 자유 레이아웃

### 6.2 마크다운 개선

Telegram의 엄격한 파싱과 Discord의 제한적 마크다운을 개선:
- 파싱 오류 시 **graceful degradation** (거부 대신 평문 폴백)
- 글자 수 제한 없음 (AI 긴 응답 대응)
- 코드 블록 + 구문 하이라이팅
- LaTeX 수식, Mermaid 다이어그램
- 일반 메시지에서 하이퍼링크 지원

---

## 7. Agent SDK 설계

### 7.1 OpenClaw Adapter와의 관계

```
OpenClaw Gateway
     │
     ▼ WebSocket (우리 메신저 Channel Adapter)
┌────────────────────────────────┐
│   우리 메신저 Agent Gateway    │
│                                │
│  ┌─ InboundContext ──────────┐ │
│  │ message + conversation    │ │
│  │ context + capabilities    │ │
│  └───────────────────────────┘ │
│                                │
│  ┌─ OutboundStream ─────────┐ │
│  │ content_delta + tool_*   │ │
│  │ rich_blocks + metadata   │ │
│  └───────────────────────────┘ │
└────────────────────────────────┘
```

OpenClaw용 Channel Adapter는 우리가 공식 제공하여, OpenClaw 커뮤니티가 즉시 연동할 수 있게 한다.

### 7.2 최소 Agent 예시

```python
from our_messenger import Agent

agent = Agent("echo-bot", token="msg_xxx")

@agent.on_message
async def echo(ctx):
    await ctx.reply(f"당신이 말한: {ctx.message.text}")

agent.run()
```

### 7.3 AI 스트리밍 Agent 예시

```python
from our_messenger import Agent
from anthropic import AsyncAnthropic

agent = Agent("claude-bot", token="msg_xxx")
claude = AsyncAnthropic()

@agent.on_message
async def chat(ctx):
    # 프로토콜이 자동 제공하는 대화 컨텍스트
    messages = [{"role": m.role, "content": m.text} for m in ctx.conversation.recent]
    messages.append({"role": "user", "content": ctx.message.text})

    async with claude.messages.stream(
        model="claude-sonnet-4-6", messages=messages, max_tokens=2048
    ) as stream:
        async for token in stream.text_stream:
            await ctx.stream(token)  # 네이티브 토큰 스트리밍
```

### 7.4 리치 블록 Agent 예시

```python
@agent.on_message
async def weather(ctx):
    data = await fetch_weather(ctx.message.text)

    await ctx.reply(
        text=f"**{data.city}** 현재 날씨: {data.condition}",
        blocks=[
            ctx.card(
                title=f"{data.city} 날씨",
                color="#4A90D9",
                fields=[
                    ("기온", f"{data.temp}°C"),
                    ("습도", f"{data.humidity}%"),
                ],
            ),
            ctx.buttons([
                ("주간 예보", "weekly"),
                ("알림 설정", "alert"),
            ]),
        ],
    )
```

---

## 8. UI/UX 원칙

### 8.1 "기술은 감추고, 대화는 드러내라"

1. **대화 중심**: 화면 90%는 대화. 설정·기능은 필요할 때만
2. **Agent = 사람처럼**: 일반 채팅 버블 + 작은 🤖 배지로 구분
3. **스트리밍 자연스럽게**: 타이핑하듯 글자가 흘러나옴
4. **어두운 기본 테마**: 다크 모드 기본 (개발자 친화)
5. **3-탭 내비게이션**: 채팅 / Agent / 설정

### 8.2 Agent 대화 UX

```
┌──────────────────────────────────────┐
│ ◄  Claude Assistant       🤖  ⋮  │
│    "이 대화를 읽고 있습니다"         │
├──────────────────────────────────────┤
│                                      │
│  [나] 오늘 서울 날씨 어때?          │
│                                      │
│  [🤖 Claude]                        │
│  서울 날씨를 확인해볼게요.           │
│                                      │
│  ┌─ 🔍 날씨 검색 중... ──────────┐  │
│  │ ✓ 서울 현재 날씨 조회 완료     │  │
│  └────────────────────────────────┘  │
│                                      │
│  ┌─ 서울 날씨 ───────────────────┐  │
│  │ 기온: 12°C  │  습도: 45%     │  │
│  │ ☀️ 맑음                       │  │
│  └────────────────────────────────┘  │
│                                      │
│  오후에는 구름이 끼겠지만            │
│  비 소식은 없어요. █                 │ ← 스트리밍
│                                      │
│  [주간 예보 보기] [알림 설정]        │ ← 버튼 블록
│                                      │
├──────────────────────────────────────┤
│ [메시지 입력...]              📎 ➤  │
└──────────────────────────────────────┘
```

---

## 9. MVP 로드맵

### Phase 1: Core Messaging (10주)

| 기능 | 우선순위 |
|------|---------|
| 회원가입/로그인 (전화번호 + 이메일) | P0 |
| 1:1 채팅 (E2EE, 텍스트+이미지+파일) | P0 |
| Markdown 렌더링 + 코드 하이라이팅 | P0 |
| 로컬 SQLCipher 저장 | P0 |
| WebSocket 실시간 메시지 | P0 |
| 읽음 확인, 미디어 전송, 프로필 | P1 |

### Phase 2: Agent & Groups (8주)

| 기능 | 우선순위 |
|------|---------|
| Agent Gateway + 스트리밍 프로토콜 | P0 |
| Python + TypeScript Agent SDK | P0 |
| **OpenClaw Channel Adapter 공식 제공** | P0 |
| 스트리밍 UI (토큰 스트리밍 표시) | P0 |
| 리치 블록 (카드, 버튼, 폼) 렌더링 | P0 |
| 그룹 채팅 (~100명, Sender Keys) | P0 |
| 스레드 | P1 |
| 기본 모더레이션 (스팸 필터, 신고) | P1 |

### Phase 3: Scale & Polish (10주)

| 기능 | 우선순위 |
|------|---------|
| 채널 (1→다 브로드캐스트) | P0 |
| 멀티 디바이스 CRDT 동기화 | P0 |
| 푸시 알림 (FCM/APNs) | P0 |
| Agent 마켓플레이스 | P1 |
| 클라우드 백업 (옵션, 암호화) | P1 |
| 로컬 전문 검색 | P1 |
| 데스크톱 앱 (macOS, Windows, Linux) | P1 |

**총 예상: ~28주 (7개월)**

---

## 10. 수익 모델 (광고 없음)

| 모델 | 설명 |
|------|------|
| **클라우드 백업 Pro** | 무제한 암호화 클라우드 백업 + 장기 보관 |
| **비즈니스 플랜** | 팀 관리 콘솔, SSO, 규정 준수 |
| **Agent 마켓 수수료** | 유료 Agent 등록 시 70/30 분배 |
| **프리미엄 테마/스티커** | 커스텀 테마 판매 |
| **API 고트래픽** | Agent 개발자용 대량 API 과금 |

---

## 11. 오픈소스 전략

- **클라이언트**: Apache 2.0 (커뮤니티 신뢰)
- **프로토콜 스펙**: 오픈 (서드파티 생태계)
- **서버**: AGPL or BSL (셀프호스팅 가능 + 상업적 보호)
- **Agent SDK**: MIT (진입장벽 최소화)
- **OpenClaw Adapter**: MIT (OpenClaw 커뮤니티 기여)

---

## 12. 핵심 KPI

| 지표 | 목표 | 의미 |
|------|------|------|
| OpenClaw Adapter 설치 수 | 1,000+ | OpenClaw 생태계 침투 |
| Agent 등록 수 | 50+ | 개발자 생태계 시작 |
| 메시지 전송 p95 | <200ms | 체감 성능 |
| 스트리밍 TTFT | <1s | AI 응답 시작 속도 |
| 베타 사용자 | 1,000+ | 초기 트랙션 |

---

## 13. 한 줄 요약

> **"Telegram만큼 쉬운 Agent 연동 + Signal만큼 강한 프라이버시 + Discord만큼 풍부한 메시지 + 어디에도 없는 네이티브 AI 스트리밍"**

이것이 OpenClaw가 가장 먼저 연동하고 싶어할 메신저다.

---

### 참고 자료

- [OpenClaw GitHub](https://github.com/openclaw/openclaw)
- [OpenClaw Architecture Overview](https://ppaolo.substack.com/p/openclaw-system-architecture-overview)
- [OpenClaw Channel Comparison](https://zenvanriel.com/ai-engineer-blog/openclaw-channel-comparison-telegram-whatsapp-signal/)
- [Best Channel Guide](https://www.easyclawd.com/blog/choose-your-channel)
- [OpenClaw Channels (DeepWiki)](https://deepwiki.com/openclaw/openclaw/8-channels)
- [Signal Protocol (Wikipedia)](https://en.wikipedia.org/wiki/Signal_Protocol)
- [Matrix Protocol](https://matrix.org/ecosystem/sdks/)
- [Telegram Bot API](https://core.telegram.org/bots/api)
- [Local-First Software (Ink & Switch)](https://www.inkandswitch.com/essay/local-first/)
