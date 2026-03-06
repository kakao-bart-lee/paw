use crate::db::DbPool;
use crate::keys::models::{KeyBundle, OneTimeKeyResponse, UploadKeysRequest};
use anyhow::Context;
use base64::{Engine as _, engine::general_purpose::STANDARD};
use sqlx::Row;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum KeysError {
    #[error("invalid_base64")]
    InvalidBase64,
    #[error("bundle_not_found")]
    BundleNotFound,
    #[error(transparent)]
    Internal(#[from] anyhow::Error),
}

pub async fn upload_keys(
    pool: &DbPool,
    user_id: Uuid,
    device_id: Option<Uuid>,
    req: UploadKeysRequest,
) -> Result<(), KeysError> {
    let identity_key = decode_base64(&req.identity_key)?;
    let signed_prekey = decode_base64(&req.signed_prekey)?;
    let signed_prekey_sig = decode_base64(&req.signed_prekey_sig)?;

    let decoded_prekeys = req
        .one_time_prekeys
        .into_iter()
        .map(|entry| {
            decode_base64(&entry.key)
                .map(|key| (entry.key_id, key))
                .map_err(|_| KeysError::InvalidBase64)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let mut tx = pool.begin().await.context("begin keys upload transaction")?;

    sqlx::query(
        "INSERT INTO prekey_bundles (user_id, device_id, identity_key, signed_prekey, signed_prekey_sig)
         VALUES ($1, $2, $3, $4, $5)
         ON CONFLICT (user_id, device_id)
         DO UPDATE SET
            identity_key = EXCLUDED.identity_key,
            signed_prekey = EXCLUDED.signed_prekey,
            signed_prekey_sig = EXCLUDED.signed_prekey_sig,
            updated_at = NOW()",
    )
    .bind(user_id)
    .bind(device_id)
    .bind(identity_key)
    .bind(signed_prekey)
    .bind(signed_prekey_sig)
    .execute(&mut *tx)
    .await
    .context("upsert prekey bundle")?;

    for (key_id, key) in decoded_prekeys {
        sqlx::query(
            "INSERT INTO one_time_prekeys (user_id, key_id, prekey)
             VALUES ($1, $2, $3)
             ON CONFLICT (user_id, key_id) DO NOTHING",
        )
        .bind(user_id)
        .bind(key_id)
        .bind(key)
        .execute(&mut *tx)
        .await
        .with_context(|| format!("insert one-time prekey key_id={key_id}"))?;
    }

    tx.commit().await.context("commit keys upload transaction")?;
    Ok(())
}

pub async fn get_key_bundle(pool: &DbPool, user_id: Uuid) -> Result<KeyBundle, KeysError> {
    let mut tx = pool.begin().await.context("begin key bundle transaction")?;

    let bundle_row = sqlx::query(
        "SELECT identity_key, signed_prekey, signed_prekey_sig
         FROM prekey_bundles
         WHERE user_id = $1
         ORDER BY updated_at DESC
         LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await
    .context("fetch prekey bundle")?
    .ok_or(KeysError::BundleNotFound)?;

    let identity_key: Vec<u8> = bundle_row.get("identity_key");
    let signed_prekey: Vec<u8> = bundle_row.get("signed_prekey");
    let signed_prekey_sig: Vec<u8> = bundle_row.get("signed_prekey_sig");

    let maybe_otk = sqlx::query(
        "SELECT id, key_id, prekey
         FROM one_time_prekeys
         WHERE user_id = $1 AND used = FALSE
         ORDER BY created_at ASC
         FOR UPDATE SKIP LOCKED
         LIMIT 1",
    )
    .bind(user_id)
    .fetch_optional(&mut *tx)
    .await
    .context("lock one-time prekey")?;

    let one_time_prekey = if let Some(row) = maybe_otk {
        let prekey_id: Uuid = row.get("id");
        let key_id: i64 = row.get("key_id");
        let prekey: Vec<u8> = row.get("prekey");

        sqlx::query("UPDATE one_time_prekeys SET used = TRUE WHERE id = $1")
            .bind(prekey_id)
            .execute(&mut *tx)
            .await
            .context("mark one-time prekey used")?;

        Some(OneTimeKeyResponse {
            key_id,
            key: STANDARD.encode(prekey),
        })
    } else {
        None
    };

    let remaining: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::BIGINT
         FROM one_time_prekeys
         WHERE user_id = $1 AND used = FALSE",
    )
    .bind(user_id)
    .fetch_one(&mut *tx)
    .await
    .context("count remaining one-time prekeys")?;

    tx.commit()
        .await
        .context("commit key bundle read transaction")?;

    Ok(KeyBundle {
        user_id,
        identity_key: STANDARD.encode(identity_key),
        signed_prekey: STANDARD.encode(signed_prekey),
        signed_prekey_sig: STANDARD.encode(signed_prekey_sig),
        one_time_prekey,
        replenish_prekeys: remaining < 5,
    })
}

fn decode_base64(value: &str) -> Result<Vec<u8>, KeysError> {
    STANDARD.decode(value).map_err(|_| KeysError::InvalidBase64)
}
