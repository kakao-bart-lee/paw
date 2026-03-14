# Paw: Flutter → Native Mobile 마이그레이션 계획

## 목적

Paw는 현재 `paw-client/` 하나의 Flutter 코드베이스로 iOS/Android/Web/Desktop을 함께 운영하고 있습니다. 이번 전환의 목표는 **모바일(iOS/Android)만 네이티브 앱으로 옮기고**, **Web/Desktop은 기존 Flutter 클라이언트를 계속 유지**하면서, 공통 비즈니스 로직은 **공유 Rust 코어(`paw-core`)** 로 수렴시키는 것입니다.

이 문서는 현재 저장소 구조와 운영 흐름을 기준으로, 실제로 착수 가능한 순서와 검증 기준을 정리한 **실행용 마이그레이션 계획**입니다.

> 실행 현황 메모 (2026-03-14): Phase 1–7 작업이 저장소에 반영되어 있으며, 아래 “현재 상태 스냅샷 (2026-03-13 기준)”은 착수 당시의 기준선 기록입니다. 최신 현재 상태는 `README.md`, `README_kr.md`, `docs/ARCHITECTURE.md`, `docs/PROJECT_SUMMARY.md`를 따른다.

---

## 현재 상태 스냅샷 (2026-03-13 기준)

| 항목 | 현재 상태 | 근거 | 마이그레이션 의미 |
|---|---|---|---|
| Rust workspace | `paw-server`, `paw-proto`, `paw-crypto`, `paw-ffi`만 포함 | `Cargo.toml` | `paw-core` 추가 전 workspace/CI/문서 동시 수정 필요 |
| Flutter 클라이언트 | `paw-client/` 하나가 Android/iOS/Web/macOS/Linux/Windows를 모두 포함 | `paw-client/android`, `ios`, `web`, `macos`, `linux`, `windows` | 단순히 `paw-client → paw-web`으로 바꾸면 Desktop 범위가 사라져 이름이 부정확해짐 |
| 로컬 검증 명령 | `make e2e-flutter`, `make e2e-playwright`, `make e2e-real` | `Makefile` | 새 네이티브 앱 도입 시 검증 체계 재정의 필요 |
| `e2e-real` 의미 | 현재는 **macOS Flutter `integration_test`** 기반 실서버 루프 | `docs/operations/README.md`, `scripts/run-real-flutter-e2e.sh` | 이후 Android/iOS 실기기 검증으로 이름/역할을 분리해야 함 |
| Rust↔Flutter 브리지 | `paw-ffi` + `flutter_rust_bridge` 사용 중 | `paw-ffi/`, `paw-client/flutter_rust_bridge.yaml` | 전환 초기에 FRB를 바로 제거하지 말고, UniFFI와 병행 운영 기간 필요 |
| FFI 설정 드리프트 | `flutter_rust_bridge.yaml`에 절대경로 존재 | `paw-client/flutter_rust_bridge.yaml` | Phase 0에서 즉시 정리해야 재현 가능 빌드 확보 가능 |
| 모바일/웹/데스크톱 정책 | 웹/네이티브 세션 정책과 WS 상태 규칙이 이미 문서화되어 있음 | `docs/operations/README.md` | 새로운 네이티브 앱도 동일한 런타임 계약을 유지해야 함 |
| 보안 저장소 경계 | 토큰/키 저장은 현재 `flutter_secure_storage`에 의존 | `paw-client/lib/core/auth/token_storage.dart`, `paw-client/lib/core/crypto/key_storage_service.dart` | Android Keystore / iOS Keychain 이전을 초기에 못 박아야 함 |
| Push 서버 준비 상태 | push token 등록 API와 DB 테이블이 이미 존재 | `docs/api/openapi.yaml`, `paw-server/src/push/*`, `paw-server/migrations/20260101000015_push_tokens.sql` | 모바일 네이티브에서는 서버 재설계가 아니라 클라이언트 통합 작업이 중심 |
| 암호 모듈 상태 | `paw-ffi`는 `create_account/encrypt/decrypt`만 제공 | `paw-ffi/src/api.rs` | `paw-core` 초기 포팅 범위는 암호화 API부터 시작 가능 |
| MLS 상태 | `paw-crypto`에 OpenMLS POC가 이미 존재 | `paw-crypto/src/mls.rs` | "T8 이후 결정"이 아니라 기존 POC를 어떻게 흡수/격리할지 명시해야 함 |
| Flutter 내부 로직 분포 | DB/검색/HTTP/WS/Sync/Auth가 이미 Dart에 분산 구현 | `paw-client/lib/core/**`, `paw-client/lib/features/**` | `paw-core` 포팅 우선순위를 이 구조에 맞춰 잡는 것이 안전 |

