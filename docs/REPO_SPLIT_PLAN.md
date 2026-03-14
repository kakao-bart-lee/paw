# 저장소 분리 계획

> 작성일: 2026-03-15
> 현재: 단일 모노레포 (`paw/`)
> 목표: 팀/도메인별 독립 개발 속도 확보

---

## 현재 의존성 그래프

```
                    ┌──────────┐
                    │ paw-proto│  (WebSocket 메시지 타입)
                    └────┬─────┘
                    ┌────┴─────┐
               ┌────▼──┐  ┌───▼────┐
               │paw-   │  │paw-    │
               │server │  │core    │
               └───┬───┘  └───┬────┘
                   │      ┌───┴────────────┐
                   │  UniFFI 아티팩트   UniFFI 아티팩트
                   │      │                │
              ┌────▼──┐ ┌─▼──────┐  ┌──────▼──┐
              │deploy │ │paw-    │  │paw-     │
              │Docker │ │android │  │ios      │
              └───────┘ └────────┘  └─────────┘

  ─── 완전 독립 (WebSocket/HTTP 통신만) ───

  ┌──────────┐  ┌────────────┐  ┌─────────────────┐
  │paw-client│  │paw-agent-  │  │openclaw-adapter  │
  │(Flutter) │  │sdk (Python)│  │+ paw-sdk-ts (TS) │
  └──────────┘  └────────────┘  └─────────────────┘

  ─── 아직 없음 ───

  ┌──────────┐
  │paw-admin │ (백오피스 — 신규)
  └──────────┘

  paw-crypto: 현재 고아 상태 (openmls PoC, 어디서도 import 안 함)
```

---

## 분리안: 7개 저장소

### 0. `paw-hq` — 시스템 HQ (신규, 코드 없음)

에이전트와 사람 모두를 위한 **단일 진실 공급원(Single Source of Truth)**.
코드는 없고, 문서 + 계약 + 오케스트레이션만 존재한다.

> OpenAI Basis 팀의 "Atlas" 레포에 해당 — 코드 레포와 별도로 운영 원칙과 팀 컨텍스트를 관리한다.

**포함 대상**:
```
paw-hq/
├── CLAUDE.md                    # 시스템 전체 에이전트 진입점
│
├── architecture/
│   ├── OVERVIEW.md              # 시스템 아키텍처 전체도
│   ├── dependency-map.md        # 저장소 간 의존성 그래프
│   └── diagrams/                # 아키텍처 다이어그램 (Mermaid/SVG)
│
├── decisions/                   # 시스템 레벨 ADR (모든 repo에 영향)
│   ├── ADR-001-otp-ed25519-over-srp.md
│   ├── ADR-002-openmls-over-vodozemac.md
│   ├── ADR-003-pg-notify-over-nats-phase1.md
│   ├── ADR-004-uniffi-shared-core.md
│   ├── ADR-005-repo-split-7-repos.md
│   └── TEMPLATE.md
│
├── contracts/                   # 저장소 간 계약
│   ├── openapi.yaml             # REST API 명세 (정본)
│   ├── protocol-v1.md           # WebSocket 프로토콜 명세 (정본)
│   ├── paw-core-ffi.udl         # UniFFI 인터페이스 정의 (사본/미러)
│   └── compatibility-matrix.md  # 버전 호환성 매트릭스
│       # paw-server v2.1 ↔ paw-core v0.4 ↔ Android v1.3 ↔ iOS v1.3 ...
│
├── roadmap/
│   ├── ROADMAP.md               # 전체 로드맵 (현재 paw/docs/ROADMAP.md 이전)
│   ├── streams/                 # 스트림별 상세 계획
│   │   ├── stream-1-agent-isolation.md
│   │   ├── stream-2-e2ee-backoffice.md
│   │   └── ...
│   └── releases/                # 릴리즈 노트
│       ├── v0.4.0.md
│       └── ...
│
├── orchestration/               # 에이전트 오케스트레이션
│   ├── task-templates/          # 크로스레포 태스크 분해 템플릿
│   │   ├── new-feature.md       # "새 기능" 태스크 분해 가이드
│   │   ├── protocol-change.md   # "프로토콜 변경" 영향 분석 가이드
│   │   └── breaking-change.md   # "하위 호환 깨짐" 마이그레이션 체크리스트
│   ├── agent-roles.md           # 에이전트 역할 정의 (구현/리뷰/보안/구조)
│   └── model-routing.md         # 태스크별 모델 배정 (Haiku/Sonnet/Opus)
│
├── quality/                     # 품질 게이트
│   ├── ai-code-review-checklist.md    # AI 코드 리뷰 체크리스트
│   ├── pre-merge-gates.md             # PR 머지 전 필수 검증
│   └── agent-legibility-scorecard.md  # 각 repo 점수 추적
│
├── integration/                 # 크로스레포 통합 테스트
│   ├── docker-compose.integration.yml # 전체 스택 통합 환경
│   ├── tests/
│   │   ├── e2e-message-flow.sh        # 서버→클라이언트 메시지 전송
│   │   ├── e2e-agent-streaming.sh     # 에이전트 스트리밍 전체 경로
│   │   ├── e2e-e2ee-flow.sh           # E2EE 키 교환→암호화 메시지
│   │   └── compatibility-check.sh     # 버전 호환성 자동 검증
│   └── .github/workflows/
│       └── integration.yml            # 야간/수동 통합 테스트
│
├── runbooks/                    # 운영 플레이북
│   ├── incident-response.md     # 장애 대응 절차
│   ├── release-process.md       # 릴리즈 절차 (전 repo 동기화)
│   └── on-call.md               # 온콜 가이드
│
└── .github/
    └── workflows/
        ├── integration.yml      # 통합 테스트
        ├── compatibility.yml    # 버전 호환성 검증
        └── sync-contracts.yml   # 계약 파일 변경 시 각 repo에 알림
```

