# Paw: Flutter → Native 전환 계획

## Context

Paw는 E2EE 메시징 + AI 에이전트 스트리밍을 지원하는 크로스플랫폼 메신저입니다.
현재 Flutter 단일 코드베이스(~13.5K LOC, 105 파일)로 iOS/Android/Web/Desktop을 지원하고 있으나,
메시징 앱의 핵심인 백그라운드 처리, 푸시 알림, 플랫폼 네이티브 UX를 위해
**모바일(iOS/Android)만 네이티브로 전환**하고, **Web/Desktop은 Flutter를 별도 유지**합니다.

Telegram 모델을 참고하여 **공유 Rust 코어(paw-core) + 네이티브 UI** 구조를 채택합니다.

### 결정 사항
- **저장소**: 모노레포 유지 (paw-core, paw-android, paw-ios 추가)
- **Web/Desktop**: Flutter 코드 별도 분리하여 유지
- **iOS 최소 버전**: iOS 16+ (NavigationStack, SwiftUI 성숙 API)
- **Android 최소 버전**: API 28 / Android 9.0 (BiometricPrompt 네이티브)
- **브릿지**: UniFFI (Kotlin + Swift 바인딩 자동 생성)

---

## Phase 0: 프로젝트 기반 구축 (1주)

### 목표
paw-core crate 생성, Android/iOS 프로젝트 스캐폴딩, UniFFI 연동 검증

### 0-1. paw-core crate 생성

**생성할 파일:**
- `paw-core/Cargo.toml` — cdylib(Android) + staticlib(iOS) 타겟
- `paw-core/src/lib.rs` — `uniffi::include_scaffolding!("paw_core")` + 모듈 선언
- `paw-core/uniffi/paw_core.udl` — 최소 UDL (no-op 함수 1개)
- `paw-core/build.rs` — UniFFI scaffolding 빌드

**수정할 파일:**
- `Cargo.toml` (workspace root) — members에 `"paw-core"` 추가
- `Cargo.toml` (workspace dependencies) — uniffi, reqwest, tokio-tungstenite, rusqlite 추가

**paw-core/Cargo.toml 의존성:**
```toml
[dependencies]
uniffi = "0.28"
tokio = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
uuid = { workspace = true }
chrono = { workspace = true }
thiserror = { workspace = true }
# Phase 1에서 추가: rusqlite, x25519-dalek, aes-gcm, hkdf, sha2, rand, zeroize
# Phase 2에서 추가: reqwest
# Phase 3에서 추가: tokio-tungstenite

[dependencies.paw-proto]
path = "../paw-proto"

[lib]
crate-type = ["cdylib", "staticlib", "lib"]

[build-dependencies]
uniffi = { version = "0.28", features = ["build"] }
```

### 0-2. Android 프로젝트 스캐폴딩

**생성할 디렉토리:** `paw-android/`
```
paw-android/
├── build.gradle.kts          ← 프로젝트 루트
├── settings.gradle.kts
├── gradle.properties
├── app/
│   ├── build.gradle.kts      ← minSdk=28, Compose BOM, Hilt, UniFFI
│   └── src/main/
│       ├── AndroidManifest.xml
│       ├── kotlin/io/paw/android/
│       │   ├── PawApplication.kt    ← Hilt @HiltAndroidApp
│       │   └── MainActivity.kt      ← ComponentActivity (Compose entry)
│       └── jniLibs/                  ← cargo-ndk 빌드 산출물 링크
└── gradle/wrapper/
```

**기술 스택:**
- Jetpack Compose BOM (latest stable)
- Hilt (DI)
- Kotlin Coroutines + Flow
- Compose Navigation (type-safe routes)
- Coil 3 (이미지 로딩)
- Markwon (마크다운 렌더링)

### 0-3. iOS 프로젝트 스캐폴딩

**생성할 디렉토리:** `paw-ios/`
```
paw-ios/
├── Paw.xcodeproj/
├── Paw/
│   ├── PawApp.swift              ← @main SwiftUI App
│   ├── ContentView.swift
│   └── Info.plist
├── PawCore/                      ← UniFFI 생성 Swift 바인딩
│   └── PawCoreFFI.xcframework   ← 빌드 산출물
└── PawTests/
```