### 핵심 판단

1. **모노레포는 유지**한다.
2. 모바일 네이티브 전환 중에도 **`paw-client/`는 당분간 유지**한다.
3. `paw-client/`를 곧바로 `paw-web/`으로 바꾸지 않는다. 현재 Desktop 코드와 테스트가 함께 있기 때문이다.
4. `paw-ffi/`는 초기에 삭제하지 않는다. 네이티브 앱이 실제로 `paw-core`를 사용하기 전까지는 Flutter 클라이언트의 암호 브리지 역할을 계속 맡긴다.
5. 공유 Rust 코어는 **UniFFI**, 기존 Flutter는 **FRB**를 사용해 **과도기 공존**을 허용한다.

---

## 목표 아키텍처

```text
paw/
├── paw-server/          # 기존 서버 유지
├── paw-proto/           # 기존 WS/streaming protocol 유지
├── paw-crypto/          # 기존 MLS 실험/검증 코드 (단계적으로 정리)
├── paw-ffi/             # Flutter용 레거시 브리지 (과도기 유지)
├── paw-core/            # 신규: 모바일 공통 Rust 코어
├── paw-android/         # 신규: Kotlin + Compose 앱
├── paw-ios/             # 신규: SwiftUI 앱
└── paw-client/          # 기존 Flutter 앱, 최종적으로 Web/Desktop 중심으로 축소
```

### 책임 분리

#### `paw-core`
- 암호화/키 생성
- 로컬 DB 및 검색
- REST API 클라이언트
- WebSocket 연결/재연결/동기화
- AI 스트리밍 조립
- 인증 상태 머신
- 이벤트 버스

#### `paw-android` / `paw-ios`
- 네이티브 UI/내비게이션
- 플랫폼 보안 저장소 연동
- 푸시 알림 / 백그라운드 처리
- 카메라/파일 피커/권한
- `paw-core` 이벤트를 UI 상태로 변환

#### `paw-client`
- 전환 기간 동안 기존 Flutter 앱 유지
- 최종적으로 Web/Desktop 클라이언트 역할 집중
- 필요 시 모바일 fallback 브랜치로 잠시 존치

---

## 비목표 (Non-goals)

이번 계획에 포함하지 않는 것:
- 서버 프로토콜 전면 개편
- Web/Desktop까지 네이티브로 재작성
- E2EE 프로토콜 재선정 자체
- 한 번에 Flutter 모바일 앱을 삭제하는 big-bang 전환

---

## 마이그레이션 원칙

1. **Big-bang 금지**: Flutter 모바일을 먼저 죽이지 않는다.
2. **동일한 런타임 계약 유지**: 인증 흐름, 세션 복원 정책, WS 상태 정의는 기존 문서와 동일해야 한다.
3. **문서와 스크립트를 같이 이동**: 경로/명령 변경 시 `Makefile`, `README`, `docs/operations/README.md`를 같은 단계에서 갱신한다.
4. **검증 명령이 먼저**: 새 Phase 시작 전에 어떤 명령이 완료 증거인지 먼저 정의한다.
5. **이름은 현실에 맞게**: Desktop이 남아있는 동안 `paw-web` 같은 이름으로 바꾸지 않는다.

---

## 단계별 실행 계획

## Phase 0. 저장소 정리 + 기준선 확보 (1주)

### 목표
마이그레이션 시작 전에 현재 Flutter/FRB/운영 문서를 재현 가능 상태로 정리하고, 이후 비교할 기준선(baseline)을 확보한다.