**왜 별도 저장소인가**:

| 질문 | 답 |
|------|---|
| paw(서버)에 두면 안 되나? | 서버 repo는 Rust 코드 repo. 시스템 문서가 서버 코드 사이에 묻힘. 클라이언트 팀이 서버 repo를 클론해야 문서를 볼 수 있음. |
| 각 repo에 분산하면? | "Thread 지원 전체 계획"이 7개 repo에 흩어짐. 전체 그림을 볼 수 없음. |
| GitHub Wiki 쓰면? | 에이전트가 Wiki를 읽을 수 없음. 코드베이스에 있어야 컨텍스트로 로드 가능. |
| Notion/Confluence? | "에이전트가 접근할 수 없는 것은 존재하지 않는다" — Codex harness 원칙 위반 |

**핵심 역할 3가지**:

1. **계약 관리**: openapi.yaml과 protocol-v1.md의 정본이 여기 있다. 각 repo는 이 계약을 소비한다.

2. **크로스레포 오케스트레이션**: "Thread 지원"이라는 기능을 Unit 1(서버) → Unit 2(proto) → Unit 3-5(클라이언트, 병렬) → Unit 6(SDK) → Unit 7(통합 테스트)로 분해하는 태스크 템플릿이 여기 있다.

3. **통합 검증**: 개별 repo CI는 자체 테스트만 실행. 전체 스택이 함께 동작하는지는 여기서 docker-compose로 통합 테스트.

**CLAUDE.md (paw-hq 전용, 시스템 레벨)**:

```markdown
# CLAUDE.md — Paw Messenger System

## 이 저장소의 역할
paw-hq는 코드가 아닌 "시스템 두뇌"다.
모든 아키텍처 결정, 저장소 간 계약, 로드맵, 통합 테스트가 여기 있다.

## 저장소 전체 지도
| 저장소 | 역할 | 언어 |
|--------|------|------|
| paw | 서버 + 공유코어 (Rust workspace) | Rust |
| paw-android | 네이티브 Android 앱 | Kotlin |
| paw-ios | 네이티브 iOS 앱 | Swift |
| paw-flutter | Web/Desktop 클라이언트 | Dart |
| paw-sdk | Agent SDK (Python, TypeScript, OpenClaw) | Python/TS |
| paw-admin | 백오피스 Dashboard | React/TS |
| paw-hq | 이 저장소 — 시스템 문서 + 계약 + 통합 | Markdown |

## 의존성 흐름
paw-hq/contracts/ → 각 repo가 소비
paw(server) → Docker image → 배포
paw(core) → UniFFI artifact → paw-android, paw-ios
모든 클라이언트 → paw-hq/contracts/openapi.yaml 준수

## 크로스레포 작업 시
1. paw-hq/orchestration/task-templates/ 에서 해당 템플릿 확인
2. 태스크를 Unit으로 분해
3. 각 Unit을 해당 repo에서 실행
4. paw-hq/integration/tests/ 로 통합 검증

## 불변 규칙
- 프로토콜 변경 → 반드시 contracts/protocol-v1.md 먼저 업데이트
- API 변경 → 반드시 contracts/openapi.yaml 먼저 업데이트
- 새 ADR → decisions/ 에 추가 후 관련 repo CLAUDE.md에 참조 추가
```

