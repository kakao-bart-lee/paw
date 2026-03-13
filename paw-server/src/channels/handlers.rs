use crate::auth::{middleware::UserId, AppState};
use crate::channels::models::{
    CreateChannelRequest, CreateChannelResponse, GetChannelMessagesQuery,
    GetChannelMessagesResponse, ListChannelsQuery, ListChannelsResponse, SendChannelMessageRequest,
    SubscribeResponse, UnsubscribeResponse,
};
use crate::channels::service::{
    self, ChannelAccess, SendPermission, SubscribeError, UnsubscribeError,
};
use crate::messages::service as message_service;
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};
use uuid::Uuid;

pub async fn create_channel(
    State(state): State<AppState>,
    Extension(UserId(user_id)): Extension<UserId>,
    Json(payload): Json<CreateChannelRequest>,
) -> Response {
    let name = payload.name.trim();
    if name.is_empty() {
        return error(
            StatusCode::BAD_REQUEST,
            "invalid_channel_name",
            "Channel name is required",
        )
        .into_response();
    }

    let is_public = payload.is_public.unwrap_or(true);
    let created = match service::create_channel(&state.db, user_id, name, is_public).await {
        Ok(channel) => channel,
        Err(err) => {
            tracing::error!(%err, owner_id = %user_id, "failed to create channel");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "channel_create_failed",
                "Could not create channel",
            )
            .into_response();
        }
    };

    let response = CreateChannelResponse {
        id: created.id,
        name: created.name,
        owner_id: created.owner_id,
        is_public: created.is_public,
        created_at: created.created_at,
    };

    (StatusCode::CREATED, Json(response)).into_response()
}

pub async fn list_channels(
    State(state): State<AppState>,
    Extension(UserId(user_id)): Extension<UserId>,
    Query(query): Query<ListChannelsQuery>,
) -> Response {
    let channels = match service::list_public_channels(&state.db, user_id, query.q.as_deref()).await
    {
        Ok(channels) => channels,
        Err(err) => {
            tracing::error!(%err, user_id = %user_id, "failed to list channels");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "channel_list_failed",
                "Could not list channels",
            )
            .into_response();
        }
    };

    Json(ListChannelsResponse { channels }).into_response()
}

pub async fn subscribe_channel(
    State(state): State<AppState>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(channel_id): Path<Uuid>,
) -> Response {
    match service::subscribe(&state.db, channel_id, user_id).await {
        Ok(subscribed) => Json(SubscribeResponse { subscribed }).into_response(),
        Err(SubscribeError::NotFound) => error(
            StatusCode::NOT_FOUND,
            "channel_not_found",
            "Channel not found",
        )
        .into_response(),
        Err(SubscribeError::Forbidden) => error(
            StatusCode::FORBIDDEN,
            "forbidden",
            "Cannot subscribe to a private channel",
        )
        .into_response(),
    }
}

pub async fn unsubscribe_channel(
    State(state): State<AppState>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(channel_id): Path<Uuid>,
) -> Response {
    match service::unsubscribe(&state.db, channel_id, user_id).await {
        Ok(unsubscribed) => Json(UnsubscribeResponse { unsubscribed }).into_response(),
        Err(UnsubscribeError::NotFound) => error(
            StatusCode::NOT_FOUND,
            "channel_not_found",
            "Channel not found",
        )
        .into_response(),
        Err(UnsubscribeError::CannotUnsubscribeOwner) => error(
            StatusCode::FORBIDDEN,
            "forbidden",
            "Channel owner cannot unsubscribe",
        )
        .into_response(),
    }
}

