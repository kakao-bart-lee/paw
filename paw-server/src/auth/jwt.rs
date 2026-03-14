use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
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
