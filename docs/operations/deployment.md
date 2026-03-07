# 배포 가이드 (Deployment Guide)

Paw 메신저 서버를 다양한 환경에 배포하는 방법을 설명합니다.

## 1. 로컬 개발 환경 (Local Development)

로컬에서 서버를 실행하고 개발하는 가장 기본적인 방법입니다.

### 사전 준비
- Rust 및 Docker 설치
- `sqlx-cli` 설치: `cargo install sqlx-cli`

### 실행 단계
1. **인프라 실행**: Docker Compose를 사용하여 PostgreSQL, MinIO, NATS를 실행합니다.
   ```bash
   make docker-up
   ```
2. **데이터베이스 초기화**: 마이그레이션을 실행하여 테이블을 생성합니다.
   ```bash
   make migrate
   ```
3. **서버 실행**: `make dev` 명령어로 인프라 확인과 서버 실행을 동시에 수행합니다.
   ```bash
   make dev
   ```

## 2. Docker Compose (운영 환경)

운영 환경에서 Docker를 사용하여 전체 스택을 실행하는 방법입니다. `deploy/docker-compose.prod.yml` 파일을 사용합니다.

### 이미지 빌드
서버 이미지를 빌드합니다.
```bash
docker build -t paw-server:latest -f deploy/Dockerfile .
```

### 실행 방법
필요한 환경 변수를 설정한 후 Docker Compose를 실행합니다.
```bash
docker compose -f deploy/docker-compose.prod.yml up -d
```

### 주요 구성 요소
- **paw-server**: Rust 기반 메인 서버 (Port 3000)
- **postgres:15-alpine**: 데이터 저장소
- **nats**: 메시지 버스
- **minio**: S3 호환 오브젝트 스토리지

## 3. Fly.io 배포

Fly.io는 Paw 서버의 기본 클라우드 배포 타겟입니다. 설정은 `deploy/fly.toml`에 정의되어 있습니다.

### 초기 설정 및 비밀값 등록
배포 전 필수 환경 변수를 Fly.io Secrets로 등록해야 합니다.
```bash
fly secrets set JWT_SECRET="your-secure-secret" \
                DATABASE_URL="postgres://..." \
                S3_ACCESS_KEY="..." \
                S3_SECRET_KEY="..."
```

### 배포 실행
```bash
fly deploy --config deploy/fly.toml
```

### 스케일링 및 지역 설정
- **Region**: 기본 지역은 `nrt` (Tokyo)입니다.
- **Resources**: 512MB RAM, Shared CPU 설정을 권장합니다.
- **Auto-stop**: 트래픽이 없을 때 머신을 자동으로 정지하도록 설정되어 있습니다.

## 4. 배포 확인 및 검증 (Verification)

배포 후 서버가 정상적으로 작동하는지 확인합니다.

### 상태 확인 (Health Check)
서버의 `/health` 엔드포인트가 `200 OK`를 반환하는지 확인합니다.
```bash
curl -i http://localhost:3000/health
```

### 로그 확인
```bash
# Docker Compose
docker compose -f deploy/docker-compose.prod.yml logs -f paw-server

# Fly.io
fly logs
```

## 5. 롤백 절차 (Rollback)

문제가 발생했을 때 이전 버전으로 되돌리는 방법입니다.

### Docker Compose 롤백
이전 태그의 이미지를 사용하여 다시 실행합니다.
```bash
docker compose -f deploy/docker-compose.prod.yml up -d --force-recreate
```

### Fly.io 롤백
이전 성공적인 배포 버전으로 롤백합니다.
```bash
fly releases rollback
```
