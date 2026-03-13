# paw-core Native Contract

`paw-core`는 Android/iOS가 병렬로 붙을 수 있도록 아래 3개 축을 안정된 계약면으로 제공합니다.

## 1. 공개 진입점

- `AuthStateMachine`
  - OTP 요청 / 검증
  - device 등록
  - session restore / logout
- `ApiClient`
  - auth / conversations / messages / users / keys
- `CoreRuntime`
  - bootstrap
  - WS hello 처리
  - sync / gap-fill / streaming 처리
  - `snapshot()`

## 2. UI가 기대할 상태 모델

### Auth
- `AuthStateView`
- `AuthStepView`

토큰 원문은 뷰에 노출하지 않고, 존재 여부만 boolean으로 제공합니다.

### Runtime
- `RuntimeSnapshot`
  - `connection`
  - `cursors`
  - `active_streams`

### Events
- `CoreEvent::AuthStateChanged`
- `CoreEvent::BootstrapProgress`
- `CoreEvent::ConnectionStateChanged`
- `CoreEvent::SyncRequested`
- `CoreEvent::AckRequested`
- `CoreEvent::MessagePersisted`
- `CoreEvent::StreamUpdated`
- `CoreEvent::StreamFinalized`

모든 이벤트 payload는 UUID/시간값을 플랫폼 친화적인 scalar(`String`, epoch millis)로 변환한 view 타입을 사용합니다.

## 3. 플랫폼 구현 원칙

- Android/iOS UI는 Rust 내부 타입 대신 **view/event 타입**에 의존
- secure storage / push / lifecycle 은 플랫폼 레이어 책임
- 현재 모바일 DB는 build unblock을 위해 `bundled sqlite`를 사용하며,
  SQLCipher 재도입은 secure-storage 설계와 함께 후속 phase에서 진행

세부 플랫폼 경계 계약:

- `docs/native-platform-contract.md`

## 디자인 분위기 가이드

디자인의 분위기 통일 기준은 아래 문서를 사용합니다.

- `docs/native-design-mood.md`

핵심 원칙:
- dark messenger mood
- layered surfaces
- thin outlines
- subdued AI-first accent