### 작업
- `paw-client/flutter_rust_bridge.yaml`의 절대경로 제거
  - 현재 `rust_output: /Users/joy/workspace/paw/paw-ffi/src/frb_generated.rs`
- 생성 파일/로컬 경로 의존성 점검
  - 예: `paw-client/ios/Flutter/Generated.xcconfig`의 로컬 절대경로
- `Makefile`와 `docs/operations/README.md`에서 현재 검증 체계 명시
  - `e2e-flutter` = 일반 Flutter integration flow
  - `e2e-playwright` = web smoke
  - `e2e-real` = 현재는 macOS Flutter real-server E2E
- README/문서 드리프트 목록 작성
  - `README.md`, `README_kr.md`, `docs/ARCHITECTURE.md`, `docs/PROJECT_SUMMARY.md`
- Flutter 클라이언트의 공유 로직 기준선 고정
  - 참조 소스:
    - `paw-client/lib/core/db/app_database.dart`
    - `paw-client/lib/core/http/api_client.dart`
    - `paw-client/lib/core/ws/ws_service.dart`
    - `paw-client/lib/core/sync/sync_service.dart`
    - `paw-client/lib/features/auth/providers/auth_provider.dart`
    - `paw-client/lib/features/chat/providers/chat_provider.dart`
    - `paw-client/lib/core/auth/token_storage.dart`
    - `paw-client/lib/core/crypto/key_storage_service.dart`
- Push 연동 기준선 고정
  - `docs/api/openapi.yaml`의 `/api/v1/push/register`, `/api/v1/push/unregister`
  - `paw-server/src/push/handlers.rs`, `paw-server/src/push/service.rs`

### 산출물
- 재현 가능한 FRB 설정
- 문서/스크립트/경로 드리프트 체크리스트
- 네이티브 전환 전 기준선 검증 결과

### 검증
```bash
git diff --check
make test
make e2e-flutter
make e2e-playwright
make e2e-real
```

---

## Phase 1. `paw-core` 스캐폴딩 + 빌드 체인 생성 (1주)

### 목표
모노레포에 `paw-core`를 추가하고, Android/iOS가 호출 가능한 최소 UniFFI 코어를 만든다.

### 작업

#### 1-1. Rust workspace 확장
- `Cargo.toml`
  - workspace member에 `paw-core` 추가
  - workspace dependency에 UniFFI 및 이후 필요한 네트워크/DB 의존성 추가
- 신규 파일
  - `paw-core/Cargo.toml`
  - `paw-core/src/lib.rs`
  - `paw-core/src/paw_core.udl`
  - `paw-core/build.rs`

#### 1-2. 최소 브리지 계약 정의
- no-op 또는 `ping()` 수준의 최소 API 노출
- Kotlin/Swift 바인딩 자동 생성 경로 고정
- 생성 산출물 커밋 정책 결정
  - 권장: 생성 바인딩은 커밋, 바이너리 산출물은 비커밋

#### 1-3. 빌드 스크립트 추가
- 신규 스크립트
  - `scripts/gen-ffi-bindings.sh`
  - `scripts/build-core-android.sh`
  - `scripts/build-core-ios.sh`
- `Makefile` 신규 타겟
  - `bindings`
  - `core-android`
  - `core-ios`

#### 1-4. 네이티브 앱 빈 프로젝트 생성
- `paw-android/` Compose skeleton
- `paw-ios/` SwiftUI skeleton
- 각 앱에서 `paw-core` no-op 호출 확인

### 주의
- 이 단계에서는 `paw-ffi`를 제거하지 않는다.
- Flutter 앱은 여전히 FRB 기반으로 동작한다.

### 검증
```bash
cargo check -p paw-core
make bindings
make core-android
make core-ios
```

---

## Phase 2. 공유 로직 포팅 1차: 암호화 + 저장 + 인증 상태 (2~3주)

### 목표
모바일 런타임의 핵심 상태를 `paw-core`로 옮긴다.

### 작업

