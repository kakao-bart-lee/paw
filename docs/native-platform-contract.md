# Native Platform Contract

모바일 앱이 병렬로 개발되기 전에, 플랫폼 의존 기능은 아래 계약으로 고정합니다.

## 1. Secure storage

`paw-core::platform`

- `SecureStorageCapabilities`
- `SecureStorageProvider`
- `SecureStorageAvailability`
- `SecureTokenVault`
- `DeviceKeyStore`
- `DeviceKeyMaterial`

### 역할 분리

- 토큰 저장: `SecureTokenVault`
- 디바이스/E2EE 키 저장: `DeviceKeyStore`

### 현재 원칙

- Android: Keystore-backed secure storage
- iOS: Keychain-backed secure storage
- fallback 허용 시에도 `SecureStorageCapabilities::memory_fallback()`로 명시
- SQLCipher 재도입 전까지는 모바일 DB 암호화와 secure storage를 별도 slice로 본다

## 2. Push registration

- `PushTokenRegistration`
- `PushRegistrationState`
- `PushPlatform`
- `PushRegistrar`

서버 기준 계약:
- `POST /api/v1/push/register`
- `DELETE /api/v1/push/register`

현재 서버는 body에서 아래만 기대합니다.
- `token`
- `platform` (`fcm`, `apns`)

`device_id`는 access token에서 파생됩니다.

## 3. Lifecycle bridge

- `LifecycleEvent`
- `LifecycleState`
- `LifecycleHint`
- `LifecycleBridge`

기본 힌트 규칙:
- `Launching` / `Active`
  - `ReconnectSocket`
  - `RefreshPushToken`
- `Inactive`
  - `FlushAcks`
- `Background`
  - `PauseRealtime`
  - `FlushAcks`
  - `PersistDrafts`
- `Terminated`
  - `FlushAcks`
  - `PersistDrafts`

이 규칙은 Android/iOS가 각각 lifecycle API는 다르게 쓰더라도,
`paw-core`와 맞물리는 런타임 기대값은 같게 유지하기 위한 기준입니다.
