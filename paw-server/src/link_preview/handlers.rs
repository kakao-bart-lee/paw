use crate::auth::{middleware::UserId, AppState};
use crate::i18n::{error_response, RequestLocale};
use crate::link_preview::models::{
    GetLinkPreviewQuery, RequestLinkPreviewsPayload, RequestLinkPreviewsResponse,
};
use crate::link_preview::service::normalize_url;
use axum::{
    extract::{Extension, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::Value;

pub async fn request_link_previews(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(_user_id)): Extension<UserId>,
    Json(payload): Json<RequestLinkPreviewsPayload>,
) -> Response {
    if payload.content.trim().is_empty() {
        return error(
            StatusCode::BAD_REQUEST,
            "invalid_content",
            &locale,
            "content is required",
        )
        .into_response();
    }

    let requested_urls = match state
        .link_preview_service
        .request_previews(&state.db, &payload.content)
        .await
    {
        Ok(urls) if !urls.is_empty() => urls,
        Ok(_) => {
            return error(
                StatusCode::BAD_REQUEST,
                "no_urls_found",
                &locale,
                "No valid URLs found in content",
            )
            .into_response();
        }
        Err(err) => {
            tracing::error!(%err, "failed to request link previews");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "link_preview_request_failed",
                &locale,
                "Could not request link previews",
            )
            .into_response();
        }
    };

    let previews = match state
        .link_preview_service
        .get_previews_by_urls(&state.db, &requested_urls)
        .await
    {
        Ok(items) => items,
        Err(err) => {
            tracing::error!(%err, "failed to fetch requested link previews");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "link_preview_fetch_failed",
                &locale,
                "Could not fetch link previews",
            )
            .into_response();
        }
    };

    (
        StatusCode::ACCEPTED,
        Json(RequestLinkPreviewsResponse {
            requested_urls,
            previews,
        }),
    )
        .into_response()
}

pub async fn get_link_preview(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(_user_id)): Extension<UserId>,
    Query(query): Query<GetLinkPreviewQuery>,
) -> Response {
    let normalized_url = match normalize_url(&query.url) {
        Some(url) => url,
        None => {
            return error(
                StatusCode::BAD_REQUEST,
                "invalid_url",
                &locale,
                "url must be a valid http or https URL",
            )
            .into_response();
        }
    };

    match state
        .link_preview_service
        .get_preview_by_url(&state.db, &normalized_url)
        .await
    {
        Ok(Some(preview)) => Json(preview).into_response(),
        Ok(None) => error(
            StatusCode::NOT_FOUND,
            "link_preview_not_found",
            &locale,
            "Link preview not found",
        )
        .into_response(),
        Err(err) => {
            tracing::error!(%err, target_url = %normalized_url, "failed to get link preview");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "link_preview_lookup_failed",
                &locale,
                "Could not load link preview",
            )
            .into_response()
        }
    }
}

fn error(status: StatusCode, code: &str, locale: &str, message: &str) -> (StatusCode, Json<Value>) {
    error_response(status, code, locale, message)
}