**기술 스택:**
- SwiftUI (iOS 16+)
- Swift Concurrency (async/await + AsyncStream)
- NavigationStack (type-safe navigation)
- Kingfisher (이미지 로딩)
- swift-markdown (마크다운 렌더링)

### 0-4. 빌드 스크립트

**생성할 파일:**
- `scripts/build-core-android.sh` — `cargo ndk -t arm64-v8a -t x86_64 -o paw-android/app/src/main/jniLibs build -p paw-core --release` + `uniffi-bindgen generate --language kotlin`로 Kotlin 바인딩 생성/배치
- `scripts/build-core-ios.sh` — `cargo build -p paw-core --target aarch64-apple-ios --release` + `cargo build -p paw-core --target aarch64-apple-ios-sim --release` (Intel CI 필요 시 `x86_64-apple-ios` 포함) + `uniffi-bindgen generate --language swift` + XCFramework 패키징
- `scripts/gen-ffi-bindings.sh` — UniFFI Kotlin/Swift 바인딩 생성 경로를 단일화하고, 생성 산출물 커밋 여부를 일관되게 관리

**수정할 파일:**
- `Makefile` — 다음 타겟 추가:
  - `make bindings` → `./scripts/gen-ffi-bindings.sh`
  - `make core-android` → `./scripts/build-core-android.sh`
  - `make core-ios` → `./scripts/build-core-ios.sh`
  - `make android` → bindings + core-android + `cd paw-android && ./gradlew assembleDebug`
  - `make ios` → bindings + core-ios + `cd paw-ios && xcodebuild`
- `paw-client/flutter_rust_bridge.yaml` — `/Users/joy/...` 절대경로 제거, 상대경로 또는 생성 스크립트 기반으로 치환
- `docs/operations/README.md` / `Makefile` — `e2e-real`이 현재 macOS Flutter integration_test 흐름인지, 이후 web/native 검증 체계에서 어떻게 재정의할지 명시

### 0-5. 검증
- [ ] `cargo check -p paw-core` 성공
- [ ] Android에서 UniFFI no-op 함수 호출 → 로그 출력 확인
- [ ] iOS 기기 + 시뮬레이터에서 UniFFI no-op 함수 호출 → 로그 출력 확인
- [ ] Kotlin/Swift 바인딩 생성이 로컬 절대경로 없이 재현 가능
- [ ] `e2e-real` 명칭/실행 경로 불일치 해소 또는 문서화 완료

---

## Phase 1: Rust 코어 — 암호화 + DB (3주)

### 목표
paw-ffi의 암호화 코드 이전, SQLCipher DB 구현, FTS5 검색 구현

### 1-1. 암호화 모듈

**생성할 파일:**
- `paw-core/src/crypto/mod.rs`
- `paw-core/src/crypto/keys.rs` — `paw-ffi/src/api.rs` 포팅
  - `create_account() -> AccountKeys`
  - `encrypt(their_pub_key, plaintext) -> Vec<u8>`
  - `decrypt(my_priv_key, ciphertext) -> Vec<u8>`
  - **신규**: `generate_ed25519_keypair() -> Ed25519KeyPair` (현재 Flutter에서 스텁)
- `paw-core/src/crypto/provider.rs` — `CryptoProvider` trait (향후 OpenMLS 전환 대비)

**참조 파일:**
- `paw-ffi/src/api.rs` — X25519-ECDH + HKDF-SHA256 + AES-256-GCM 구현
- `paw-crypto/src/mls.rs` — OpenMLS 구현 (T8 이후 결정)

**의존성 추가 (paw-core/Cargo.toml):**
- x25519-dalek 2.x (features: static_secrets, reusable_secrets)
- aes-gcm 0.10
- hkdf 0.12
- sha2 (workspace)
- ed25519-dalek 2.x
- rand 0.8
- zeroize 1.x

### 1-2. 데이터베이스 모듈

