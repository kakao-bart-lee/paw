use crate::db::DbPool;
use crate::media::service::MediaService;
use std::time::Duration;
use uuid::Uuid;

use super::models::{BackupEntry, BackupFrequency, BackupSettings, InitiateBackupResponse};

const PRESIGNED_URL_EXPIRY_SECS: u64 = 3600;

pub async fn initiate_backup(
    db: &DbPool,
    media: &MediaService,
    user_id: Uuid,
) -> Result<InitiateBackupResponse, anyhow::Error> {
    let backup_id = Uuid::new_v4();
    let s3_key = format!("backups/{}/{}.enc", user_id, backup_id);

    let upload_url = media
        .presigned_put_url(&s3_key, Duration::from_secs(PRESIGNED_URL_EXPIRY_SECS))
        .await?;

    sqlx::query(
        "INSERT INTO backups (id, user_id, s3_key) VALUES ($1, $2, $3)",
    )
    .bind(backup_id)
    .bind(user_id)
    .bind(&s3_key)
    .execute(db.as_ref())
    .await?;

    Ok(InitiateBackupResponse {
        backup_id,
        upload_url,
        s3_key,
    })
}

pub async fn list_backups(
    db: &DbPool,
    user_id: Uuid,
) -> Result<Vec<BackupEntry>, anyhow::Error> {
    let rows = sqlx::query_as::<_, (Uuid, i64, chrono::DateTime<chrono::Utc>)>(
        "SELECT id, size_bytes, created_at FROM backups WHERE user_id = $1 ORDER BY created_at DESC",
    )
    .bind(user_id)
    .fetch_all(db.as_ref())
    .await?;

    Ok(rows
        .into_iter()
        .map(|(id, size_bytes, created_at)| BackupEntry {
            id,
            size_bytes,
            created_at,
        })
        .collect())
}

pub async fn restore_backup(
    db: &DbPool,
    media: &MediaService,
    user_id: Uuid,
    backup_id: Uuid,
) -> Result<Option<String>, anyhow::Error> {
    let s3_key = sqlx::query_scalar::<_, String>(
        "SELECT s3_key FROM backups WHERE id = $1 AND user_id = $2",
    )
    .bind(backup_id)
    .bind(user_id)
    .fetch_optional(db.as_ref())
    .await?;

    let s3_key = match s3_key {
        Some(k) => k,
        None => return Ok(None),
    };

    let download_url = media
        .presigned_url(&s3_key, Duration::from_secs(PRESIGNED_URL_EXPIRY_SECS))
        .await?;

    Ok(Some(download_url))
}

pub async fn delete_backup(
    db: &DbPool,
    media: &MediaService,
    user_id: Uuid,
    backup_id: Uuid,
) -> Result<bool, anyhow::Error> {
    let s3_key = sqlx::query_scalar::<_, String>(
        "SELECT s3_key FROM backups WHERE id = $1 AND user_id = $2",
    )
    .bind(backup_id)
    .bind(user_id)
    .fetch_optional(db.as_ref())
    .await?;

    let s3_key = match s3_key {
        Some(k) => k,
        None => return Ok(false),
    };

    media.delete_object(&s3_key).await?;

    sqlx::query("DELETE FROM backups WHERE id = $1 AND user_id = $2")
        .bind(backup_id)
        .bind(user_id)
        .execute(db.as_ref())
        .await?;

    Ok(true)
}

pub async fn get_backup_settings(
    db: &DbPool,
    user_id: Uuid,
) -> Result<BackupSettings, anyhow::Error> {
    let frequency = sqlx::query_scalar::<_, String>(
        "SELECT frequency FROM backup_settings WHERE user_id = $1",
    )
    .bind(user_id)
    .fetch_optional(db.as_ref())
    .await?;

    let frequency = match frequency.as_deref() {
        Some("daily") => BackupFrequency::Daily,
        Some("weekly") => BackupFrequency::Weekly,
        _ => BackupFrequency::Never,
    };

    Ok(BackupSettings { frequency })
}

pub async fn update_backup_settings(
    db: &DbPool,
    user_id: Uuid,
    settings: &BackupSettings,
) -> Result<(), anyhow::Error> {
    sqlx::query(
        "INSERT INTO backup_settings (user_id, frequency, updated_at) \
         VALUES ($1, $2, now()) \
         ON CONFLICT (user_id) DO UPDATE SET frequency = $2, updated_at = now()",
    )
    .bind(user_id)
    .bind(settings.frequency.as_str())
    .execute(db.as_ref())
    .await?;

    Ok(())
}
