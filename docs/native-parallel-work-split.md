# Native Parallel Work Split

이 문서는 Android/iOS 네이티브 앱을 **병렬로 개발**할 때의 작업 경계, 체크포인트, 완료 기준을 고정합니다.

관련 기준 문서:
- `docs/native-migration-plan.md`
- `docs/native-core-contract.md`
- `docs/native-platform-contract.md`
- `docs/native-design-mood.md`

## 현재 전제

아래 항목은 이미 준비된 것으로 간주합니다.

- `paw-core` runtime/view/event contract 정리 완료
- UniFFI surface 1차 고정 완료
- secure storage / push / lifecycle contract 정리 완료
- Android/iOS bootstrap/auth shell preview 준비 완료

즉, 이제부터는 **플랫폼별 실제 구현**을 병렬로 진행할 수 있습니다.

---

## Track 분리

### Track A — Android

소유 범위:
- `paw-android/**`

주요 작업:
1. `SecureTokenVault` → Android Keystore adapter
2. `DeviceKeyStore` → Android secure key storage
3. `PushRegistrar` → FCM register/unregister
4. real bootstrap wiring
   - stored token restore
   - lifecycle hint 반영
   - runtime snapshot 반영
5. auth flow shell → 실제 상태 전이 연결
   - phone input
   - otp verify
   - device name
   - username setup

완료 기준:
- `cd paw-android && ./gradlew :app:assembleDebug`
- preview 상태가 아니라 real bootstrap/auth state 전이가 UI에 반영됨

---

### Track B — iOS

소유 범위:
- `paw-ios/**`

주요 작업:
1. `SecureTokenVault` → Keychain adapter
2. `DeviceKeyStore` → Keychain / secure storage adapter
3. `PushRegistrar` → APNs register/unregister
4. real bootstrap wiring
   - stored token restore
   - lifecycle hint 반영
   - runtime snapshot 반영
5. auth flow shell → 실제 상태 전이 연결
   - phone input
   - otp verify
   - device name
   - username setup

완료 기준:
- `cd paw-ios && DEVELOPER_DIR=/Applications/Xcode.app/Contents/Developer xcodebuild -project Paw.xcodeproj -scheme Paw -destination 'platform=iOS Simulator,name=iPhone 17 Pro' test`
- preview 상태가 아니라 real bootstrap/auth state 전이가 UI에 반영됨

---

### Track C — Shared Core / Leader

소유 범위:
- `paw-core/**`
- `docs/**`
- 공용 스크립트 / bindings 관련 파일

주요 작업:
1. Android/iOS 구현에 필요한 공통 helper API 보강
2. contract 변경 시 Rust/UDL/generated bindings 동기화
3. lifecycle / push / secure storage contract drift 방지
4. 문서 유지

완료 기준:
- `cargo clippy -p paw-core --all-targets -- -D warnings`
- `cargo test -p paw-core`
- `cargo check -p paw-ffi`
- `make bindings`

---

## 체크포인트

### Checkpoint 1 — Secure storage
- Android: Keystore token vault + device key storage
- iOS: Keychain token vault + device key storage

### Checkpoint 2 — Auth flow
- Android: 실제 auth 상태 전이 연결
- iOS: 실제 auth 상태 전이 연결

### Checkpoint 3 — Push / lifecycle
- Android: FCM + lifecycle wiring
- iOS: APNs + lifecycle wiring

### Checkpoint 4 — Core refresh
- `paw-core` helper 보강
- UniFFI bindings 재생성
- contract 문서 갱신

---

## 충돌 방지 규칙

### 플랫폼 간 수정 금지
- Android track은 `paw-ios/**` 수정 금지
- iOS track은 `paw-android/**` 수정 금지

### 공통 contract 변경 규칙
공통 변경은 leader만 처리:
- `paw-core/**`
- `docs/native-*.md`
- `scripts/gen-ffi-bindings.sh`
- generated bindings 관련 파일

### contract 변경 시 필수 동기화
아래는 항상 함께 맞춘다:
1. Rust 타입
2. UDL
3. generated bindings
4. 문서

---

## 디자인 분위기 규칙

구현 구조는 플랫폼에 맞게 조정 가능하지만, 디자인의 분위기는 가능한 한 유지합니다.

기준 문서:
- `docs/native-design-mood.md`

핵심:
- dark messenger mood
- layered surfaces
- thin outlines
- subdued AI-first accent

즉, 정보 구조와 플랫폼 네이티브 패턴은 달라질 수 있어도,
색감 / 밀도 / 톤 / 표면 계층감은 양쪽이 같은 계열로 유지합니다.

---

## 실행용 명령 템플릿

아래 명령을 그대로 사용해도 됩니다.

```text
$autopilot $ralph $team docs/native-migration-plan.md 와 docs/native-parallel-work-split.md 를 실행 기준으로 삼아 Android/iOS 병렬 개발을 진행하세요. Track A는 paw-android/** 전담으로 Keystore + FCM + real bootstrap/auth wiring, Track B는 paw-ios/** 전담으로 Keychain + APNs + real bootstrap/auth wiring, Leader는 paw-core/** 와 docs/** 전담으로 공통 contract/helper/bindings 유지보수를 맡습니다. 플랫폼 간 상대 디렉토리 수정은 금지하고, 공통 contract 변경은 leader만 처리하세요. 각 checkpoint마다 build/test evidence를 남기고 다음 단계로 진행하세요.
```