**생성할 파일:**
- `paw-core/src/db/mod.rs`
- `paw-core/src/db/schema.rs` — DDL 정의 + 마이그레이션
  - conversations_table: id(PK), name, avatar_url, last_seq(default 0), unread_count(default 0), updated_at
  - messages_table: id(PK), conversation_id, sender_id, content, format(default 'markdown'), seq, created_at, is_me, is_agent
  - 인덱스: messages_conv_seq (conversation_id, seq)
  - messages_fts FTS5 가상 테이블 + INSERT/DELETE/UPDATE 트리거 3개
  - 참조: `paw-client/lib/core/db/app_database.dart`
- `paw-core/src/db/cipher.rs` — SQLCipher 키 유도 (HKDF from device secret)
- `paw-core/src/db/messages.rs` — MessagesDao 포팅
- `paw-core/src/db/conversations.rs` — ConversationsDao 포팅

**의존성 추가:** rusqlite (features: bundled-sqlcipher, vtab)

### 1-3. 검색 모듈

**생성할 파일:**
- `paw-core/src/search/mod.rs`
  - `build_fts5_query(raw: &str) -> String`
  - `search(query: &str, limit: u32) -> Vec<SearchResult>`

### 1-4. 검증
- [ ] 암호화/복호화 라운드트립 테스트
- [ ] DB 스키마 생성 + FTS5 트리거 동작 테스트
- [ ] FTS5 검색 결과 정렬 테스트
- [ ] UniFFI를 통해 Android/iOS에서 `create_account()` 호출 확인

---

## Phase 2: Rust 코어 — HTTP + 인증 (2주)

### 목표
REST API 클라이언트 구현, 인증 상태 머신 구현, 이벤트 버스 구축

### 2-1. HTTP 클라이언트

**생성할 파일:**
- `paw-core/src/http/mod.rs`
- `paw-core/src/http/client.rs` — 전체 API 엔드포인트 포팅
  - 참조: `paw-client/lib/core/http/api_client.dart`
  - reqwest::Client, 15초 타임아웃, Bearer 토큰 주입
  - 401 응답 시 SessionExpired 이벤트 emit
  - 엔드포인트 목록:
    - Auth: request_otp, verify_otp, register_device, refresh_token
    - Conversations: list, create, add_member, remove_member
    - Messages: send, get_messages (after_seq, limit)
    - Users: get_me, update_me, get_user_by_id, search_user
    - Keys: upload_key_bundle, get_key_bundle
- `paw-core/src/http/media.rs` — multipart 미디어 업로드
- `paw-core/src/http/error.rs` — ApiError enum (unauthorized, forbidden, server, network, timeout, client, unknown)

**의존성 추가:** reqwest (features: rustls-tls, multipart, json)

### 2-2. 인증 상태 머신

**생성할 파일:**
- `paw-core/src/auth/mod.rs`
- `paw-core/src/auth/state_machine.rs`
  - 참조: `paw-client/lib/features/auth/providers/auth_provider.dart`
  - 상태: PhoneInput → OtpVerify { phone } → DeviceName { phone, session_token } → Authenticated { access_token, refresh_token }
  - 전환: request_otp, verify_otp, register_device, restore_session, logout
  - register_device 시: 실제 Ed25519 키 생성 + 키 번들 업로드
  - 토큰 갱신: 401 수신 시 refresh → 실패 시 SessionExpired
- `paw-core/src/auth/token_store.rs` — `KeyValueStore` trait (플랫폼이 구현 제공)

### 2-3. 이벤트 버스

**생성할 파일:**
- `paw-core/src/events/mod.rs`
  - `tokio::sync::broadcast` 기반
  - PawEvent enum:
    - AuthStateChanged(AuthState)
    - ConversationsUpdated(Vec\<Conversation\>)
    - MessagesUpdated { conversation_id, messages }
    - StreamDelta { stream_id, conversation_id, delta }
    - StreamComplete { stream_id, message }
    - TypingChanged { conversation_id, user_ids }
    - ConnectionStateChanged(ConnectionState)
    - SessionExpired
  - UniFFI callback interface `EventHandler`로 네이티브에 전달

### 2-4. UDL 업데이트

**수정할 파일:** `paw-core/uniffi/paw_core.udl`
- PawCore interface에 인증 메서드 추가
- PawEvent enum + EventHandler callback 정의
- PawError enum 정의