#### 2-1. 암호화 포팅
- 기준 소스: `paw-ffi/src/api.rs`
- 신규 모듈
  - `paw-core/src/crypto/mod.rs`
  - `paw-core/src/crypto/keys.rs`
  - `paw-core/src/crypto/provider.rs`
- 포함 기능
  - `create_account`
  - `encrypt`
  - `decrypt`
  - `generate_ed25519_keypair`

#### 2-2. DB + 검색 포팅
- 기준 소스
  - `paw-client/lib/core/db/app_database.dart`
  - `paw-client/lib/core/db/daos/messages_dao.dart`
  - `paw-client/lib/core/db/daos/conversations_dao.dart`
  - `paw-client/lib/core/search/search_service.dart`
- 신규 모듈
  - `paw-core/src/db/**`
  - `paw-core/src/search/mod.rs`
- 방향
  - SQLCipher 사용
  - FTS5 검색 지원
  - DB를 source of truth로 유지

#### 2-3. 인증 상태 머신 포팅
- 기준 소스: `paw-client/lib/features/auth/providers/auth_provider.dart`
- 신규 모듈
  - `paw-core/src/auth/mod.rs`
  - `paw-core/src/auth/state_machine.rs`
  - `paw-core/src/auth/token_store.rs`
- 반드시 유지할 현재 상태/정책
  - `authMethodSelect → phoneInput → otpVerify → deviceName → usernameSetup → authenticated`
  - 세션 복원 실패 시 즉시 폐기
  - `discoverableByPhone`, `username` bootstrap 반영
- 플랫폼 구현 책임
  - Android: Keystore
  - iOS: Keychain

#### 2-4. 이벤트 버스 추가
- `paw-core/src/events/mod.rs`
- 네이티브 앱은 callback/stream을 통해 상태를 수신

### 검증
```bash
# 모듈 단위 회귀
cargo test -p paw-core

# 라이브 서버 스모크(Phase 2 안에 신규 추가 필수)
make local-stack
# 예: OTP 요청 -> verify -> register-device -> session restore 를 검증하는
# paw-core 전용 live-server smoke target 또는 integration test
```

추가 원칙:
- `auth::` 같은 필터 기반 테스트만으로 Phase 2를 종료하지 않는다. 필터는 0건 매치로도 통과할 수 있다.
- Phase 2 종료 전에는 **실제 `paw-server`를 상대로 OTP / device registration / session restore**를 검증하는 스모크가 반드시 있어야 한다.
- `paw-core`용 live-server smoke가 준비되기 전까지는 기존 `make e2e-real`도 회귀 가드로 계속 유지한다.

---

## Phase 3. 공유 로직 포팅 2차: HTTP + WebSocket + 동기화 + 스트리밍 (2주)

### 목표
실시간 메시징과 AI 스트리밍을 `paw-core`로 옮겨 모바일 네이티브 UI가 동일한 백엔드 계약을 사용하도록 만든다.

### 작업

#### 3-1. HTTP 클라이언트
- 기준 소스: `paw-client/lib/core/http/api_client.dart`
- 신규 모듈
  - `paw-core/src/http/mod.rs`
  - `paw-core/src/http/client.rs`
  - `paw-core/src/http/media.rs`
  - `paw-core/src/http/error.rs`
- 포함 포인트
  - 기존 auth / conversations / messages / users / keys 엔드포인트 포팅
  - push token 등록/해제 엔드포인트는 서버 재설계 없이 클라이언트 통합으로 처리

#### 3-2. WebSocket 및 재연결
- 기준 소스
  - `paw-client/lib/core/ws/ws_service.dart`
  - `paw-client/lib/core/ws/reconnection_manager.dart`
- 신규 모듈
  - `paw-core/src/ws/mod.rs`
  - `paw-core/src/ws/service.rs`
  - `paw-core/src/ws/reconnect.rs`
- 메시지 타입은 가능한 한 `paw-proto`를 직접 재사용해 Dart/Rust shape 드리프트를 줄인다.

#### 3-3. 동기화/스트리밍
- 기준 소스
  - `paw-client/lib/core/sync/sync_service.dart`
  - `paw-client/lib/features/chat/providers/chat_provider.dart`