pub async fn send_channel_message(
    State(state): State<AppState>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(channel_id): Path<Uuid>,
    Json(payload): Json<SendChannelMessageRequest>,
) -> Response {
    if payload.content.trim().is_empty() {
        return error(
            StatusCode::BAD_REQUEST,
            "invalid_content",
            "Message content is required",
        )
        .into_response();
    }

    let format = payload.format.to_ascii_lowercase();
    if format != "markdown" && format != "plain" {
        return error(
            StatusCode::BAD_REQUEST,
            "invalid_format",
            "format must be markdown or plain",
        )
        .into_response();
    }

    match service::send_permission(&state.db, channel_id, user_id).await {
        Ok(SendPermission::Owner) => {}
        Ok(SendPermission::NotOwner) => {
            return error(
                StatusCode::FORBIDDEN,
                "forbidden",
                "Only channel owner can send messages",
            )
            .into_response();
        }
        Ok(SendPermission::NotFound) => {
            return error(
                StatusCode::NOT_FOUND,
                "channel_not_found",
                "Channel not found",
            )
            .into_response();
        }
        Err(err) => {
            tracing::error!(%err, channel_id = %channel_id, user_id = %user_id, "failed to validate channel sender");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "channel_permission_failed",
                "Could not validate channel permission",
            )
            .into_response();
        }
    }

    match message_service::get_idempotent_message(
        &state.db,
        channel_id,
        user_id,
        payload.idempotency_key,
    )
    .await
    {
        Ok(Some(existing)) => return Json(existing).into_response(),
        Ok(None) => {}
        Err(err) => {
            tracing::error!(%err, channel_id = %channel_id, owner_id = %user_id, "failed idempotency lookup");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "message_lookup_failed",
                "Could not send message",
            )
            .into_response();
        }
    }

    match message_service::send_message(
        &state.db,
        channel_id,
        user_id,
        payload.content.trim(),
        &format,
        payload.idempotency_key,
    )
    .await
    {
        Ok(created) => Json(created).into_response(),
        Err(err) => {
            tracing::warn!(%err, channel_id = %channel_id, owner_id = %user_id, "channel message insert failed, checking idempotency replay");
            match message_service::get_idempotent_message(
                &state.db,
                channel_id,
                user_id,
                payload.idempotency_key,
            )
            .await
            {
                Ok(Some(existing)) => Json(existing).into_response(),
                _ => error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "message_send_failed",
                    "Could not send message",
                )
                .into_response(),
            }
        }
    }
}

pub async fn get_channel_messages(
    State(state): State<AppState>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(channel_id): Path<Uuid>,
    Query(query): Query<GetChannelMessagesQuery>,
) -> Response {
    match service::access_for_user(&state.db, channel_id, user_id).await {
        Ok(ChannelAccess::Owner | ChannelAccess::Subscriber) => {}
        Ok(ChannelAccess::Forbidden) => {
            return error(
                StatusCode::FORBIDDEN,
                "forbidden",
                "User is not subscribed to this channel",
            )
            .into_response();
        }
        Ok(ChannelAccess::NotFound) => {
            return error(
                StatusCode::NOT_FOUND,
                "channel_not_found",
                "Channel not found",
            )
            .into_response();
        }
        Err(err) => {
            tracing::error!(%err, channel_id = %channel_id, user_id = %user_id, "failed to validate channel access");
            return error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "channel_access_failed",
                "Could not validate channel access",
            )
            .into_response();
        }
    }

    let after_seq = query.after_seq.unwrap_or(0);
    let limit = query.limit.unwrap_or(50).clamp(1, 50);

    let mut messages =
        match message_service::get_messages(&state.db, channel_id, after_seq, limit + 1).await {
            Ok(rows) => rows,
            Err(err) => {
                tracing::error!(%err, channel_id = %channel_id, "failed to fetch channel messages");
                return error(
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "channel_message_history_failed",
                    "Could not fetch channel message history",
                )
                .into_response();
            }
        };

    let has_more = messages.len() as i64 > limit;
    if has_more {
        messages.truncate(limit as usize);
    }

    Json(GetChannelMessagesResponse { messages, has_more }).into_response()
}

fn error(status: StatusCode, code: &str, message: &str) -> (StatusCode, Json<Value>) {
    (
        status,
        Json(json!({
            "error": code,
            "message": message,
        })),
    )
}
