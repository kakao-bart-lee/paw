# 모니터링 및 관측성 (Monitoring & Observability)

Paw 서버의 상태를 감시하고 문제를 진단하는 방법을 설명합니다.

## 1. 상태 확인 (Health Check)

서버의 생존 여부를 확인하기 위한 엔드포인트입니다.

- **Endpoint**: `GET /health`
- **Success Response**: `200 OK`
- **Usage**: 로드 밸런서(L4/L7)나 Kubernetes/Fly.io의 Liveness Probe로 사용합니다.

```bash
curl -f http://localhost:38173/health
```

## 2. 로깅 (Logging)

Paw 서버는 `tracing` 크레이트를 사용하여 구조화된 로그를 생성합니다.

### 로그 레벨 설정
`RUST_LOG` 환경 변수를 통해 로그 상세도를 조절합니다.
- `error`: 심각한 오류
- `warn`: 잠재적 문제 (예: NATS 연결 실패)
- `info`: 주요 이벤트 (기본값)
- `debug`: 개발 및 디버깅용 상세 정보

```bash
export RUST_LOG="paw_server=debug,tower_http=info"
```

### 로그 확인 명령어
```bash
# Docker Compose 로그 확인
docker compose logs -f paw-server

# 특정 문자열 검색
docker compose logs paw-server | grep "ERROR"

# request_id 기반 인증 실패 추적
docker compose logs paw-server | grep "auth failed"
```

### 요청 ID / 인증 실패 표준
- 서버는 모든 HTTP 응답에 `x-request-id` 헤더를 부여합니다.
- 인증 미들웨어 실패 로그는 `request_id`, `path`, `code`를 공통 필드로 기록합니다.
- 401 응답 본문에도 `request_id`를 포함해 클라이언트/서버 로그를 상호 추적할 수 있습니다.

## 3. 인프라 모니터링 (Infrastructure)

### NATS 모니터링
개발 환경의 Docker Compose에서는 NATS 모니터링 포트가 활성화되어 있습니다.
- **Monitoring URL**: `http://localhost:38223`
- **주요 지표**: 연결된 클라이언트 수, 메시지 처리량, 메모리 사용량

### 데이터베이스 연결 풀
`sqlx`의 연결 풀 상태를 로그를 통해 모니터링할 수 있습니다. 연결 획득 대기 시간이 길어지면 `DATABASE_URL`의 풀 사이즈 조정을 검토하세요.

## 4. 주요 관찰 지표 (Key Metrics)

운영 시 다음 지표들을 중점적으로 모니터링하는 것을 권장합니다:

1. **WebSocket 연결 수**: 현재 서버에 접속 중인 실시간 사용자 수
2. **메시지 처리량 (Throughput)**: 초당 전송/수신되는 메시지 수
3. **DB 쿼리 지연 시간**: 데이터베이스 응답 속도
4. **메모리 및 CPU 사용량**: 특히 미디어 처리 시의 리소스 변화
5. **인증 실패율(401/403)**: 토큰 만료 폭증, 클라이언트 세션 정책 문제 조기 탐지
6. **클라이언트 WS 상태 전이 빈도**: `connecting → connected → retrying → disconnected` 패턴 이상 감지

## 5. 클라이언트 핵심 이벤트

클라이언트는 다음 이벤트를 구조화 로그(`paw.client`)로 남깁니다.

- 로그인 성공/실패 (`auth.login.*`)
- 세션 복원 성공/실패 (`auth.session.restore.*`)
- WS 상태 전이 (`ws.state.*`)
- Sync 시작/완료 (`sync.start`, `sync.complete`)

## 6. 알림 권장 사항 (Alerting)

다음 상황 발생 시 알림(Slack, Email 등)을 받도록 설정하는 것이 좋습니다:

- `/health` 엔드포인트 응답 실패
- 로그 내 `ERROR` 레벨 발생 빈도 급증
- DB 연결 풀 고갈 (Connection pool exhausted)
- 디스크 공간 부족 (특히 미디어 저장소 및 로그 파일)