- 신규 모듈
  - `paw-core/src/sync/mod.rs`
  - `paw-core/src/sync/engine.rs`
  - `paw-core/src/sync/streaming.rs`

#### 3-4. 최상위 런타임 조립
- `paw-core/src/core.rs`
- 초기화 순서 고정
  1. DB 열기
  2. 저장 토큰 복원
  3. 세션 검증
  4. WS 연결

### 검증
```bash
# 모듈/통합 회귀
cargo test -p paw-core

# 라이브 서버/실시간 스모크(Phase 3 안에 신규 추가 필수)
make local-stack
# 예: hello_ok -> sync -> gap-fill -> stream delta/complete 를 검증하는
# paw-core 전용 WS/sync live-server smoke target 또는 integration test
```

추가 원칙:
- `ws::`, `sync::` 같은 이름 필터만으로 Phase 3를 종료하지 않는다.
- Phase 3 종료 전에는 **실제 `paw-server`와 붙여 hello_ok / sync / gap-fill / streaming**을 검증하는 live-server smoke가 반드시 있어야 한다.

---

## Phase 4. Android 네이티브 앱 구현 (3~4주)

### 목표
Compose 기반 Android 앱이 `paw-core`를 사용해 Flutter 모바일과 동등한 핵심 기능을 제공하도록 만든다.

### 작업
- `paw-android/`에 앱 구조 수립
  - Application / DI / navigation / theme
- `PawCoreManager` 도입
  - UniFFI callback → `Flow`/`StateFlow` 변환
- 인증 화면 구현
- 대화 목록 / 채팅 / 검색 / 그룹 정보 구현
- 스트리밍 버블 / 타이핑 / 읽음 상태 구현
- FCM / 백그라운드 처리 연결

### 우선순위
1. 로그인/세션 복원
2. 대화 목록
3. 채팅 송수신
4. AI 스트리밍
5. 푸시/백그라운드
6. 프로필/설정/에이전트

### 검증
```bash
make core-android
cd paw-android && ./gradlew assembleDebug
```

검증 체크:
- 인증 전체 플로우
- 메시지 송수신
- 스트리밍 응답 누적
- 백그라운드 복귀 후 재연결
- 푸시 수신

---

## Phase 5. iOS 네이티브 앱 구현 (3~4주)

### 목표
SwiftUI 기반 iOS 앱이 Android와 동일한 코어 계약을 사용하도록 만든다.

### 작업
- `paw-ios/` 앱/테스트/xcframework 구조 수립
- `PawCoreManager.swift` 도입
  - UniFFI callback → `AsyncStream` 변환
- 인증 / 대화 목록 / 채팅 / 검색 / 설정 구현
- APNs / Notification Service Extension 연결
- 백그라운드 복호화 및 세션 복원 처리

### 검증
```bash
make core-ios
cd paw-ios && xcodebuild -scheme Paw -destination 'platform=iOS Simulator,name=iPhone 15'
```

검증 체크:
- Android와 동일한 인증/메시징 플로우
- iOS 16+ 호환성
- APNs 수신/복귀

---

## Phase 6. Flutter 클라이언트 축소: Web/Desktop 중심 재배치 (2~3주)

### 목표
모바일 기능을 네이티브 앱으로 넘긴 뒤, `paw-client/`를 Web/Desktop 중심 코드베이스로 정리한다.

### 중요한 수정
기존 초안의 **`paw-client/` → `paw-web/` 즉시 리네이밍은 보류**한다.

이유:
- 현재 `paw-client/`에는 `macos/`, `linux/`, `windows/`가 존재한다.
- `test/desktop_service_test.dart`, `lib/core/platform/desktop_service.dart` 등 Desktop 전용 코드가 이미 있다.
- 따라서 현 시점의 정확한 표현은 **Web/Desktop Flutter 클라이언트**이지 **Web 전용 앱**이 아니다.

### 작업
- 모바일 전용 Flutter 코드 제거 또는 조건부 비활성화
- `paw-client/`의 책임을 Web/Desktop으로 한정
- FRB 사용 영역 축소 또는 대체
  - Web/Desktop이 계속 Rust를 써야 한다면 `paw-core` WASM/별도 브리지 등 대체 경로를 먼저 확보
