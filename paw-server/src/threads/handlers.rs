use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::Value;
use uuid::Uuid;

use super::{
    models::{ArchiveThreadResponse, CreateThreadRequest, UpdateThreadTitleRequest},
    service::{self, CreateThreadError},
};
use crate::auth::{middleware::UserId, AppState};
use crate::context_engine::models::ThreadCreatedHook;
use crate::context_engine::LifecycleHooks;
use crate::i18n::{error_response, RequestLocale};
use crate::messages::service::{self as message_service, Membership};
use chrono::Utc;

fn error(status: StatusCode, code: &str, locale: &str, message: &str) -> (StatusCode, Json<Value>) {
    error_response(status, code, locale, message)
}

async fn ensure_membership(
    state: &AppState,
    conversation_id: Uuid,
    user_id: Uuid,
    locale: &str,
) -> Result<(), Response> {
    match message_service::check_member(&state.db, conversation_id, user_id).await {
        Ok(Membership::Member) => Ok(()),
        Ok(Membership::NotMember) => Err(error(
            StatusCode::FORBIDDEN,
            "forbidden",
            locale,
            "User is not a member of this conversation",
        )
        .into_response()),
        Ok(Membership::ConversationNotFound) => Err(error(
            StatusCode::NOT_FOUND,
            "conversation_not_found",
            locale,
            "Conversation not found",
        )
        .into_response()),
        Err(err) => {
            tracing::error!(%err, conversation_id = %conversation_id, user_id = %user_id, "failed membership check");
            Err(error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "membership_check_failed",
                locale,
                "Could not validate conversation membership",
            )
            .into_response())
        }
    }
}

pub async fn list_threads(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(conversation_id): Path<Uuid>,
) -> Response {
    match ensure_membership(&state, conversation_id, user_id, &locale).await {
        Ok(()) => {}
        Err(response) => return response,
    }

    match service::list_threads(&state.db, conversation_id).await {
        Ok(threads) => Json(threads).into_response(),
        Err(err) => {
            tracing::error!(%err, conversation_id = %conversation_id, "failed to list threads");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "thread_list_failed",
                &locale,
                "Could not list threads",
            )
            .into_response()
        }
    }
}

pub async fn create_thread(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(conversation_id): Path<Uuid>,
    Json(payload): Json<CreateThreadRequest>,
) -> Response {
    match ensure_membership(&state, conversation_id, user_id, &locale).await {
        Ok(()) => {}
        Err(response) => return response,
    }

    match service::create_thread(
        &state.db,
        conversation_id,
        payload.root_message_id,
        user_id,
        payload.title,
    )
    .await
    {
        Ok(thread) => {
            state
                .context_engine
                .on_thread_created(ThreadCreatedHook {
                    conversation_id,
                    thread_id: thread.id,
                    root_message_id: thread.root_message_id,
                    created_by: user_id,
                    title: thread.title.clone(),
                    timestamp: Utc::now(),
                })
                .await;

            (StatusCode::CREATED, Json(thread)).into_response()
        }
        Err(CreateThreadError::ConversationNotFound) => error(
            StatusCode::NOT_FOUND,
            "conversation_not_found",
            &locale,
            "Conversation not found",
        )
        .into_response(),
        Err(CreateThreadError::ThreadsNotAllowed) => error(
            StatusCode::BAD_REQUEST,
            "threads_not_allowed",
            &locale,
            "Threads are only allowed in group conversations",
        )
        .into_response(),
        Err(CreateThreadError::RootMessageNotFound) => error(
            StatusCode::BAD_REQUEST,
            "message_not_found",
            &locale,
            "Root message not found in this conversation",
        )
        .into_response(),
        Err(CreateThreadError::RootMessageMustBeMainTimeline) => error(
            StatusCode::BAD_REQUEST,
            "root_message_invalid",
            &locale,
            "Root message must belong to the main timeline",
        )
        .into_response(),
        Err(CreateThreadError::ThreadAlreadyExists) => error(
            StatusCode::CONFLICT,
            "thread_already_exists",
            &locale,
            "Thread already exists for this root message",
        )
        .into_response(),
        Err(CreateThreadError::ThreadLimitExceeded) => error(
            StatusCode::TOO_MANY_REQUESTS,
            "thread_limit_exceeded",
            &locale,
            "Maximum 100 threads per conversation reached",
        )
        .into_response(),
        Err(CreateThreadError::Database(err)) => {
            tracing::error!(%err, conversation_id = %conversation_id, root_message_id = %payload.root_message_id, "failed to create thread");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "thread_create_failed",
                &locale,
                "Could not create thread",
            )
            .into_response()
        }
    }
}

pub async fn get_thread(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((conversation_id, thread_id)): Path<(Uuid, Uuid)>,
) -> Response {
    match ensure_membership(&state, conversation_id, user_id, &locale).await {
        Ok(()) => {}
        Err(response) => return response,
    }

    match service::get_thread(&state.db, conversation_id, thread_id).await {
        Ok(Some(thread)) => Json(thread).into_response(),
        Ok(None) => error(
            StatusCode::NOT_FOUND,
            "thread_not_found",
            &locale,
            "Thread not found",
        )
        .into_response(),
        Err(err) => {
            tracing::error!(%err, conversation_id = %conversation_id, thread_id = %thread_id, "failed to load thread");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "thread_get_failed",
                &locale,
                "Could not load thread",
            )
            .into_response()
        }
    }
}

pub async fn update_thread_title(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((conversation_id, thread_id)): Path<(Uuid, Uuid)>,
    Json(payload): Json<UpdateThreadTitleRequest>,
) -> Response {
    match ensure_membership(&state, conversation_id, user_id, &locale).await {
        Ok(()) => {}
        Err(response) => return response,
    }

    match service::update_thread_title(&state.db, conversation_id, thread_id, payload.title).await {
        Ok(Some(thread)) => Json(thread).into_response(),
        Ok(None) => error(
            StatusCode::NOT_FOUND,
            "thread_not_found",
            &locale,
            "Thread not found",
        )
        .into_response(),
        Err(err) => {
            tracing::error!(%err, conversation_id = %conversation_id, thread_id = %thread_id, "failed to update thread title");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "thread_update_failed",
                &locale,
                "Could not update thread title",
            )
            .into_response()
        }
    }
}

pub async fn archive_thread(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((conversation_id, thread_id)): Path<(Uuid, Uuid)>,
) -> Response {
    match ensure_membership(&state, conversation_id, user_id, &locale).await {
        Ok(()) => {}
        Err(response) => return response,
    }

    match service::archive_thread(&state.db, conversation_id, thread_id, user_id).await {
        Ok(true) => Json(ArchiveThreadResponse { archived: true }).into_response(),
        Ok(false) => error(
            StatusCode::NOT_FOUND,
            "thread_not_found",
            &locale,
            "Thread not found",
        )
        .into_response(),
        Err(err) => {
            tracing::error!(%err, conversation_id = %conversation_id, thread_id = %thread_id, "failed to archive thread");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "thread_archive_failed",
                &locale,
                "Could not archive thread",
            )
            .into_response()
        }
    }
}
