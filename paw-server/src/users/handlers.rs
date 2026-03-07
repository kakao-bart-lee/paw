use axum::{
    Extension, Json,
    extract::{Path, Query, State},
    http::StatusCode,
};
use serde_json::{Value, json};
use uuid::Uuid;

use crate::auth::AppState;
use crate::auth::middleware::UserId;

use super::models::{PublicUser, SearchQuery, UpdateProfileRequest, User};

fn error(status: StatusCode, code: &str, message: &str) -> (StatusCode, Json<Value>) {
    (
        status,
        Json(json!({
            "error": code,
            "message": message,
        })),
    )
}

pub async fn get_me(
    Extension(user_id): Extension<UserId>,
    State(state): State<AppState>,
) -> (StatusCode, Json<Value>) {
    match sqlx::query_as::<_, User>(
        "SELECT id, phone, display_name, avatar_url, created_at \
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
        Ok(None) => error(StatusCode::NOT_FOUND, "user_not_found", "User not found"),
        Err(_) => error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "query_failed",
            "Failed to fetch user profile",
        ),
    }
}

pub async fn update_me(
    Extension(user_id): Extension<UserId>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateProfileRequest>,
) -> (StatusCode, Json<Value>) {
    match sqlx::query_as::<_, User>(
        "UPDATE users SET \
         display_name = COALESCE($1, display_name), \
         avatar_url = COALESCE($2, avatar_url) \
         WHERE id = $3 \
         RETURNING id, phone, display_name, avatar_url, created_at",
    )
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
        Ok(None) => error(StatusCode::NOT_FOUND, "user_not_found", "User not found"),
        Err(_) => error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "update_failed",
            "Failed to update profile",
        ),
    }
}

pub async fn search_user(
    Extension(_user_id): Extension<UserId>,
    State(state): State<AppState>,
    Query(params): Query<SearchQuery>,
) -> (StatusCode, Json<Value>) {
    match sqlx::query_as::<_, PublicUser>(
        "SELECT id, display_name, avatar_url FROM users WHERE phone = $1",
    )
    .bind(&params.phone)
    .fetch_optional(state.db.as_ref())
    .await
    {
        Ok(Some(user)) => (
            StatusCode::OK,
            Json(serde_json::to_value(user).unwrap_or(Value::Null)),
        ),
        Ok(None) => error(StatusCode::NOT_FOUND, "user_not_found", "User not found"),
        Err(_) => error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "query_failed",
            "Failed to search users",
        ),
    }
}

pub async fn get_user(
    Extension(_user_id): Extension<UserId>,
    State(state): State<AppState>,
    Path(target_user_id): Path<Uuid>,
) -> (StatusCode, Json<Value>) {
    match sqlx::query_as::<_, PublicUser>(
        "SELECT id, display_name, avatar_url FROM users WHERE id = $1",
    )
    .bind(target_user_id)
    .fetch_optional(state.db.as_ref())
    .await
    {
        Ok(Some(user)) => (
            StatusCode::OK,
            Json(serde_json::to_value(user).unwrap_or(Value::Null)),
        ),
        Ok(None) => error(StatusCode::NOT_FOUND, "user_not_found", "User not found"),
        Err(_) => error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "query_failed",
            "Failed to fetch user",
        ),
    }
}