**담당**: 전체 팀 (모두가 기여, 아키텍트가 관리)

---

### 1. `paw` (현재 저장소 유지) — Rust Platform

**포함 대상**:
```
paw/
├── Cargo.toml              # workspace root
├── paw-server/             # 서버
├── paw-core/               # 공유 모바일 런타임
├── paw-proto/              # 프로토콜 타입
├── paw-crypto/             # E2EE (openmls)
├── scripts/                # 빌드 스크립트 (UniFFI, NDK, iOS)
├── deploy/                 # 서버 배포 (Fly.io, Docker)
├── docker-compose.yml      # 개발 인프라
├── Dockerfile
├── k6/                     # 벤치마크
└── .github/workflows/
    ├── server.yml
    └── core.yml            # core + proto + crypto
```

**Rust를 분리하지 않는 이유**:
- `paw-server`와 `paw-core`가 `paw-proto`를 path dependency로 공유
- 분리하면 paw-proto를 별도 크레이트 레지스트리에 게시해야 함 → 오버헤드 대비 이점 없음
- Cargo workspace 단일 `Cargo.lock`으로 의존성 일관성 보장

**CI 산출물**:
- Docker 이미지 → Container Registry (서버 배포)
- `libpaw_core` UniFFI 아티팩트 → GitHub Releases
  - Android: `paw-core-android-{version}.tar.gz` (arm64-v8a, x86_64 `.so` + Kotlin 바인딩)
  - iOS: `paw-core-ios-{version}.tar.gz` (`.a` + `.xcframework` + Swift 바인딩)

**담당**: 서버/코어 팀

---

### 2. `paw-android` — Android 네이티브 앱

**포함 대상**:
```
paw-android/
├── app/
│   ├── build.gradle.kts
│   └── src/
├── gradle/
├── build.gradle.kts        # root
├── settings.gradle.kts
└── .github/workflows/
    └── android.yml
```

**현재 → 변경 사항**:

현재 paw-core를 상대 경로로 참조:
```kotlin
// 현재
java.srcDir("../../paw-core/generated/kotlin")
jniLibs.srcDir("src/main/jniLibs")  // 수동 복사
```

변경 후 GitHub Release 아티팩트 다운로드:
```kotlin
// 변경 후
// gradle task가 CI에서 paw-core 아티팩트를 다운로드
tasks.register("downloadPawCore") {
    val version = project.property("pawCoreVersion") as String
    // GitHub Releases에서 paw-core-android-{version}.tar.gz 다운로드
    // generated/kotlin/ 와 jniLibs/ 에 추출
}
```

**로컬 개발 시**:
- `paw` 저장소를 옆에 클론하고 `PAW_CORE_LOCAL=../paw` 환경변수로 로컬 빌드 사용 가능
- 또는 CI가 발행한 최신 아티팩트를 `./gradlew downloadPawCore`로 받아 사용

**담당**: Android 팀

---

### 3. `paw-ios` — iOS 네이티브 앱

**포함 대상**:
```
paw-ios/
├── Paw/
│   ├── Paw.xcodeproj
│   └── (SwiftUI sources)
├── PawCore/                # 아티팩트 저장 디렉토리
│   └── Artifacts/
└── .github/workflows/
    └── ios.yml
```

**변경 사항**: Android와 동일 패턴

```swift
// Swift Package Manager 또는 스크립트 기반
// CI에서 paw-core-ios-{version}.tar.gz 다운로드
// PawCore/Artifacts/ 에 .a + Swift 바인딩 배치
```

향후 Swift Package로 래핑하면 SPM dependency로 관리 가능:
```swift
// Package.swift (향후)
.package(url: "https://github.com/org/paw-core-swift", from: "1.0.0")
```

**담당**: iOS 팀

---

### 4. `paw-flutter` — Flutter Web/Desktop 클라이언트

