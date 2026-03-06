use axum::{Json, extract::State};
use chrono::Utc;
use serde::Deserialize;
use serde_json::{Value, json};
use sqlx::Row;
use uuid::Uuid;

use super::{AppState, device, jwt, otp};

#[derive(Debug, Deserialize)]
pub struct RequestOtpRequest {
    pub phone: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyOtpRequest {
    pub phone: String,
    pub code: String,
}

#[derive(Debug, Deserialize)]
pub struct RegisterDeviceRequest {
    pub session_token: String,
    pub device_name: String,
    pub ed25519_public_key: String,
}

#[derive(Debug, Deserialize)]
pub struct RefreshTokenRequest {
    pub refresh_token: String,
}

fn error_json(code: &str, message: &str) -> Json<Value> {
    Json(json!({
        "error": code,
        "message": message,
    }))
}

fn valid_phone(phone: &str) -> bool {
    phone.starts_with('+') && phone.len() >= 8
}

pub async fn request_otp(
    State(state): State<AppState>,
    Json(payload): Json<RequestOtpRequest>,
) -> Json<Value> {
    if !valid_phone(&payload.phone) {
        return error_json("invalid_phone", "Phone number must be E.164 format");
    }

    let code = otp::generate_otp();
    let expires_at = otp::otp_expires_at();

    let insert_result = sqlx::query(
        "INSERT INTO otp_codes (phone, code, expires_at) VALUES ($1, $2, $3)",
    )
    .bind(&payload.phone)
    .bind(&code)
    .bind(expires_at)
    .execute(state.db.as_ref())
    .await;

    if insert_result.is_err() {
        return error_json("otp_store_failed", "Failed to create OTP");
    }

    tracing::info!(phone = %payload.phone, otp_code = %code, "Generated OTP (Phase 1: console log)");

    Json(json!({ "ok": true }))
}

pub async fn verify_otp(
    State(state): State<AppState>,
    Json(payload): Json<VerifyOtpRequest>,
) -> Json<Value> {
    if !valid_phone(&payload.phone) {
        return error_json("invalid_phone", "Phone number must be E.164 format");
    }

    if payload.code.len() != 6 || !payload.code.chars().all(|c| c.is_ascii_digit()) {
        return error_json("invalid_code_format", "OTP must be a 6-digit code");
    }

    let otp_row = sqlx::query(
        "SELECT id, expires_at, used_at \
         FROM otp_codes \
         WHERE phone = $1 AND code = $2 \
         ORDER BY created_at DESC \
         LIMIT 1",
    )
    .bind(&payload.phone)
    .bind(&payload.code)
    .fetch_optional(state.db.as_ref())
    .await;

    let Some(otp_row) = (match otp_row {
        Ok(row) => row,
        Err(_) => return error_json("otp_query_failed", "Failed to verify OTP"),
    }) else {
        return error_json("invalid_otp", "Invalid or expired OTP");
    };

    let otp_id: Uuid = otp_row.get("id");
    let expires_at = otp_row.get::<chrono::DateTime<Utc>, _>("expires_at");
    let used_at = otp_row.get::<Option<chrono::DateTime<Utc>>, _>("used_at");

    if used_at.is_some() || expires_at <= Utc::now() {
        return error_json("invalid_otp", "Invalid or expired OTP");
    }

    let mark_used = sqlx::query("UPDATE otp_codes SET used_at = NOW() WHERE id = $1 AND used_at IS NULL")
        .bind(otp_id)
        .execute(state.db.as_ref())
        .await;

    match mark_used {
        Ok(result) if result.rows_affected() == 1 => {}
        _ => return error_json("otp_already_used", "OTP has already been used"),
    }

    let user_id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO users (phone, display_name) \
         VALUES ($1, '') \
         ON CONFLICT (phone) DO UPDATE SET updated_at = NOW() \
         RETURNING id",
    )
    .bind(&payload.phone)
    .fetch_one(state.db.as_ref())
    .await;

    let user_id = match user_id {
        Ok(id) => id,
        Err(_) => return error_json("user_upsert_failed", "Failed to create user"),
    };

    let session_token = match jwt::issue_session_token(user_id, &state.jwt_secret) {
        Ok(token) => token,
        Err(_) => return error_json("session_issue_failed", "Failed to create session token"),
    };

    Json(json!({
        "user_id": user_id,
        "session_token": session_token,
    }))
}

pub async fn register_device(
    State(state): State<AppState>,
    Json(payload): Json<RegisterDeviceRequest>,
) -> Json<Value> {
    if payload.device_name.trim().is_empty() {
        return error_json("invalid_device_name", "Device name is required");
    }

    let claims = match jwt::verify_token(
        &payload.session_token,
        &state.jwt_secret,
        Some(jwt::TOKEN_TYPE_SESSION),
    ) {
        Ok(claims) => claims,
        Err(_) => return error_json("invalid_session_token", "Session token is invalid"),
    };

    let ed25519_public_key = match device::decode_ed25519_public_key(&payload.ed25519_public_key) {
        Ok(key) => key,
        Err(message) => return error_json("invalid_device_key", &message),
    };

    let device_id = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO devices (user_id, device_name, ed25519_public_key, platform, last_active_at) \
         VALUES ($1, $2, $3, 'cli', NOW()) \
         RETURNING id",
    )
    .bind(claims.sub)
    .bind(payload.device_name.trim())
    .bind(ed25519_public_key)
    .fetch_one(state.db.as_ref())
    .await;

    let device_id = match device_id {
        Ok(id) => id,
        Err(_) => return error_json("device_register_failed", "Failed to register device"),
    };

    let access_token = match jwt::issue_access_token(claims.sub, Some(device_id), &state.jwt_secret) {
        Ok(token) => token,
        Err(_) => return error_json("access_issue_failed", "Failed to issue access token"),
    };

    let refresh_token =
        match jwt::issue_refresh_token(claims.sub, Some(device_id), &state.jwt_secret) {
            Ok(token) => token,
            Err(_) => return error_json("refresh_issue_failed", "Failed to issue refresh token"),
        };

    Json(json!({
        "access_token": access_token,
        "refresh_token": refresh_token,
    }))
}

pub async fn refresh_token(
    State(state): State<AppState>,
    Json(payload): Json<RefreshTokenRequest>,
) -> Json<Value> {
    let claims = match jwt::verify_token(
        &payload.refresh_token,
        &state.jwt_secret,
        Some(jwt::TOKEN_TYPE_REFRESH),
    ) {
        Ok(claims) => claims,
        Err(_) => return error_json("invalid_refresh_token", "Refresh token is invalid"),
    };

    let access_token = match jwt::issue_access_token(claims.sub, claims.device_id, &state.jwt_secret) {
        Ok(token) => token,
        Err(_) => return error_json("access_issue_failed", "Failed to issue access token"),
    };

    Json(json!({
        "access_token": access_token,
    }))
}
