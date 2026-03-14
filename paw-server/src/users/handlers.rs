use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use serde_json::Value;
use sqlx::Error as SqlxError;
use uuid::Uuid;

use crate::auth::middleware::UserId;
use crate::auth::AppState;
use crate::i18n::{error_response, normalize_locale, RequestLocale};

use super::models::{PublicUser, SearchQuery, UpdateProfileRequest, User};

fn normalize_username(input: &str) -> Option<String> {
    let normalized = input.trim().to_ascii_lowercase();
    let reserved = ["admin", "support", "system", "help", "paw"];
    let valid = !normalized.is_empty()
        && normalized.len() >= 3
        && normalized.len() <= 20
        && !reserved.contains(&normalized.as_str())
        && normalized
            .chars()
            .all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_');

    valid.then_some(normalized)
}

pub async fn get_me(
    Extension(user_id): Extension<UserId>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    State(state): State<AppState>,
) -> (StatusCode, Json<Value>) {
    match sqlx::query_as::<_, User>(
        "SELECT id, phone, username, preferred_locale, discoverable_by_phone, phone_verified_at, display_name, avatar_url, created_at \
         FROM users WHERE id = $1",
    )
    .bind(user_id.0)
    .fetch_optional(state.db.as_ref())
    .await
    {
        Ok(Some(user)) => (
            StatusCode::OK,
            Json(serde_json::to_value(user).unwrap_or(Value::Null)),
        ),
        Ok(None) => error_response(StatusCode::NOT_FOUND, "user_not_found", &locale, "User not found"),
        Err(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "query_failed",
            &locale,
            "Failed to fetch user profile",
        ),
    }
}

pub async fn update_me(
    Extension(user_id): Extension<UserId>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateProfileRequest>,
) -> (StatusCode, Json<Value>) {
    let username = payload
        .username
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty());
    let preferred_locale = match payload.preferred_locale.as_deref() {
        Some(value) => match normalize_locale(value) {
            Some(locale) => Some(locale),
            None => {
                return error_response(
                    StatusCode::BAD_REQUEST,
                    "invalid_preferred_locale",
                    &locale,
                    "Preferred locale must be a valid BCP-47 style tag such as ko-KR or en-US",
                );
            }
        },
        None => None,
    };

    if let Some(raw_username) = username {
        if normalize_username(raw_username).is_none() {
            return error_response(
                StatusCode::BAD_REQUEST,
                "invalid_username",
                &locale,
                "Username must be 3-20 chars of lowercase letters, numbers, or underscores",
            );
        }
    }

    match sqlx::query_as::<_, User>(
        "UPDATE users SET \
         username = COALESCE($1, username), \
         preferred_locale = COALESCE($2, preferred_locale), \
         discoverable_by_phone = COALESCE($3, discoverable_by_phone), \
         display_name = COALESCE($4, display_name), \
         avatar_url = COALESCE($5, avatar_url) \
         WHERE id = $6 \
         RETURNING id, phone, username, preferred_locale, discoverable_by_phone, phone_verified_at, display_name, avatar_url, created_at",
    )
    .bind(username.and_then(normalize_username))
    .bind(preferred_locale)
    .bind(payload.discoverable_by_phone)
    .bind(&payload.display_name)
    .bind(&payload.avatar_url)
    .bind(user_id.0)
    .fetch_optional(state.db.as_ref())
    .await
    {
        Ok(Some(user)) => (
            StatusCode::OK,
            Json(serde_json::to_value(user).unwrap_or(Value::Null)),
        ),
        Ok(None) => error_response(StatusCode::NOT_FOUND, "user_not_found", &locale, "User not found"),
        Err(SqlxError::Database(db_err)) if db_err.code().as_deref() == Some("23505") => error_response(
            StatusCode::CONFLICT,
            "username_taken",
            &locale,
            "Username is already in use",
        ),
        Err(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "update_failed",
            &locale,
            "Failed to update profile",
        ),
    }
}

pub async fn search_user(
    Extension(_user_id): Extension<UserId>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> (StatusCode, Json<Value>) {
    let result = if let Some(username) = params.username.as_deref() {
        let Some(username) = normalize_username(username) else {
            return error_response(
                StatusCode::BAD_REQUEST,
                "invalid_username",
                &locale,
                "Username must be 3-20 chars of lowercase letters, numbers, or underscores",
            );
        };

        sqlx::query_as::<_, PublicUser>(
            "SELECT id, username, display_name, avatar_url \
             FROM users \
             WHERE username = $1",
        )
        .bind(username)
        .fetch_optional(state.db.as_ref())
        .await
    } else if let Some(phone) = params.phone.as_deref() {
        sqlx::query_as::<_, PublicUser>(
            "SELECT id, username, display_name, avatar_url \
             FROM users \
             WHERE phone = $1 \
               AND discoverable_by_phone = TRUE \
               AND phone_verified_at IS NOT NULL",
        )
        .bind(phone)
        .fetch_optional(state.db.as_ref())
        .await
    } else {
        return error_response(
            StatusCode::BAD_REQUEST,
            "missing_search_param",
            &locale,
            "Provide either username or phone",
        );
    };

    match result {
        Ok(Some(user)) => (
            StatusCode::OK,
            Json(serde_json::to_value(user).unwrap_or(Value::Null)),
        ),
        Ok(None) => error_response(
            StatusCode::NOT_FOUND,
            "user_not_found",
            &locale,
            "User not found",
        ),
        Err(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "query_failed",
            &locale,
            "Failed to search users",
        ),
    }
}

pub async fn get_user(
    Extension(_user_id): Extension<UserId>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    State(state): State<AppState>,
    Path(target_user_id): Path<Uuid>,
) -> (StatusCode, Json<Value>) {
    match sqlx::query_as::<_, PublicUser>(
        "SELECT id, username, display_name, avatar_url FROM users WHERE id = $1",
    )
    .bind(target_user_id)
    .fetch_optional(state.db.as_ref())
    .await
    {
        Ok(Some(user)) => (
            StatusCode::OK,
            Json(serde_json::to_value(user).unwrap_or(Value::Null)),
        ),
        Ok(None) => error_response(
            StatusCode::NOT_FOUND,
            "user_not_found",
            &locale,
            "User not found",
        ),
        Err(_) => error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "query_failed",
            &locale,
            "Failed to fetch user",
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_username;
    use crate::i18n::normalize_locale;

    #[test]
    fn normalize_username_accepts_valid_input() {
        assert_eq!(
            normalize_username("  Paw_Friend19 "),
            Some("paw_friend19".to_string())
        );
    }

    #[test]
    fn normalize_username_rejects_invalid_input() {
        assert_eq!(normalize_username("ab"), None);
        assert_eq!(normalize_username("user-name"), None);
        assert_eq!(normalize_username("UPPER.CASE"), None);
        assert_eq!(normalize_username("admin"), None);
        assert_eq!(normalize_username("support"), None);
    }

    #[test]
    fn preferred_locale_accepts_valid_bcp47_style_tags() {
        assert_eq!(normalize_locale("ko_kr"), Some("ko-KR".to_string()));
        assert_eq!(normalize_locale("en-us"), Some("en-US".to_string()));
    }
}
