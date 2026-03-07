# Back Office 준비 문서 (Living Spec)

이 문서는 기능 구현과 동시에 백오피스(운영/CS/모니터링/Admin UI)에 필요한 요구사항을 누적 관리하기 위한 기준 문서입니다.

## 1. 목적

- 제품 기능 변경 시 백오피스 영향도를 빠르게 식별
- 운영/CS 관점에서 필요한 데이터, 액션, 감사 로그를 선제 정의
- 릴리즈 전 백오피스 준비 상태를 점검할 수 있는 체크리스트 제공

## 2. 운영 원칙

- 기능 PR/커밋 단위로 본 문서를 함께 갱신
- 새로운 API/상태머신/오류코드가 추가되면 `데이터 계약`과 `관측 이벤트`를 동시 업데이트
- 사용자 실패 시나리오(401, 네트워크, WS 끊김 등)는 백오피스 대응 항목에 반드시 반영

## 3. 데이터 계약 (Back Office 관점)

### 3.1 인증/세션
- 세션 상태: `authenticated | unauthenticated`
- 세션 만료 이유: `unauthorized` (401 기반)
- 토큰 정책
  - 웹: 자동 복원 스킵
  - 네이티브: 복원 후 `getMe` 실패 시 토큰 즉시 폐기

### 3.2 WebSocket 상태
- 클라이언트 상태: `connecting | connected | retrying | disconnected`
- 재연결 정책: 지수 백오프, 최대 시도 제한
- GAP 복구 규칙
  - `seq <= lastSeq`: 중복/지연 프레임 무시
  - `seq > lastSeq + 1`: `requestSync(lastSeq)` 강제

### 3.3 오류 코드 (UI 표준)
- `unauthorized`
- `forbidden`
- `server`
- `network`
- `timeout`
- `client`
- `unknown`

## 4. 백오피스 화면 요구사항 (초안)

### 4.1 세션/인증 대시보드
- 로그인 성공/실패 추이
- 401/403 비율
- 토큰 만료 후 재로그인 전환율

### 4.2 실시간 연결 대시보드
- WS 상태 전이 횟수/비율
- 재연결 시도 분포(시도 횟수, 평균 복구 시간)
- 대화별 GAP 복구 호출량

### 4.3 사용자 지원(CS) 화면
- `request_id`로 서버 로그/클라이언트 이벤트 추적
- 최근 오류 코드 히스토리
- 웹 비지원 기능(E2EE) 안내 노출 여부

## 5. 관측 이벤트 체크리스트

### 5.1 클라이언트
- `auth.login.*`
- `auth.session.restore.*`
- `auth.session.cleared`
- `ws.state.*`
- `sync.start`
- `sync.complete`

### 5.2 서버
- auth 실패 로그: `request_id`, `path`, `code`
- 요청 완료 로그: `request_id`, `method`, `path`, `status`, `latency_ms`

## 6. 릴리즈 게이트 (Back Office Readiness)

- 핵심 플로우 E2E 통과 (`make e2e-flutter`, `make e2e-playwright`)
- 콘솔 error/pageerror 0
- 401/WS 재연결/GAP 복구 실패 시 크래시 없음
- 백오피스 추적 필드(request_id, error code, ws state)가 문서와 구현에 일치

## 7. 변경 이력

- 2026-03-07: 초기 문서 생성
  - 인증/세션/WS/오류코드/관측 이벤트 기준 반영
  - 현재 구현 상태 기준으로 백오피스 요구사항 틀 정리
