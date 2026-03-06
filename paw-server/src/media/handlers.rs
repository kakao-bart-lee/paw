use axum::{
    Extension, Json,
    extract::{Multipart, Path, State},
    http::{StatusCode, header},
    response::{IntoResponse, Response},
};
use serde_json::json;
use std::time::Duration;
use uuid::Uuid;

use crate::auth::AppState;
use crate::auth::middleware::UserId;

const MAX_FILE_SIZE: usize = 50 * 1024 * 1024; // 50 MB
pub(crate) const MEDIA_CACHE_CONTROL_HEADER_VALUE: &str = "public, max-age=31536000, immutable";

const ALLOWED_CONTENT_TYPES: &[&str] = &[
    "image/jpeg",
    "image/png",
    "image/gif",
    "image/webp",
    "video/mp4",
    "audio/mpeg",
    "application/pdf",
];

fn media_type_from_mime(mime: &str) -> &'static str {
    if mime.starts_with("image/") {
        "image"
    } else if mime.starts_with("video/") {
        "video"
    } else if mime.starts_with("audio/") {
        "audio"
    } else {
        "file"
    }
}

fn error_response(status: StatusCode, code: &str, message: &str) -> Response {
    (
        status,
        Json(json!({
            "error": code,
            "message": message,
        })),
    )
        .into_response()
}

pub async fn upload(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    mut multipart: Multipart,
) -> Response {
    let mut file_data: Option<Vec<u8>> = None;
    let mut file_content_type: Option<String> = None;
    let mut file_name: Option<String> = None;
    let mut explicit_content_type: Option<String> = None;

    while let Ok(Some(field)) = multipart.next_field().await {
        let name = field.name().unwrap_or("").to_string();

        match name.as_str() {
            "file" => {
                file_content_type = field.content_type().map(|ct| ct.to_string());
                file_name = field.file_name().map(|f| f.to_string());
                match field.bytes().await {
                    Ok(bytes) => file_data = Some(bytes.to_vec()),
                    Err(_) => {
                        return error_response(
                            StatusCode::BAD_REQUEST,
                            "read_failed",
                            "Failed to read file data",
                        );
                    }
                }
            }
            "content_type" => {
                if let Ok(text) = field.text().await {
                    explicit_content_type = Some(text);
                }
            }
            _ => {}
        }
    }

    let data = match file_data {
        Some(d) => d,
        None => {
            return error_response(
                StatusCode::BAD_REQUEST,
                "missing_file",
                "No file field in multipart form",
            );
        }
    };

    if data.len() > MAX_FILE_SIZE {
        return error_response(
            StatusCode::PAYLOAD_TOO_LARGE,
            "file_too_large",
            "File exceeds maximum size of 50MB",
        );
    }

    let content_type = explicit_content_type
        .or(file_content_type)
        .unwrap_or_else(|| "application/octet-stream".to_string());

    if !ALLOWED_CONTENT_TYPES.contains(&content_type.as_str()) {
        return error_response(
            StatusCode::BAD_REQUEST,
            "unsupported_content_type",
            "Content type is not supported",
        );
    }

    let file_name = file_name.unwrap_or_else(|| "unnamed".to_string());
    let media_id = Uuid::new_v4();
    let s3_key = format!("media/{}/{}/{}", user_id.0, media_id, file_name);
    let file_size = data.len() as i64;
    let media_type = media_type_from_mime(&content_type);

    if let Err(e) = state.media_service.upload(&s3_key, data, &content_type).await {
        tracing::error!(error = %e, "S3 upload failed");
        return error_response(
            StatusCode::INTERNAL_SERVER_ERROR,
            "upload_failed",
            "Failed to upload file to storage",
        );
    }

    let result = sqlx::query_scalar::<_, Uuid>(
        "INSERT INTO media_attachments (uploader_id, media_type, mime_type, file_name, file_size, s3_key)
         VALUES ($1, $2, $3, $4, $5, $6)
         RETURNING id",
    )
    .bind(user_id.0)
    .bind(media_type)
    .bind(&content_type)
    .bind(&file_name)
    .bind(file_size)
    .bind(&s3_key)
    .fetch_one(state.db.as_ref())
    .await;

    match result {
        Ok(id) => (
            StatusCode::CREATED,
            Json(json!({
                "id": id,
                "url": s3_key,
                "content_type": content_type,
                "size_bytes": file_size,
            })),
        )
            .into_response(),
        Err(e) => {
            tracing::error!(error = %e, "Failed to insert media record");
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "db_insert_failed",
                "Failed to save media record",
            )
        }
    }
}

pub async fn get_url(
    State(state): State<AppState>,
    Extension(_user_id): Extension<UserId>,
    Path(media_id): Path<Uuid>,
) -> Response {
    let record = sqlx::query_scalar::<_, String>(
        "SELECT s3_key FROM media_attachments WHERE id = $1",
    )
    .bind(media_id)
    .fetch_optional(state.db.as_ref())
    .await;

    let s3_key = match record {
        Ok(Some(key)) => key,
        Ok(None) => {
            return error_response(
                StatusCode::NOT_FOUND,
                "media_not_found",
                "Media attachment not found",
            );
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to query media record");
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "db_query_failed",
                "Failed to query media record",
            );
        }
    };

    let expires_in = Duration::from_secs(3600);
    match state.media_service.presigned_url(&s3_key, expires_in).await {
        Ok(url) => {
            let expires_at = chrono::Utc::now() + chrono::Duration::seconds(3600);
            (
                StatusCode::OK,
                [(header::CACHE_CONTROL, MEDIA_CACHE_CONTROL_HEADER_VALUE)],
                Json(json!({
                    "url": url,
                    "expires_at": expires_at,
                })),
            )
                .into_response()
        }
        Err(e) => {
            tracing::error!(error = %e, "Failed to generate presigned URL");
            error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "presign_failed",
                "Failed to generate presigned URL",
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn media_cache_header_matches_cdn_immutable_policy() {
        assert_eq!(
            MEDIA_CACHE_CONTROL_HEADER_VALUE,
            "public, max-age=31536000, immutable"
        );
    }
}