### 2-5. 검증
- [ ] 실제 서버에 OTP 요청 → 검증 → 디바이스 등록 흐름 테스트
- [ ] 토큰 저장/복원 테스트
- [ ] 401 → refresh → 재시도 테스트
- [ ] EventHandler 콜백이 양 플랫폼에서 수신되는지 확인

---

## Phase 3: Rust 코어 — WebSocket + 동기화 (2주)

### 목표
실시간 WebSocket 통신, 재연결 로직, 메시지 동기화, AI 스트리밍 누적

### 3-1. WebSocket 서비스

**생성할 파일:**
- `paw-core/src/ws/mod.rs`
- `paw-core/src/ws/service.rs`
  - 참조: `paw-client/lib/core/ws/ws_service.dart`
  - tokio-tungstenite 기반
  - 상태 머신: Disconnected → Connecting → Connected → Retrying
  - 연결 시: ConnectMsg 전송 → HelloOk 수신 → SyncEngine.sync_all() 트리거
  - 메시지 전송: send_typing_start/stop, send_ack, request_sync
  - URI 구성: http→ws, https→wss 변환 + /ws?token= 경로
- `paw-core/src/ws/reconnect.rs`
  - 참조: `paw-client/lib/core/ws/reconnection_manager.dart`
  - 지수 백오프: [1, 2, 4, 8, 16, 30]초, 최대 10회
  - tokio::time::sleep 사용

**의존성 추가:** tokio-tungstenite (features: rustls-tls-native-roots)

### 3-2. 동기화 엔진

**생성할 파일:**
- `paw-core/src/sync/mod.rs`
- `paw-core/src/sync/engine.rs`
  - 참조: `paw-client/lib/core/sync/sync_service.dart` + `chat_provider.dart`
  - sync_all(): 모든 대화의 last_seq를 DB에서 조회 → request_sync 호출
  - handle_message_received(msg):
    - seq <= last_seq → 중복, ack만 전송
    - seq > last_seq + 1 → 갭, request_sync
    - seq == last_seq + 1 → DB upsert, MessagesUpdated emit, ack, last_seq 갱신
  - **중요**: DB가 primary source of truth (Flutter와 다른 점)
- `paw-core/src/sync/streaming.rs`
  - 참조: `paw-client/lib/features/chat/providers/chat_provider.dart`
  - HashMap\<stream_id, StreamBuffer\> 관리
  - handle_stream_start → 버퍼 생성
  - handle_content_delta → 버퍼 append + StreamDelta emit
  - handle_tool_start/end → 도구 실행 상태 추적
  - handle_stream_end → 최종 Message 조립 → DB upsert → StreamComplete emit

### 3-3. PawCore 최상위 객체 완성

**생성할 파일:**
- `paw-core/src/core.rs`
  - PawCore struct: tokio Runtime 소유, 모든 서비스 초기화
  - 초기화 순서: DB(SQLCipher) → 토큰 복원 → WS 연결
  - shutdown: runtime.shutdown_timeout(5s)
  - Send + Sync 보장 (tokio::sync::Mutex 사용)

### 3-4. 검증
- [ ] WS 연결 → HelloOk → sync_all 트리거 테스트
- [ ] 재연결 백오프 동작 테스트
- [ ] 갭 감지 → 재동기화 테스트
- [ ] AI 스트리밍 delta → 누적 → 최종 메시지 조립 테스트
- [ ] 양 플랫폼에서 실시간 메시지 수신 확인

---

## Phase 4: Android 네이티브 UI (4주)

### 목표
Jetpack Compose로 전체 화면 구현, PawCore 연동

### 4-1. 코어 연동 레이어

**생성할 파일:**
- `paw-android/app/src/main/kotlin/io/paw/android/core/PawCoreManager.kt`
  - @Singleton (Hilt)
  - EventHandler 구현 → PawEvent를 MutableSharedFlow\<PawEvent\>로 변환
  - Application.onCreate에서 초기화 (Keystore에서 DB secret 유도)
- `paw-android/app/src/main/kotlin/io/paw/android/core/di/CoreModule.kt`
  - Hilt @Module: PawCoreManager, PawCore 인스턴스 제공

### 4-2. 인증 화면

