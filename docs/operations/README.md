# 운영 및 배포 가이드 (Operations & Deployment)

이 디렉토리는 Paw 메신저 서버의 운영, 배포, 데이터베이스 관리 및 모니터링에 관한 문서를 포함하고 있습니다.

## 목차 (Table of Contents)

1. [배포 가이드 (Deployment Guide)](deployment.md) - 로컬, Docker, Fly.io 배포 방법
2. [환경 변수 설정 (Environment Variables)](environment-variables.md) - 서버 설정 및 보안 변수 목록
3. [데이터베이스 관리 (Database Operations)](database.md) - 마이그레이션 및 백업 절차
4. [모니터링 및 관측성 (Monitoring & Observability)](monitoring.md) - 상태 확인 및 로그 관리
5. [CDN 설정 (CDN Setup)](cdn-setup.md) - 미디어 캐싱 및 전송 최적화
6. [Back Office 준비 문서](../backoffice/README.md) - 운영/CS/Admin 대비 요구사항 관리
7. [Hybrid Auth 롤아웃 체크리스트](hybrid-auth-rollout.md) - username/optional phone/discoverability 검증 순서

운영 중 인증 이슈를 추적할 때는 `monitoring.md`의 `x-request-id`/`auth failed` 표준 로그 규칙을 우선 적용하세요.

## E2E 검증 실행

현재 클라이언트 검증 명령은 Flutter 기준 경로와 Web 경로가 함께 존재합니다. 모바일 네이티브 앱이 아직 도입되지 않았으므로, 아래 명령의 현재 의미를 고정해 둡니다.

1. Flutter 공식 Integration Test
```bash
make e2e-flutter
```

2. 웹 콘솔 안정성 Playwright smoke
```bash
make e2e-playwright
```

`e2e-playwright`는 Flutter web-server를 자동 기동한 뒤 `/login`, `/chat`, `/profile/me` 라우트 가드와 console/pageerror 0건을 검증합니다.

3. 실서버 full-loop E2E
```bash
make e2e-real
```

`make e2e-real`은 **현재는 `make e2e-real-flutter`의 별칭(alias)** 이며, 서버를 테스트 모드(OTP debug code 노출)로 올린 뒤 macOS Flutter `integration_test`를 실행합니다.

4. 실서버 Web full-loop E2E
```bash
make e2e-real-web
```

`e2e-real-web`은 서버와 Flutter web-server를 함께 기동한 뒤 Playwright real full-loop를 실행합니다.

> 참고: Android/iOS 네이티브 검증 명령은 `paw-core`, `paw-android`, `paw-ios`가 실제로 추가된 뒤 별도 타깃으로 도입합니다.

## 클라이언트 정책 스냅샷

- 웹 세션: 자동 복원을 건너뛰고 사용자가 명시적으로 로그인해야 합니다.
- 네이티브 세션: 저장 토큰 복원 후 `getMe` 검증이 실패하면 토큰을 즉시 폐기합니다.
- 보호 라우트: 비인증 상태에서 `/chat`, `/profile/me` 접근 시 `/login`으로 리다이렉트됩니다.
- WebSocket 상태: `connecting`, `connected`, `retrying`, `disconnected` 네 가지 상태만 사용합니다.
- `connected`는 소켓 생성 시점이 아니라 서버 `hello_ok` 수신 이후에만 성립합니다.

백오피스 영향 추적은 [backlog-map.md](../backoffice/backlog-map.md)에서 상태를 함께 관리합니다.

## 로컬 퀵스타트 (Local Quick-start)

로컬 개발 환경을 빠르게 시작하려면 다음 3개의 명령어를 실행하세요:

```bash
# 1. 인프라 스트럭처 실행 (PostgreSQL, MinIO, NATS)
make docker-up

# 2. 데이터베이스 마이그레이션 적용
make migrate

# 3. 개발 서버 실행
make dev
```

## 사전 요구 사항 (Prerequisites)

운영 및 관리를 위해 다음 도구들이 설치되어 있어야 합니다:

- **Rust**: 1.75 버전 이상 (Cargo 포함)
- **Docker & Docker Compose**: 컨테이너화된 서비스 실행용
- **sqlx-cli**: 데이터베이스 마이그레이션 관리용 (`cargo install sqlx-cli`)

## 관련 문서 (Related Documents)

- [메인 README.md](../../README.md) - 프로젝트 개요 및 개발 가이드
- [아키텍처 설계](../ARCHITECTURE.md) - 시스템 구조 및 설계 원칙
