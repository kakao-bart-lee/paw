use crate::db::DbPool;
use chrono::{DateTime, Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use uuid::Uuid;

pub const TOKEN_TYPE_ACCESS: &str = "access";
pub const TOKEN_TYPE_REFRESH: &str = "refresh";
pub const TOKEN_TYPE_SESSION: &str = "session";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub device_id: Option<Uuid>,
    pub token_type: String,
    pub exp: i64,
    pub iat: i64,
}

fn issue_token(
    user_id: Uuid,
    device_id: Option<Uuid>,
    token_type: &str,
    ttl: Duration,
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let claims = Claims {
        sub: user_id,
        device_id,
        token_type: token_type.to_string(),
        iat: now.timestamp(),
        exp: (now + ttl).timestamp(),
    };

    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

pub fn issue_session_token(
    user_id: Uuid,
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    issue_token(
        user_id,
        None,
        TOKEN_TYPE_SESSION,
        Duration::minutes(15),
        secret,
    )
}

pub fn issue_access_token(
    user_id: Uuid,
    device_id: Option<Uuid>,
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    issue_token(
        user_id,
        device_id,
        TOKEN_TYPE_ACCESS,
        Duration::days(7),
        secret,
    )
}

pub fn issue_refresh_token(
    user_id: Uuid,
    device_id: Option<Uuid>,
    secret: &str,
) -> Result<String, jsonwebtoken::errors::Error> {
    issue_token(
        user_id,
        device_id,
        TOKEN_TYPE_REFRESH,
        Duration::days(30),
        secret,
    )
}

pub fn verify_token(
    token: &str,
    secret: &str,
    expected_type: Option<&str>,
) -> Result<Claims, String> {
    let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = true;

    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )
    .map_err(|_| "invalid token".to_string())?;

    if let Some(expected) = expected_type {
        if data.claims.token_type != expected {
            return Err("invalid token type".to_string());
        }
    }

    Ok(data.claims)
}

pub fn is_token_issued_before_revocation(
    iat: i64,
    token_revoked_at: Option<DateTime<Utc>>,
) -> bool {
    token_revoked_at
        .map(|revoked_at| iat < revoked_at.timestamp())
        .unwrap_or(false)
}

pub async fn verify_token_with_revocation(
    token: &str,
    secret: &str,
    expected_type: Option<&str>,
    db: &DbPool,
) -> Result<Claims, String> {
    let claims = verify_token(token, secret, expected_type)?;

    let user_row = sqlx::query("SELECT token_revoked_at, deleted_at FROM users WHERE id = $1")
        .bind(claims.sub)
        .fetch_optional(db.as_ref())
        .await
        .map_err(|_| "token verification failed".to_string())?;

    let Some(user_row) = user_row else {
        return Err("invalid token".to_string());
    };

    let token_revoked_at = user_row
        .try_get::<Option<DateTime<Utc>>, _>("token_revoked_at")
        .map_err(|_| "token verification failed".to_string())?;

    let deleted_at = user_row
        .try_get::<Option<DateTime<Utc>>, _>("deleted_at")
        .map_err(|_| "token verification failed".to_string())?;

    if deleted_at.is_some() {
        return Err("token revoked".to_string());
    }

    if is_token_issued_before_revocation(claims.iat, token_revoked_at) {
        return Err("token revoked".to_string());
    }

    Ok(claims)
}

#[cfg(test)]
mod tests {
    use super::is_token_issued_before_revocation;
    use chrono::{Duration, Utc};

    #[test]
    fn token_before_revocation_is_rejected() {
        let revoked_at = Utc::now();
        let iat = (revoked_at - Duration::seconds(5)).timestamp();
        assert!(is_token_issued_before_revocation(iat, Some(revoked_at)));
    }

    #[test]
    fn token_after_revocation_is_allowed() {
        let revoked_at = Utc::now();
        let iat = (revoked_at + Duration::seconds(5)).timestamp();
        assert!(!is_token_issued_before_revocation(iat, Some(revoked_at)));
    }

    #[test]
    fn token_without_revocation_is_allowed() {
        let iat = Utc::now().timestamp();
        assert!(!is_token_issued_before_revocation(iat, None));
    }
}