**생성할 파일:**
- `ui/auth/AuthViewModel.kt` — AuthStateChanged 수집 → StateFlow\<AuthUiState\>
- `ui/auth/PhoneInputScreen.kt` — 전화번호 입력
- `ui/auth/OtpVerifyScreen.kt` — OTP 6자리 입력
- `ui/auth/DeviceNameScreen.kt` — 디바이스 이름 등록

### 4-3. 대화 목록

**생성할 파일:**
- `ui/chat/ConversationsViewModel.kt` — ConversationsUpdated 수집
- `ui/chat/ConversationsScreen.kt` — LazyColumn + 필터 칩 (전체/보안/Agent/안 읽음)
- `ui/chat/CreateGroupScreen.kt` — 그룹 대화 생성

### 4-4. 채팅 화면 (핵심)

**생성할 파일:**
- `ui/chat/ChatViewModel.kt`
  - MessagesUpdated, StreamDelta, StreamComplete, TypingChanged 수집
  - MutableStateFlow\<List\<MessageUi\>\> + Map\<streamId, StreamingMessageUi\>
- `ui/chat/ChatScreen.kt` — 역순 LazyColumn
- `ui/chat/components/MessageBubble.kt` — 일반 메시지 버블
- `ui/chat/components/StreamBubble.kt` — AI 스트리밍 버블 (StateFlow\<String\> per delta)
- `ui/chat/components/MessageInput.kt` — 텍스트 입력 + 전송
- `ui/chat/components/TypingIndicator.kt`
- `ui/chat/components/ReadReceiptIndicator.kt`
- `ui/chat/components/ToolIndicator.kt` — AI 도구 실행 시각화
- `ui/chat/components/E2eeBanner.kt`
- `ui/chat/GroupInfoScreen.kt`
- `ui/chat/KeyVerificationScreen.kt`
- `ui/chat/SearchScreen.kt`

### 4-5. 기타 화면

**생성할 파일:**
- `ui/profile/MyProfileScreen.kt`
- `ui/profile/UserProfileScreen.kt`
- `ui/agent/AgentScreen.kt` — 에이전트 마켓플레이스
- `ui/settings/SettingsScreen.kt`

### 4-6. 디자인 시스템

**생성할 파일:**
- `ui/theme/PawTheme.kt` — app_theme.dart의 모든 색상/타이포그래피 이전
  - Primary: 0xFF63E6BE, Background: 0xFF0B1113, Surface1-4, Sent/Received/Agent 버블 색상
  - Typography: headlineLarge(30sp/w800) ~ labelSmall(11sp/w600)
- `ui/theme/PawColors.kt` — 색상 상수
- `ui/components/MessengerAvatar.kt` — 공통 아바타 컴포넌트

### 4-7. 네비게이션

**생성할 파일:**
- `ui/navigation/PawNavGraph.kt` — AuthGraph + MainGraph
- `ui/navigation/MainShell.kt` — BottomNavigation (채팅/에이전트/설정)

### 4-8. 푸시 알림

**생성할 파일:**
- `service/PawFirebaseMessagingService.kt` — FCM 토큰 등록, 메시지 수신 처리

### 4-9. 검증
- [ ] 인증 전체 플로우 (전화번호 → OTP → 디바이스 등록)
- [ ] 대화 목록 로드 + 실시간 업데이트
- [ ] 메시지 전송/수신 실시간 동작
- [ ] AI 스트리밍 응답이 delta별로 UI 업데이트
- [ ] 앱 백그라운드/포그라운드 전환 시 재연결
- [ ] FCM 푸시 알림 수신

---

## Phase 5: iOS 네이티브 UI (4주)

### 목표
SwiftUI로 전체 화면 구현, Android와 기능 동일

### 5-1. 코어 연동 레이어

**생성할 파일:**
- `Paw/Core/PawCoreManager.swift`
  - @MainActor 싱글톤, EventHandler 프로토콜 구현
  - PawEvent → AsyncStream\<PawEvent\> 변환
  - UIApplication 시작 시 초기화 (Keychain에서 DB secret 유도)

### 5-2. 화면 구현 (Android 4-2 ~ 4-5와 동일한 구조)