**포함 대상**:
```
paw-flutter/               # 현재 paw-client/ 이름 변경
├── lib/
├── test/
├── e2e/
├── web/
├── pubspec.yaml
└── .github/workflows/
    └── flutter.yml
```

**변경 사항**: 거의 없음

Flutter 클라이언트는 이미 Rust와 완전 독립:
- 서버와 HTTP/WebSocket으로만 통신
- 자체 Dart 암호화 (cryptography, crypto 패키지)
- 자체 SQLite (Drift)

환경 설정만 분리:
```dart
// 서버 URL을 환경 변수 또는 빌드 설정으로 관리
const serverUrl = String.fromEnvironment('PAW_SERVER_URL',
    defaultValue: 'http://localhost:38173');
```

**담당**: Web/Desktop 팀 (또는 프론트 팀)

---

### 5. `paw-sdk` — Agent SDK 생태계

**포함 대상**:
```
paw-sdk/
├── python/                 # 현재 agents/paw-agent-sdk/
│   ├── paw_agent_sdk/
│   ├── pyproject.toml
│   ├── examples/
│   └── tests/
├── typescript/             # 현재 adapters/paw-sdk-ts/
│   ├── src/
│   ├── package.json
│   └── dist/
├── openclaw/               # 현재 adapters/openclaw-adapter/
│   ├── src/
│   ├── package.json
│   └── dist/
└── .github/workflows/
    ├── python-sdk.yml      # PyPI 게시
    ├── ts-sdk.yml          # npm 게시
    └── openclaw-adapter.yml
```

**변경 사항**: 패키지 게시 파이프라인 추가

```yaml
# python-sdk.yml
on:
  push:
    tags: ['python-v*']
jobs:
  publish:
    steps:
      - uses: actions/setup-python@v5
      - run: pip install build twine
      - run: python -m build
      - run: twine upload dist/*
```

**담당**: SDK/플랫폼 팀

---

### 6. `paw-admin` — 백오피스 Dashboard (신규 생성)

**초기 구조**:
```
paw-admin/
├── src/
│   ├── app/                # 라우팅, 레이아웃
│   ├── features/
│   │   ├── overview/       # 대시보드 메인
│   │   ├── users/          # 사용자 관리
│   │   ├── agents/         # 에이전트 관리
│   │   ├── moderation/     # 모더레이션
│   │   └── analytics/      # 분석
│   ├── shared/             # 공통 컴포넌트, API 클라이언트
│   └── main.tsx
├── package.json
├── vite.config.ts
├── tailwind.config.ts
└── .github/workflows/
    └── admin.yml
```

**기술 선정 권고: React + Vite + TailwindCSS**

| 기준 | Flutter Web | React + Vite |
|------|-------------|--------------|
| Admin 생태계 (차트, 테이블, 폼) | 빈약 | 풍부 (Recharts, TanStack Table 등) |
| 개발 속도 | 보통 | 빠름 (HMR, 풍부한 라이브러리) |
| 번들 크기 | 큼 (CanvasKit) | 작음 |
| 기존 투자 활용 | ✅ Flutter 경험 | ❌ 새 스택 |
| 실시간 대시보드 적합성 | 보통 | ✅ (React Query, SWR) |

Admin은 데이터 테이블, 차트, 복잡한 폼이 중심이므로 React 생태계가 유리하다.
팀이 Flutter만 경험이 있다면 Flutter Web도 가능하지만, 생태계 차이가 크다.

**담당**: 프론트 팀 (또는 풀스택)

---

## 마이그레이션 절차

### Phase A: 준비 (1일)

```bash
# 1. 현재 저장소 백업
git tag pre-split-snapshot

# 2. 분리 대상 디렉토리의 git 이력 추출 준비
# git-filter-repo 설치
brew install git-filter-repo
```

### Phase B: 독립 저장소 생성 (각 1-2시간)

이력 보존이 필요한 저장소는 `git filter-repo`로 추출.
신규 저장소는 새로 init.

