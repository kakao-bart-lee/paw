# Hybrid Auth 롤아웃 체크리스트

username 기반 계정 식별과 선택적 전화번호 공개(`discoverable_by_phone`)를 함께 도입할 때 운영/QA가 확인할 항목입니다.

## 1. 배포 전 확인

- DB 스키마에 `username`, `discoverable_by_phone`가 추가되었는지 확인합니다.
- 기존 OTP 로그인 사용자는 회귀 없이 로그인/기기 등록이 가능한지 확인합니다.
- `phone`이 `NULL`인 사용자도 `/users/me` 응답과 프로필 화면에서 오류 없이 표시되는지 확인합니다.

## 2. API 검증

```bash
# 본인 프로필
curl -H "Authorization: Bearer $TOKEN" http://localhost:38173/users/me

# username 검색(기본 경로)
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:38173/users/search?username=paw_friend"

# 전화번호 검색(옵트인 사용자만 성공해야 함)
curl -H "Authorization: Bearer $TOKEN" \
  "http://localhost:38173/users/search?phone=%2B821012345678"
```

확인 포인트:

- `/users/me` 응답이 `phone: null` 또는 문자열을 모두 허용하는지
- `username`이 신규/기존 사용자 모두에 대해 일관되게 내려오는지
- `/users/search?username=...`가 공개 프로필(`id`, `username`, `display_name`, `avatar_url`)만 반환하는지
- `discoverable_by_phone=false` 사용자는 `/users/search`에서 조회되지 않는지
- OTP를 통해 로그인한 계정은 `phone_verified_at`이 채워져 opt-in 시에만 전화번호 검색이 가능한지

## 3. 클라이언트 회귀 검증

```bash
make e2e-flutter
make e2e-playwright
make e2e-real
```

추가 수동 확인:

- OTP 로그인 → 기기 등록 → 채팅 진입
- 프로필 화면에서 전화번호가 비어 있어도 UI가 깨지지 않는지
- 전화번호 공개를 끈 사용자 계정으로 친구 검색이 차단되는지

## 4. 관측 포인트

배포 직후 아래 로그/지표를 집중 확인합니다.

- `auth.login.request_otp.*`
- `auth.login.verify_otp.*`
- `auth.login.success`
- `auth.session.restore.*`
- 서버 `auth failed` 로그의 `code`, `path`, `request_id`
- `/users/search` 404/403 비율 급증 여부

## 5. 장애 시 롤백 판단

즉시 롤백 또는 feature-disable 검토 기준:

- OTP 로그인 성공률이 배포 전 대비 의미 있게 하락
- `/users/me` 응답 파싱 오류로 클라이언트 프로필/세션 복원이 실패
- 비공개 전화번호 사용자가 검색되는 개인정보 노출 징후 발견