**생성할 파일:**
- `Features/Auth/` — AuthViewModel, PhoneInputView, OtpVerifyView, DeviceNameView
- `Features/Chat/` — ConversationsViewModel, ChatViewModel, ConversationsView, ChatView
- `Features/Chat/Components/` — MessageBubble, StreamBubble, MessageInput, TypingIndicator 등
- `Features/Profile/` — MyProfileView, UserProfileView
- `Features/Agent/` — AgentView
- `Features/Settings/` — SettingsView

### 5-3. 디자인 시스템

**생성할 파일:**
- `UI/Theme/PawTheme.swift` — Color extension으로 모든 디자인 토큰 정의
- `UI/Theme/PawTypography.swift` — Font extension
- `UI/Components/MessengerAvatar.swift`

### 5-4. 네비게이션

- NavigationStack + NavigationPath 기반
- TabView (채팅/에이전트/설정)

### 5-5. 푸시 알림

- APNs + UNUserNotificationCenter
- Notification Service Extension (E2EE 메시지 백그라운드 복호화)

### 5-6. 검증
- [ ] Android와 동일한 전체 플로우 테스트
- [ ] iOS 16 기기에서 호환성 확인

---

## Phase 6: Flutter Web 분리 + 기능 완성 (3주)

### 목표
Flutter 웹 코드 분리, 미완성 기능 완성, i18n

### 6-1. Flutter Web 분리

**수정할 파일:**
- `paw-client/` → `paw-web/`으로 리네이밍
- `paw-web/pubspec.yaml` — 네이티브 전용 패키지 제거 (flutter_secure_storage 등)
- `paw-web/lib/src/rust/` — web WASM 바인딩만 유지

**Makefile 업데이트:**
- `make web` → `cd paw-web && flutter run -d chrome`

### 6-2. E2EE 완성 (paw-core)

- [ ] Ed25519 실제 키 생성 (register_device 시)
- [ ] 키 번들 업로드 (register_device 완료 후)
- [ ] E2EE 대화 시 수신자 공개키 조회 → 암호화 → 전송
- [ ] 토큰 갱신 플로우 (401 → refresh → 재시도 → 실패 시 SessionExpired)

### 6-3. 미디어 지원

- [ ] paw-core/src/http/media.rs — multipart 업로드 구현
- [ ] 각 네이티브 앱에서 미디어 피커 → paw-core 업로드 연동

### 6-4. i18n

- Android: `res/values/strings.xml` (한국어), `res/values-en/strings.xml` (영어)
- iOS: `ko.lproj/Localizable.strings`, `en.lproj/Localizable.strings`
- 현재 하드코딩된 한국어 문자열 추출

### 6-5. 검증
- [ ] paw-web이 독립적으로 빌드/실행 가능
- [ ] E2EE 메시지 암호화/복호화 end-to-end 테스트
- [ ] 미디어 업로드/다운로드 테스트
- [ ] 한국어/영어 전환 테스트

---

## Phase 7: paw-ffi 퇴역 + CI 정비 (1주)

### 목표
Flutter 네이티브 코드 정리, CI/CD 파이프라인 구축

### 7-1. 정리

- [ ] Git 태그 `flutter-native-final` 생성 (전환 전 마지막 스냅샷)
- [ ] `paw-ffi/` 디렉토리 삭제 (paw-core로 흡수 완료)
- [ ] `Cargo.toml` workspace members에서 `"paw-ffi"` 제거
- [ ] `paw-web/`에서 paw-ffi 의존 제거 (web은 WASM 직접 사용)

### 7-2. CI/CD