```bash
# === paw-android (이력 보존) ===
git clone paw paw-android-split
cd paw-android-split
git filter-repo --path paw-android/ --path-rename paw-android/:
# GitHub에 paw-android 저장소 생성 후 push

# === paw-ios (이력 보존) ===
git clone paw paw-ios-split
cd paw-ios-split
git filter-repo --path paw-ios/ --path-rename paw-ios/:

# === paw-flutter (이력 보존) ===
git clone paw paw-flutter-split
cd paw-flutter-split
git filter-repo --path paw-client/ --path-rename paw-client/:

# === paw-sdk (이력 보존) ===
git clone paw paw-sdk-split
cd paw-sdk-split
git filter-repo \
  --path agents/paw-agent-sdk/ \
  --path adapters/paw-sdk-ts/ \
  --path adapters/openclaw-adapter/ \
  --path-rename agents/paw-agent-sdk/:python/ \
  --path-rename adapters/paw-sdk-ts/:typescript/ \
  --path-rename adapters/openclaw-adapter/:openclaw/

# === paw-admin (신규) ===
mkdir paw-admin && cd paw-admin
npm create vite@latest . -- --template react-ts
# 초기 구조 설정
```

### Phase C: 원본 저장소 정리 (paw)

```bash
cd paw

# 분리된 디렉토리 제거
git rm -r paw-android/ paw-ios/ paw-client/ agents/ adapters/

# 남는 구조:
# paw-server/, paw-core/, paw-proto/, paw-crypto/
# deploy/, docker-compose.yml, Dockerfile, scripts/, k6/
# docs/, Makefile, Cargo.toml, .github/workflows/

# Cargo workspace members 업데이트
# .github/workflows에서 android/ios/flutter 워크플로우 제거
```

### Phase D: CI/CD 연결 (2-3일)

**paw (원본) → 아티팩트 게시**:
```yaml
# .github/workflows/core-release.yml
name: Release paw-core artifacts
on:
  push:
    tags: ['core-v*']
jobs:
  build-android:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install Rust + NDK
        # ...
      - name: Build Android artifacts
        run: make core-android
      - name: Package
        run: |
          tar czf paw-core-android-${GITHUB_REF_NAME}.tar.gz \
            paw-core/generated/kotlin/ \
            paw-android-artifacts/jniLibs/
      - name: Upload to Release
        uses: softprops/action-gh-release@v2
        with:
          files: paw-core-android-*.tar.gz

  build-ios:
    runs-on: macos-latest
    steps:
      - uses: actions/checkout@v4
      - name: Build iOS artifacts
        run: make core-ios
      - name: Package
        run: |
          tar czf paw-core-ios-${GITHUB_REF_NAME}.tar.gz \
            paw-core/generated/swift/ \
            paw-ios-artifacts/
      - name: Upload to Release
        uses: softprops/action-gh-release@v2
        with:
          files: paw-core-ios-*.tar.gz
```

**paw-android → 아티팩트 소비**:
```yaml
# .github/workflows/android.yml
jobs:
  build:
    steps:
      - uses: actions/checkout@v4
      - name: Download paw-core artifact
        run: |
          VERSION=$(cat .paw-core-version)
          gh release download "core-v${VERSION}" \
            --repo org/paw \
            --pattern "paw-core-android-*.tar.gz"
          tar xzf paw-core-android-*.tar.gz
      - name: Build Android
        run: ./gradlew assembleRelease
```

### Phase E: 문서 정리 (1일)

각 저장소에 최소 README:
- 빌드 방법
- 의존성 (paw-core 버전 등)
- 로컬 개발 설정
- CI/CD 설명

---

## 분리 후 저장소 관계도

```
                    ┌─────────────────────────────┐
                    │         paw-hq              │
                    │  시스템 두뇌 (코드 없음)       │
                    │                             │
                    │  contracts/                  │
                    │  ├── openapi.yaml  (API 정본) │
                    │  ├── protocol-v1.md (WS 정본) │
                    │  └── compatibility-matrix.md │
                    │                             │
                    │  orchestration/              │
                    │  ├── task-templates/         │
                    │  └── model-routing.md        │
                    │                             │
                    │  integration/                │
                    │  └── tests/ (전체 스택 E2E)   │
                    └──────────┬──────────────────┘
                               │ 계약 + 태스크 분배
          ┌────────────────────┼────────────────────┐
          │                    │                     │
          ▼                    ▼                     ▼
┌─────────────────┐  ┌──────────────┐  ┌────────────────────┐
│  paw            │  │  클라이언트   │  │  생태계             │
│  (Rust Platform)│  │              │  │                    │
│  ├── server     │  │  paw-android │  │  paw-sdk           │
│  ├── core       │  │  paw-ios     │  │  ├── python        │
│  ├── proto      │  │  paw-flutter │  │  ├── typescript    │
│  └── crypto     │  │              │  │  └── openclaw      │
│                 │  │  paw-admin   │  │                    │
└────────┬────────┘  └──────┬───────┘  └────────────────────┘
         │                  │
         │  artifacts       │  API (HTTP/WS)
         │  (UniFFI)        │
         └──────────────────┘
```