- 경로/명령 정리
  - `scripts/run-local-dev.sh`
  - `scripts/run-playwright-smoke.sh`
  - `scripts/run-real-web-e2e.sh`
  - `README.md`, `README_kr.md`, `docs/ARCHITECTURE.md`

### 이름 정책
- **권장**: 이 단계에서는 `paw-client/` 이름 유지
- **선택**: Desktop까지 포함하는 의미로 `paw-flutter/`로 후속 리네이밍
- **비권장**: Desktop이 남아있는데 `paw-web/`로 변경

### 검증
```bash
# Web 회귀
make e2e-playwright
./scripts/run-real-web-e2e.sh

# Desktop 회귀
cd paw-client && flutter test test/desktop_service_test.dart
cd paw-client && flutter build macos
```

추가 원칙:
- 이 단계의 범위가 계속 **Web/Desktop**이라면 Playwright만으로 종료하지 않는다.
- 최소한 macOS desktop build/test는 필수 gate로 유지하고, Linux/Windows를 계속 지원 범위에 둘 경우 해당 플랫폼도 CI에서 build 또는 smoke target을 추가한다.
- 만약 Linux/Windows 검증을 당장 유지하지 않을 계획이면, 이 단계에서 지원 범위를 명시적으로 축소해야 한다.

---

## Phase 7. 레거시 제거 + CI/문서 최종 정리 (1주)

### 목표
네이티브 모바일이 운영 가능한 수준에 도달한 뒤, 레거시 Flutter 모바일/FRB 경로를 정리한다.

### 작업

#### 7-1. 제거 조건
아래가 모두 충족되기 전에는 `paw-ffi/`를 삭제하지 않는다.
- Android 네이티브 앱 핵심 플로우 통과
- iOS 네이티브 앱 핵심 플로우 통과
- Web/Desktop Flutter 클라이언트가 `paw-ffi` 없이도 독립 실행 가능
- 운영 문서/CI/개발 스크립트가 새 구조 반영 완료

#### 7-2. 정리 대상
- `paw-ffi/` 삭제
- workspace member에서 `paw-ffi` 제거
- FRB 관련 build step 제거
- Flutter 모바일용 문서/스크립트 제거

#### 7-3. CI 재편
신규 workflow 예시:
- `core.yml` — `paw-core/**`, `paw-proto/**`
- `android.yml` — `paw-android/**`, `paw-core/**`
- `ios.yml` — `paw-ios/**`, `paw-core/**`
- `flutter.yml` — `paw-client/**` (Web/Desktop)
- `server.yml` — 기존 서버 유지

#### 7-4. 문서 최종 동기화
반드시 함께 수정:
- `README.md`
- `README_kr.md`
- `docs/ARCHITECTURE.md`
- `docs/PROJECT_SUMMARY.md`
- `docs/operations/README.md`

### 검증
```bash
cargo test --workspace
git diff --check
```

---

## 파일/디렉터리별 변경 맵

| 경로 | 변경 내용 |
|---|---|
| `Cargo.toml` | `paw-core` 추가, 최종적으로 `paw-ffi` 제거 |
| `Makefile` | native/core/web-desktop 검증 명령 추가 및 `e2e-real` 재정의 |
| `scripts/` | core 빌드/바인딩 스크립트 추가, Flutter web/desktop 실행 스크립트 정리 |
| `docs/operations/README.md` | 검증 명령의 현재 의미와 전환 후 의미를 모두 명시 |
| `README.md`, `README_kr.md` | 저장소 구조와 실행 방법 업데이트 |
| `docs/ARCHITECTURE.md`, `docs/PROJECT_SUMMARY.md` | Flutter 단일 클라이언트 가정 제거 |
| `paw-client/` | Web/Desktop 중심으로 축소 |
| `paw-ffi/` | 과도기 유지 후 제거 |
| `paw-core/` | 신규 공유 코어 |
| `paw-android/`, `paw-ios/` | 신규 네이티브 앱 |

---

## 주요 리스크와 대응