**생성할 파일:** `.github/workflows/`
- `core.yml` — paw-core/** 또는 paw-proto/** 변경 시: cargo test, cargo clippy, UniFFI 바인딩 생성
- `android.yml` — paw-android/**, paw-core/**, 또는 paw-proto/** 변경 시: core 빌드 + Gradle assembleRelease
- `ios.yml` — paw-ios/**, paw-core/**, 또는 paw-proto/** 변경 시: core 빌드 + xcodebuild archive
- `web.yml` — paw-web/** 변경 시: flutter build web
- `server.yml` — paw-server/** 또는 paw-proto/** 변경 시 (기존 유지/수정)

### 7-3. Makefile 최종 업데이트

```makefile
# 기존 유지
make server          # cargo run -p paw-server
make test            # cargo test --workspace
make lint            # cargo clippy --workspace

# 신규
make core-android    # Rust → Android .so
make core-ios        # Rust → iOS .a + XCFramework
make android         # core-android + Gradle build
make ios             # core-ios + Xcode build
make web             # flutter build web (paw-web)
make dev-android     # 로컬 서버 + Android 에뮬레이터
make dev-ios         # 로컬 서버 + iOS 시뮬레이터
```

### 7-4. 검증
- [ ] `cargo test --workspace` 전체 통과
- [ ] `make android` 성공
- [ ] `make ios` 성공
- [ ] `make web` 성공
- [ ] CI 파이프라인 전체 그린

---

## 최종 저장소 구조

```
paw/
├── Cargo.toml                  ← workspace: paw-server, paw-proto, paw-crypto, paw-core
├── Makefile                    ← 통합 빌드 커맨드
├── paw-server/                 ← Axum 백엔드 (변경 없음)
├── paw-proto/                  ← 공유 WS 프로토콜 (서버 + 코어)
├── paw-crypto/                 ← OpenMLS (T8 결정 후 paw-core에 통합 가능)
├── paw-core/                   ← 신규: 공유 Rust 라이브러리
│   ├── Cargo.toml
│   ├── uniffi/paw_core.udl
│   ├── build.rs
│   └── src/
│       ├── lib.rs
│       ├── core.rs             ← PawCore (TDLib의 TdClient 역할)
│       ├── crypto/             ← 암호화 (X25519, AES-GCM, Ed25519)
│       ├── db/                 ← SQLCipher + FTS5
│       ├── http/               ← REST API 클라이언트
│       ├── ws/                 ← WebSocket + 재연결
│       ├── sync/               ← 동기화 엔진 + 스트리밍
│       ├── auth/               ← 인증 상태 머신
│       ├── search/             ← 전문 검색
│       └── events/             ← 이벤트 버스
├── paw-android/                ← 신규: Kotlin + Compose
│   ├── app/
│   └── gradle/
├── paw-ios/                    ← 신규: Swift + SwiftUI
│   ├── Paw/
│   └── PawCore/
├── paw-web/                    ← 기존 paw-client 리네이밍
├── adapters/
├── agents/
├── scripts/
│   ├── build-core-android.sh
│   ├── build-core-ios.sh
│   └── (기존 스크립트 유지)
├── deploy/
├── k6/
└── .github/workflows/
    ├── core.yml
    ├── android.yml
    ├── ios.yml
    ├── web.yml
    └── server.yml
```

---

## 리스크 & 대응

| # | 리스크 | 수준 | 대응 |
|---|--------|------|------|
| 1 | UniFFI async 안정성 (0.28) | 높음 | async 결과를 직접 반환하지 않고 이벤트로 emit. void 반환 + PawEvent 패턴 |
| 2 | E2EE 프로토콜 미확정 (T8) | 높음 | CryptoProvider trait 추상화, feature flag로 전환 |
| 3 | SQLCipher 번들 크기 (~8MB) | 중간 | R8/strip으로 최적화. 보안 앱이므로 허용 가능한 트레이드오프 |
| 4 | DB → primary source of truth | 중간 | Phase 1에서 DB-first 패턴 확립. 앱 시작→DB 로드→WS 델타 동기화 |
| 5 | paw-proto Dart 미러 드리프트 | 중간 | paw-web은 기존 Dart 미러 유지, 네이티브는 paw-proto 직접 사용으로 해결 |
| 6 | 한국어 하드코딩 | 낮음 | Phase 6에서 i18n 체계 도입 |

---

## 의존성 그래프 (Phase 순서)

```
Phase 0 (기반)
    │
    ├── Phase 1 (암호화 + DB) ──┐
    │                            │
    ├── Phase 2 (HTTP + 인증) ──┤
    │                            │
    └── Phase 3 (WS + 동기화) ──┤
                                 │
                    ┌────────────┤
                    │            │
              Phase 4 (Android)  Phase 5 (iOS)  ← 병렬 가능
                    │            │
                    └────────────┤
                                 │
                          Phase 6 (Web 분리 + 완성)
                                 │
                          Phase 7 (정리 + CI)
```
