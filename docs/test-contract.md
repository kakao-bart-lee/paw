# Platform-Independent Test Contract

모든 네이티브 클라이언트(Android, iOS, Flutter)가 리팩토링 전후로 동일한 동작을 보장하기 위한 행위 기반 테스트 명세.

각 테스트는 **플랫폼 구현과 무관한 계약**이며, 플랫폼별 테스트 파일이 이 명세를 1:1로 구현해야 한다.

---

## TC-AUTH: Auth Flow State Machine

auth flow는 모든 플랫폼에서 동일한 상태 전이를 따른다.

### TC-AUTH-01: 초기 상태

- 저장된 토큰이 없을 때, 초기 auth step은 `AUTH_METHOD_SELECT`
- 에러 없음, 로딩 아님

### TC-AUTH-02: Phone → OTP 전이

- `AUTH_METHOD_SELECT` → phone input 시작 → `PHONE_INPUT`
- phone 제출 성공 → `OTP_VERIFY`로 전이
- phone 필드에 입력한 값이 auth state에 반영됨

### TC-AUTH-03: OTP → Device Name 전이

- OTP 검증 성공 → `DEVICE_NAME`으로 전이
- session token 획득 상태 반영

### TC-AUTH-04: Device Registration → Username or Authenticated

- device 등록 성공 → access/refresh token 저장
- 서버 응답에 username이 없으면 → `USERNAME_SETUP`
- 서버 응답에 username이 있으면 → `AUTHENTICATED`

### TC-AUTH-05: Username Setup → Authenticated

- username 제출 성공 → `AUTHENTICATED`
- username, discoverable_by_phone 값이 반영됨

### TC-AUTH-06: Username Skip

- username 건너뛰기 → `AUTHENTICATED`
- username은 빈 문자열 또는 기존값 유지
- access token은 유효한 상태

### TC-AUTH-07: 각 단계에서 빈 입력 검증

- phone 빈 입력 → 에러 메시지, 상태 전이 없음
- OTP 빈 입력 → 에러 메시지, 상태 전이 없음
- device name 빈 입력 → 에러 메시지, 상태 전이 없음
- username 빈 입력 → 에러 메시지, 상태 전이 없음

### TC-AUTH-08: 로딩 상태

- 비동기 auth 요청 중 `isLoading = true`
- 요청 완료 후 `isLoading = false`
- 로딩 중 에러 필드는 null

### TC-AUTH-09: Auth 에러 핸들링

- 네트워크 오류 시 에러 메시지 설정, 현재 step 유지
- 에러 후 재시도 가능

---

## TC-TOKEN: Token Vault

### TC-TOKEN-01: Round-trip (저장 → 읽기 → 삭제)

- 토큰 저장 → 동일 토큰 읽기 성공
- 토큰 삭제 → 읽기 시 null/empty

### TC-TOKEN-02: 덮어쓰기

- 기존 토큰 위에 새 토큰 저장 → 새 값 반환

### TC-TOKEN-03: 빈 상태 읽기

- 저장된 적 없는 상태에서 읽기 → null/empty

---

## TC-DEVICE-KEY: Device Key Store

### TC-DEVICE-KEY-01: 키 생성 및 저장

- 키가 없을 때 → `loadOrCreate()` → 새 키 생성 및 저장
- 이후 `load()` → 동일 키 반환

### TC-DEVICE-KEY-02: Round-trip

- 키 저장 → 존재 확인 true
- 키 삭제 → 존재 확인 false

---

## TC-PUSH: Push Registration

### TC-PUSH-01: 초기 상태

- 등록 전 상태 → `Unregistered`

### TC-PUSH-02: 등록 → 해제 사이클

- 등록 → 상태 `Registered`, 토큰 반영
- 해제 → 상태 `Unregistered`

### TC-PUSH-03: 플랫폼 태그

- Android: `fcm`
- iOS: `apns`

---

## TC-LIFECYCLE: Lifecycle Bridge

`docs/native-platform-contract.md`에 정의된 힌트 규칙과 일치해야 한다.

### TC-LIFECYCLE-01: Active 힌트

- Active/Launching 상태 → `["ReconnectSocket", "RefreshPushToken"]`

### TC-LIFECYCLE-02: Background 힌트

- Background 상태 → `["PauseRealtime", "FlushAcks", "PersistDrafts"]`

### TC-LIFECYCLE-03: Inactive 힌트

- Inactive 상태 → `["FlushAcks"]`

### TC-LIFECYCLE-04: Terminated 힌트

- Terminated 상태 → `["FlushAcks", "PersistDrafts"]`

---

## TC-CHAT: Chat Shell

### TC-CHAT-01: 인증 전 채팅 불가