---

## 버전 관리 전략

### paw-core 버전 고정

모바일 저장소에서 paw-core 버전을 명시적으로 관리:

```
# paw-android/.paw-core-version
core-v0.4.0

# paw-ios/.paw-core-version
core-v0.4.0
```

paw-core에 breaking change 시:
1. `paw` 저장소에서 core 태그 발행 (`core-v0.5.0`)
2. CI가 아티팩트 빌드 + GitHub Release 게시
3. 모바일 저장소에서 `.paw-core-version` 업데이트 PR
4. 자동화: Dependabot custom config 또는 GitHub Actions workflow dispatch

### API 호환성

서버 API 변경 시:
1. `paw` 저장소에서 OpenAPI spec 업데이트
2. 버전 태그 발행 (`server-v2.1.0`)
3. 클라이언트 저장소에서 API 호환성 확인 후 업데이트

---

## 실행 순서 권고

| 순서 | 작업 | 소요 | 이유 |
|------|------|------|------|
| **0** | **`paw-hq` 생성** | **3시간** | **모든 분리의 전제 조건. 계약/문서를 먼저 독립시켜야 각 repo가 참조 가능** |
| 1 | `paw-admin` 신규 생성 | 1시간 | 기존 코드 없음, 가장 간단 |
| 2 | `paw-sdk` 분리 | 2시간 | 완전 독립, 이력 추출 단순 |
| 3 | `paw-flutter` 분리 | 2시간 | 완전 독립, 변경 최소 |
| 4 | `paw-core` 아티팩트 CI 구축 | 4시간 | 모바일 분리의 전제 조건 |
| 5 | `paw-android` 분리 | 3시간 | 아티팩트 소비 방식 변경 필요 |
| 6 | `paw-ios` 분리 | 3시간 | 동일 |
| 7 | 원본 `paw` 정리 | 1시간 | 분리 완료 후 불필요 디렉토리 제거 |
| 8 | 각 repo CLAUDE.md 생성 | 2시간 | paw-hq의 시스템 CLAUDE.md를 참조하는 repo별 진입점 |

**총 소요: 약 2.5일 (집중 작업 기준)**

---

## 주의사항

1. **시스템 문서 → paw-hq**: 아키텍처, 프로토콜, API 명세, ADR, 로드맵은 paw-hq로 이전. 각 repo에는 repo 전용 문서만 유지.

2. **각 repo CLAUDE.md → paw-hq 참조**: 각 repo의 CLAUDE.md는 시스템 전체 맥락은 `paw-hq` 참조하도록 안내.
   ```markdown
   ## 시스템 전체 맥락
   시스템 아키텍처, API 계약, 프로토콜 명세는 paw-hq 저장소를 참조:
   https://github.com/org/paw-hq
   ```

3. **`.env.example`**: 서버 설정은 `paw`에, 클라이언트 설정은 각 저장소에 분리.

4. **git-filter-repo vs 새로 시작**: 이력이 중요한 저장소(server, core, android, ios)는 filter-repo 사용. SDK는 이력이 짧으므로 새로 시작해도 무방. paw-hq는 새로 init.

5. **Monorepo 도구 불필요**: Turborepo, Nx 등은 이 규모에서 오버엔지니어링. 각 저장소의 CI가 독립적으로 동작하면 충분. 크로스레포 오케스트레이션은 paw-hq의 GitHub Actions가 담당.

6. **Private 저장소**: paw-core 아티팩트에 비공개 코드가 포함되므로, GitHub Release도 private 저장소 내에서 관리. `GITHUB_TOKEN` 권한으로 cross-repo 접근.

7. **계약 변경 프로토콜**: openapi.yaml 또는 protocol-v1.md를 변경할 때는 반드시 paw-hq에서 먼저 PR → 머지 후 각 repo에서 구현. 역순(구현 먼저, 계약 나중) 금지.
