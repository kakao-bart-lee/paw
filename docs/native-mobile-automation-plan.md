# Native Mobile Automation Plan

이 문서는 `paw-android/`, `paw-ios/`, `paw-core/` 기준으로 **Android Emulator + iOS Simulator 자동화** 구조를 정리합니다.

관련 문서:
- `docs/native-migration-plan.md`
- `docs/native-core-contract.md`
- `docs/native-platform-contract.md`
- `docs/native-parallel-work-split.md`

---

## 목표

자동화는 3층으로 나눕니다.

1. **공통 코어 검증**
   - `paw-core` 단위 테스트 / contract 검증
2. **플랫폼별 네이티브 검증**
   - Android: build + unit + Compose/UI/instrumentation
   - iOS: build + XCTest + XCUITest
3. **공통 E2E 검증**
   - Android/iOS 동일 시나리오를 cross-platform 도구로 검증

---

## 권장 스택

### 1) Core layer

- `cargo test -p paw-core`
- `cargo clippy -p paw-core --all-targets -- -D warnings`
- `make bindings`

역할:
- 상태 머신
- runtime snapshot
- event/view contract
- secure storage / push / lifecycle contract

---

### 2) Android layer

권장:
- `./gradlew :app:assembleDebug`
- `./gradlew :app:testDebugUnitTest`
- `./gradlew :app:lintDebug`
- 이후 추가:
  - `./gradlew :app:connectedDebugAndroidTest`
  - Compose UI Test

자동화 대상:
- auth step 전이
- conversations list
- chat runtime shell
- optimistic send
- lifecycle resume/background 반응

현재 상태:
- emulator 실행 가능
- APK install / launch 가능
- adb screenshot 가능

즉, Android는 바로 다음 slice에서 **UI instrumentation 자동화**를 붙일 수 있습니다.

---

### 3) iOS layer

권장:
- `xcodebuild -project Paw.xcodeproj -scheme Paw -destination 'platform=iOS Simulator,name=iPhone 17 Pro' build`
- `xcodebuild ... test`
- 이후 추가:
  - XCUITest target

자동화 대상:
- auth bootstrap unlock
- conversations list
- chat runtime shell
- logout/relock
- lifecycle foreground/background 반응

현재 상태:
- simulator launch 가능
- app install / launch 가능
- screenshot 가능
- 다만 ad-hoc CLI 탭 입력은 현재 안정적이지 않음

즉, iOS는 **XCUITest를 붙이는 것이 가장 안정적인 다음 단계**입니다.

---

## 공통 E2E

권장 도구:
- Maestro

이유:
- Android/iOS 공통 시나리오를 하나의 flow 집합으로 관리 가능
- simulator/emulator 기반 smoke에 적합
- auth → conversations → chat → logout 흐름 자동화에 잘 맞음

권장 공통 시나리오:
1. 앱 시작
2. bootstrap/auth shell 확인
3. 로그인/인증 흐름
4. conversations list 진입
5. chat room 진입
6. 메시지 입력 / optimistic echo 확인
7. lifecycle background/foreground
8. logout

---

## CI 구조

### core
- Rust core build/test/clippy/bindings

### android
- build
- unit test
- lint
- emulator smoke

### ios
- build
- XCTest
- simulator smoke

### e2e-mobile
- Maestro cross-platform smoke

---

## 바로 다음 구현 순서

### Phase A
- Android Compose UI/instrumentation test 추가
- iOS XCTest 유지 + shell state assertions 보강

### Phase B
- iOS XCUITest target 추가
- auth/conversations/chat shell smoke 작성

### Phase C
- Maestro flow 추가
- Android/iOS 공통 smoke 정리

---

## 현재 실측 기준

확인 완료:
- Android Emulator:
  - app install/launch 성공
  - screenshot 성공
  - 실제 `AuthMethodSelect -> PhoneInput` 전이 확인
- iOS Simulator:
  - app launch 성공
  - screenshot 성공
  - conversations/chat runtime shell 화면 확인

주의:
- iOS는 현재 ad-hoc CLI 입력 자동화보다는 XCUITest/Maestro가 더 적합

---

## 추천 결론

가장 현실적인 다음 단계:

1. Android: Compose/UI instrumentation
2. iOS: XCUITest target
3. 공통: Maestro smoke

즉,
- **플랫폼별 테스트는 플랫폼 도구로**
- **공통 시나리오는 Maestro로**
- **비즈니스 로직은 paw-core에서 최대한 검증**

이 구조가 유지보수 비용과 신뢰도 균형이 가장 좋습니다.