- 인증 완료 전 메시지 전송 시도 → 에러 또는 빈 상태

### TC-CHAT-02: 대화 목록 로드

- 인증 완료 후 대화 목록 로드 성공
- 대화 항목에 id, name 존재

### TC-CHAT-03: 대화 선택 → 메시지 로드

- 대화 선택 시 해당 대화의 메시지 목록 로드
- selectedConversationId 반영

### TC-CHAT-04: 대화 선택 — 존재하는 ID 유지

- 이미 선택된 ID가 대화 목록에 있으면 유지

### TC-CHAT-05: 대화 선택 — 없는 ID는 첫 번째로 fallback

- 선택된 ID가 목록에 없으면 첫 번째 대화로 fallback
- 목록이 비어있으면 null

### TC-CHAT-06: 메시지 전송 — Optimistic Update

- 메시지 전송 시 즉시 messages 목록에 추가 (optimistic)
- `sendingMessage = true`
- draft 초기화

### TC-CHAT-07: 메시지 전송 성공 — 서버 확인으로 교체

- 서버 응답 도착 시 optimistic 메시지를 confirmed로 교체
- id, seq, createdAt 서버 값으로 갱신
- 대화 목록의 lastMessage 갱신

### TC-CHAT-08: 메시지 전송 실패 — Optimistic Rollback

- 전송 실패 시 optimistic 메시지 제거
- 에러 메시지 설정
- `sendingMessage = false`

### TC-CHAT-09: 빈 draft 전송 불가

- draft가 비어있거나 공백일 때 전송 → 에러 메시지, 전송하지 않음

### TC-CHAT-10: 대화 미선택 상태에서 전송 불가

- selectedConversationId가 null일 때 전송 → 에러 메시지

### TC-CHAT-11: Runtime Cursor 동기화

- 메시지가 있는 대화의 cursor는 최대 seq 값을 반영

---

## TC-SESSION: Session Restore

### TC-SESSION-01: 저장된 토큰으로 복원

- 유효한 access token이 저장된 상태에서 앱 시작
- → `AUTHENTICATED` (또는 username 없으면 `USERNAME_SETUP`)
- access token 기반으로 /me 호출 성공

### TC-SESSION-02: 저장된 토큰이 없을 때

- 토큰 없음 → `AUTH_METHOD_SELECT`

### TC-SESSION-03: 만료/무효 토큰 처리

- 저장된 토큰으로 /me 호출 실패
- → 토큰 삭제, `AUTH_METHOD_SELECT`로 복귀
- 에러 메시지 반영

---

## TC-LOGOUT: Logout

### TC-LOGOUT-01: 전체 상태 초기화

- 토큰 삭제 (vault clear)
- auth state → 초기 상태
- chat state → 빈 상태
- connection → Disconnected

### TC-LOGOUT-02: Push 해제

- logout 시 push unregister 호출

---

## TC-PHONE: Phone Normalization (Android/iOS)

### TC-PHONE-01: 빈 입력

- `""` → `""`
- `"  "` → `""`

### TC-PHONE-02: 이미 + prefix

- `"+821012345678"` → `"+821012345678"` (그대로)

### TC-PHONE-03: 82로 시작

- `"821012345678"` → `"+821012345678"`

### TC-PHONE-04: 0으로 시작

- `"01012345678"` → `"+8210 12345678"`

### TC-PHONE-05: 순수 숫자

- `"1012345678"` → `"+821012345678"`

---

## 커버리지 현황 참조

| 영역 | Android (현재) | iOS (현재) | Flutter (현재) |
|------|---------------|-----------|---------------|
| TC-AUTH | 없음 | 3개 (02,05,06) | 있음 |
| TC-TOKEN | 없음 | 1개 (01) | 있음 |
| TC-DEVICE-KEY | 없음 | 1개 (02) | N/A |
| TC-PUSH | 없음 | 1개 (02) | N/A |
| TC-LIFECYCLE | 없음 | 1개 (01,02) | N/A |
| TC-CHAT | 2개 (04,05) | 2개 (01,11) | 있음 |
| TC-SESSION | 없음 | 1개 (01) | 있음 |
| TC-LOGOUT | 없음 | 1개 (01,02) | 있음 |
| TC-PHONE | 없음 | 없음 | N/A |

---

## 구현 가이드

1. 각 테스트 ID(`TC-XXX-NN`)를 테스트 함수명에 포함시켜 추적 가능하게 할 것
2. 네트워크 의존 테스트는 mock/stub API 사용
3. 플랫폼별 secure storage는 in-memory 구현으로 대체
4. 테스트 간 상태 격리 보장 (setUp/tearDown)
