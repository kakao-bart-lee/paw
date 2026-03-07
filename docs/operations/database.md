# 데이터베이스 관리 (Database Operations)

Paw 서버는 PostgreSQL 16을 사용하며, `sqlx`를 통해 마이그레이션을 관리합니다.

## 1. 마이그레이션 실행 (Running Migrations)

새로운 데이터베이스 스키마 변경 사항을 적용하려면 다음 명령어를 사용합니다.

```bash
# Makefile 사용 시
make migrate

# sqlx-cli 직접 사용 시
sqlx migrate run
```

## 2. 마이그레이션 생성 (Creating Migrations)

새로운 스키마 변경 파일을 생성하려면 다음 명령어를 사용합니다.

```bash
# Makefile 사용 시
make migrate-add name=add_new_table

# sqlx-cli 직접 사용 시
sqlx migrate add add_new_table
```

## 3. 마이그레이션 목록 (Migration List)

현재 프로젝트에 포함된 마이그레이션 파일 및 목적입니다:

1. `01_create_users`: 사용자 테이블 및 기본 인증 정보
2. `02_create_devices`: 사용자 기기 및 세션 관리
3. `03_create_otp_codes`: 2단계 인증용 OTP 코드
4. `04_create_conversations`: 대화방(1:1 및 그룹) 정보
5. `05_create_messages`: 메시지 본문 및 메타데이터
6. `06_create_media`: 업로드된 미디어 파일 정보
7. `07_create_read_receipts`: 메시지 읽음 확인 상태
8. `08_create_prekey_bundles`: Signal 프로토콜용 프리키 번들
9. `09_create_one_time_prekeys`: Signal 프로토콜용 일회용 프리키
10. `10_create_agent_tokens`: AI 에이전트 인증 토큰
11. `11_add_agent_avatar_revoked`: 에이전트 아바타 및 권한 취소 필드 추가
12. `12_group_chat_limits`: 그룹 채팅 인원 제한 설정
13. `13_conversation_agents`: 대화방 내 에이전트 할당 정보
14. `14_channels`: 공지용 채널 기능
15. `15_push_tokens`: 모바일 푸시 알림 토큰
16. `16_backups`: 사용자 데이터 백업 메타데이터
17. `17_agent_marketplace`: 에이전트 마켓플레이스 정보
18. `18_performance_indexes`: 쿼리 최적화를 위한 인덱스 추가
19. `19_moderation`: 콘텐츠 중재 및 신고 시스템

## 4. 백업 및 복구 (Backup & Restore)

### 데이터베이스 백업
`pg_dump`를 사용하여 데이터베이스 전체를 백업합니다.
```bash
pg_dump -U postgres -h localhost paw > paw_backup_$(date +%Y%m%d).sql
```

### 데이터베이스 복구
백업된 SQL 파일을 사용하여 데이터를 복구합니다.
```bash
psql -U postgres -h localhost paw < paw_backup_file.sql
```

## 5. 연결 풀링 (Connection Pooling)

Paw 서버는 `sqlx::PgPool`을 사용하여 효율적인 연결 관리를 수행합니다.
- **최대 연결 수**: 기본값은 10개이며, 환경에 따라 조정 가능합니다.
- **유휴 시간**: 일정 시간 사용되지 않는 연결은 자동으로 닫힙니다.

## 6. 롤백 (Rollback)

최근 적용된 마이그레이션을 취소해야 하는 경우 다음 명령어를 사용합니다.
```bash
sqlx migrate revert
```
> [!WARNING]
> `revert` 실행 시 해당 마이그레이션에서 생성된 데이터가 삭제될 수 있으므로 주의가 필요합니다.