| 리스크 | 수준 | 대응 |
|---|---|---|
| UniFFI와 FRB 과도기 중복 유지 비용 | 높음 | `paw-ffi` 조기 삭제 금지, 단계적 축소 |
| `paw-web` 식 리네이밍으로 범위 왜곡 | 높음 | `paw-client` 유지 또는 `paw-flutter` 검토 |
| 모바일/웹/데스크톱 검증 체계 혼재 | 높음 | `e2e-real` 명칭과 운영 문서를 Phase 0/7에서 정리 |
| 보안 저장소/세션 복원 차이 | 중간 | `token_store`는 플랫폼 구현으로 분리 |
| DB/WS/스트리밍 로직 포팅 중 동작 차이 | 중간 | 기존 Dart 구현을 기준선으로 테스트 추가 |
| 문서 드리프트 | 중간 | README/ARCHITECTURE/PROJECT_SUMMARY/operations를 동일 PR에서 갱신 |
| MLS/OpenMLS 방향성 혼선 | 중간 | `paw-crypto`의 현 상태를 문서에 반영하고 `CryptoProvider`로 분리 |

---

## 단계 종료 기준 (Definition of Done)

### Phase 0 완료
- FRB 절대경로 제거
- 현재 검증 명령 의미가 문서화됨
- 기준선 테스트 1회 통과

### Phase 1 완료
- `paw-core`가 workspace에 추가됨
- Android/iOS에서 no-op UniFFI 호출 성공
- core 바인딩 생성 스크립트가 재현 가능함

### Phase 2 완료
- `paw-core` 암호화/DB/검색/인증 회귀 테스트 통과
- 실제 `paw-server`를 상대로 OTP / verify / register-device / session restore live-server smoke 통과
- 플랫폼 secure storage와 연결 설계 확정

### Phase 3 완료
- `paw-core` HTTP/WS/Sync/Streaming 회귀 테스트 통과
- 실제 `paw-server`를 상대로 hello_ok / sync / gap-fill / streaming live-server smoke 통과
- `paw-core` 초기화 순서가 고정되고 문서화됨

### Phase 4 완료
- Android에서 로그인/대화/송수신/스트리밍/푸시 검증 완료

### Phase 5 완료
- iOS에서 Android와 동등한 핵심 플로우 검증 완료

### Phase 6 완료
- Flutter 클라이언트가 Web/Desktop 중심 구조로 정리됨
- Playwright + `scripts/run-real-web-e2e.sh`가 독립 실행 가능함
- `desktop_service_test.dart`와 macOS desktop build가 필수 gate로 유지됨
- Linux/Windows를 계속 지원한다면 해당 desktop CI gate도 명시됨

### Phase 7 완료
- Flutter Web/Desktop이 더 이상 의존하지 않는 경우에 한해 `paw-ffi` 제거
- CI와 문서가 최종 구조와 일치

---

## 권장 즉시 다음 작업

1. Phase 0 작업으로 `paw-client/flutter_rust_bridge.yaml` 절대경로 제거
2. `docs/operations/README.md`에 `e2e-real`의 현재 의미와 향후 분리 계획 추가
3. `Cargo.toml` / `Makefile` 기준으로 `paw-core` 도입 diff 초안 작성
4. `paw-client`를 `paw-web`로 바꾸는 기존 가정은 폐기하고, 이름 정책을 별도 결정 항목으로 관리

---

## 참조 기준 파일

- `Cargo.toml`
- `Makefile`
- `docs/operations/README.md`
- `README.md`
- `README_kr.md`
- `docs/ARCHITECTURE.md`
- `docs/PROJECT_SUMMARY.md`
- `paw-client/flutter_rust_bridge.yaml`
- `paw-ffi/src/api.rs`
- `paw-crypto/src/mls.rs`
- `paw-client/lib/core/db/app_database.dart`
- `paw-client/lib/core/http/api_client.dart`
- `paw-client/lib/core/ws/ws_service.dart`
- `paw-client/lib/core/sync/sync_service.dart`
- `paw-client/lib/features/auth/providers/auth_provider.dart`
- `paw-client/lib/features/chat/providers/chat_provider.dart`
