# 환경 변수 설정 (Environment Variables)

Paw 서버의 동작을 제어하는 주요 환경 변수 목록입니다.

## 환경 변수 상세 (Variable Details)

### 핵심 설정 (Core)
| 변수명 | 필수 여부 | 기본값 | 설명 |
|:---|:---:|:---:|:---|
| `JWT_SECRET` | Yes | `dev_only_...` | JWT 서명에 사용되는 비밀키. 운영 환경에서 반드시 변경해야 함. |

> [!CAUTION]
> **보안 경고**: `JWT_SECRET`은 운영 환경에서 반드시 강력한 무작위 문자열로 변경해야 합니다. 기본값을 그대로 사용할 경우 인증 시스템이 취약해집니다.

### 데이터베이스 (Database)
| 변수명 | 필수 여부 | 기본값 | 설명 |
|:---|:---:|:---:|:---|
| `DATABASE_URL` | Yes | - | PostgreSQL 연결 문자열 (예: `postgres://user:pass@host:5432/db`) |

### 오브젝트 스토리지 (Storage - S3/R2)
| 변수명 | 필수 여부 | 기본값 | 설명 |
|:---|:---:|:---:|:---|
| `S3_ENDPOINT` | Yes | - | S3 호환 스토리지 엔드포인트 URL |
| `S3_BUCKET` | Yes | - | 미디어 파일을 저장할 버킷 이름 |
| `S3_ACCESS_KEY` | Yes | - | 스토리지 액세스 키 |
| `S3_SECRET_KEY` | Yes | - | 스토리지 비밀 키 |
| `S3_REGION` | Yes | - | 리전 정보 (Cloudflare R2의 경우 `auto`) |

### 메시징 (Messaging - NATS)
| 변수명 | 필수 여부 | 기본값 | 설명 |
|:---|:---:|:---:|:---|
| `NATS_URL` | No | `localhost:34223` | NATS 서버 주소. 연결 실패 시 서버는 경고를 남기고 계속 작동함. |

### 로깅 (Logging)
| 변수명 | 필수 여부 | 기본값 | 설명 |
|:---|:---:|:---:|:---|
| `RUST_LOG` | No | `info` | 로그 레벨 설정 (예: `paw_server=debug,info`) |

### E2E 테스트 (Test-only)
| 변수명 | 필수 여부 | 기본값 | 설명 |
|:---|:---:|:---:|:---|
| `PAW_EXPOSE_OTP_FOR_E2E` | No | `false` | `true/1`이면 `/auth/request-otp` 응답에 `debug_code`를 포함. **테스트 전용**으로만 사용해야 함. |

## 로컬 개발용 .env 예시 (.env Example)

로컬 개발 시 프로젝트 루트에 `.env` 파일을 생성하여 사용할 수 있습니다.

```env
# Core
JWT_SECRET=your_local_development_secret_key

# Database
DATABASE_URL=postgres://postgres:postgres@localhost:35432/paw

# Storage (Local MinIO)
S3_ENDPOINT=http://localhost:39080
S3_BUCKET=paw-media
S3_ACCESS_KEY=minioadmin
S3_SECRET_KEY=minioadmin
S3_REGION=us-east-1

# Messaging
NATS_URL=nats://localhost:34223

# Logging
RUST_LOG=paw_server=debug,tower_http=debug,info

# Test-only
PAW_EXPOSE_OTP_FOR_E2E=false
```
