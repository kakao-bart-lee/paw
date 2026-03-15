use axum::{
    extract::{Path, Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension, Json,
};
use futures_util::{SinkExt, StreamExt};
use paw_proto::{AgentStreamMsg, ServerMessage, StreamEndMsg, PROTOCOL_VERSION};
use serde::Serialize;
use serde_json::{json, Value};
use sqlx::Row;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

use super::models::{
    AgentProfile, InstallAgentResponse, InstalledAgentsResponse, InviteAgentRequest,
    InviteAgentResponse, MarketplaceSearchQuery, MarketplaceSearchResponse, PublishAgentRequest,
    PublishAgentResponse, RegisterAgentRequest, RegisterAgentResponse, RevokeAgentResponse,
    RotateAgentKeyResponse, UninstallAgentResponse,
};
use super::service;
use crate::auth::middleware::UserId;
use crate::auth::AppState;
use crate::i18n::{error_response, error_response_with_details, localized_message, RequestLocale};
use crate::messages::service::{check_member, Membership};

const MAX_STREAM_DURATION: Duration = Duration::from_secs(300);
const MAX_STREAM_BYTES: usize = 1_048_576;
pub const MAX_CONCURRENT_STREAMS_PER_AGENT: usize = 10;
pub const MAX_DELTA_SIZE: usize = 4096;

#[derive(Clone, Copy)]
struct StreamRelayState {
    conversation_id: Uuid,
    thread_id: Option<Uuid>,
    started_at: Instant,
    bytes_sent: usize,
}

#[derive(Debug, Serialize)]
pub struct RemoveAgentResponse {
    pub removed: bool,
}

pub async fn register_agent_handler(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(user_id): Extension<UserId>,
    Json(req): Json<RegisterAgentRequest>,
) -> Result<Json<RegisterAgentResponse>, (StatusCode, Json<serde_json::Value>)> {
    let resp = service::register_agent(&state.db, user_id.0, req)
        .await
        .map_err(|e| {
            tracing::error!("failed to register agent: {e}");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "registration_failed",
                &locale,
                "Failed to register agent",
            )
        })?;

    Ok(Json(resp))
}

pub async fn get_agent_handler(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<AgentProfile>, (StatusCode, Json<serde_json::Value>)> {
    let profile = service::get_agent_profile(&state.db, agent_id)
        .await
        .map_err(|e| {
            tracing::error!("failed to fetch agent profile: {e}");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                &locale,
                "Failed to fetch agent profile",
            )
        })?;

    match profile {
        Some(p) => Ok(Json(p)),
        None => Err(error(
            StatusCode::NOT_FOUND,
            "agent_not_found",
            &locale,
            "Agent not found",
        )),
    }
}

pub async fn revoke_agent_handler(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(user_id): Extension<UserId>,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<RevokeAgentResponse>, (StatusCode, Json<serde_json::Value>)> {
    let result = service::revoke_agent_token(&state.db, agent_id, user_id.0)
        .await
        .map_err(|e| {
            tracing::error!("failed to revoke agent token: {e}");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                &locale,
                "Failed to revoke agent token",
            )
        })?;

    match result {
        Some(r) => Ok(Json(r)),
        None => Err(error(
            StatusCode::NOT_FOUND,
            "agent_not_found_or_not_owner",
            &locale,
            "Agent not found or you are not the owner",
        )),
    }
}

pub async fn rotate_agent_key_handler(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(user_id): Extension<UserId>,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<RotateAgentKeyResponse>, (StatusCode, Json<serde_json::Value>)> {
    let result = service::rotate_agent_token(&state.db, agent_id, user_id.0)
        .await
        .map_err(|e| {
            tracing::error!("failed to rotate agent token: {e}");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "rotate_failed",
                &locale,
                "Failed to rotate agent key",
            )
        })?;

    match result {
        Some(r) => Ok(Json(r)),
        None => Err(error(
            StatusCode::NOT_FOUND,
            "agent_not_found_or_not_owner",
            &locale,
            "Agent not found or you are not the owner",
        )),
    }
}

pub async fn invite_agent_handler(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(conversation_id): Path<Uuid>,
    Json(payload): Json<InviteAgentRequest>,
) -> Result<Json<InviteAgentResponse>, (StatusCode, Json<Value>)> {
    match check_member(&state.db, conversation_id, user_id).await {
        Ok(Membership::Member) => {}
        Ok(Membership::NotMember) => {
            return Err(error(
                StatusCode::FORBIDDEN,
                "forbidden",
                &locale,
                "User is not a member of this conversation",
            ));
        }
        Ok(Membership::ConversationNotFound) => {
            return Err(error(
                StatusCode::NOT_FOUND,
                "conversation_not_found",
                &locale,
                "Conversation not found",
            ));
        }
        Err(err) => {
            tracing::error!(%err, conversation_id = %conversation_id, user_id = %user_id, "failed membership check");
            return Err(error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "membership_check_failed",
                &locale,
                "Could not validate conversation membership",
            ));
        }
    }

    let can_manage = service::can_manage_conversation_agents(&state.db, conversation_id, user_id)
        .await
        .map_err(|err| {
            tracing::error!(%err, conversation_id = %conversation_id, user_id = %user_id, "failed role check for inviting agent");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                &locale,
                "Failed to invite agent",
            )
        })?;

    if !can_manage {
        return Err(error(
            StatusCode::FORBIDDEN,
            "forbidden",
            &locale,
            "Only conversation admins can invite agents",
        ));
    }

    match service::invite_agent_to_conversation(
        &state.db,
        conversation_id,
        payload.agent_id,
        user_id,
    )
    .await
    {
        Ok(true) => Ok(Json(InviteAgentResponse { invited: true })),
        Ok(false) => Err(error(
            StatusCode::CONFLICT,
            "already_invited",
            &locale,
            "Agent is already invited to this conversation",
        )),
        Err(err) if err.to_string() == "agent_not_found_or_revoked" => Err(error(
            StatusCode::NOT_FOUND,
            "agent_not_found",
            &locale,
            "Agent not found or revoked",
        )),
        Err(err) => {
            tracing::error!(%err, conversation_id = %conversation_id, agent_id = %payload.agent_id, "failed to invite agent");
            Err(error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "invite_failed",
                &locale,
                "Failed to invite agent",
            ))
        }
    }
}

pub async fn remove_agent_handler(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((conversation_id, agent_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<RemoveAgentResponse>, (StatusCode, Json<Value>)> {
    match service::remove_agent_from_conversation(&state.db, conversation_id, agent_id, user_id)
        .await
    {
        Ok(true) => Ok(Json(RemoveAgentResponse { removed: true })),
        Ok(false) => Err(error(
            StatusCode::NOT_FOUND,
            "agent_not_found",
            &locale,
            "Agent is not in this conversation",
        )),
        Err(err) if err.to_string() == "not_admin" => Err(error(
            StatusCode::FORBIDDEN,
            "forbidden",
            &locale,
            "Only conversation admins can remove agents",
        )),
        Err(err) => {
            tracing::error!(%err, conversation_id = %conversation_id, agent_id = %agent_id, user_id = %user_id, "failed to remove agent");
            Err(error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "remove_failed",
                &locale,
                "Failed to remove agent",
            ))
        }
    }
}

pub async fn marketplace_search_handler(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Query(params): Query<MarketplaceSearchQuery>,
) -> Result<Json<MarketplaceSearchResponse>, (StatusCode, Json<Value>)> {
    let agents = service::search_marketplace_agents(
        state.db.as_ref(),
        params.q.as_deref(),
        params.category.as_deref(),
        &params.sort,
    )
    .await
    .map_err(|e| {
        tracing::error!("marketplace search failed: {e}");
        error(
            StatusCode::INTERNAL_SERVER_ERROR,
            "search_failed",
            &locale,
            "Failed to search marketplace",
        )
    })?;

    let count = agents.len();
    Ok(Json(MarketplaceSearchResponse { agents, count }))
}

pub async fn marketplace_agent_detail_handler(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<Value>)> {
    let detail = service::get_marketplace_agent_detail(state.db.as_ref(), agent_id)
        .await
        .map_err(|e| {
            tracing::error!("marketplace agent detail failed: {e}");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                &locale,
                "Failed to get agent detail",
            )
        })?;

    match detail {
        Some(d) => Ok(Json(serde_json::to_value(d).unwrap())),
        None => Err(error(
            StatusCode::NOT_FOUND,
            "agent_not_found",
            &locale,
            "Agent not found in marketplace",
        )),
    }
}

pub async fn install_agent_handler(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<InstallAgentResponse>, (StatusCode, Json<Value>)> {
    match service::install_agent(state.db.as_ref(), user_id, agent_id).await {
        Ok(true) => Ok(Json(InstallAgentResponse {
            installed: true,
            agent_id,
        })),
        Ok(false) => Err(error(
            StatusCode::CONFLICT,
            "already_installed",
            &locale,
            "Agent is already installed",
        )),
        Err(err) if err.to_string() == "agent_not_found" => Err(error(
            StatusCode::NOT_FOUND,
            "agent_not_found",
            &locale,
            "Agent not found in marketplace",
        )),
        Err(err) => {
            tracing::error!("install agent failed: {err}");
            Err(error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "install_failed",
                &locale,
                "Failed to install agent",
            ))
        }
    }
}

pub async fn uninstall_agent_handler(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<UninstallAgentResponse>, (StatusCode, Json<Value>)> {
    match service::uninstall_agent(state.db.as_ref(), user_id, agent_id).await {
        Ok(true) => Ok(Json(UninstallAgentResponse {
            uninstalled: true,
            agent_id,
        })),
        Ok(false) => Err(error(
            StatusCode::NOT_FOUND,
            "not_installed",
            &locale,
            "Agent is not installed",
        )),
        Err(err) => {
            tracing::error!("uninstall agent failed: {err}");
            Err(error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "uninstall_failed",
                &locale,
                "Failed to uninstall agent",
            ))
        }
    }
}

pub async fn list_installed_agents_handler(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
) -> Result<Json<InstalledAgentsResponse>, (StatusCode, Json<Value>)> {
    let agents = service::list_installed_agents(state.db.as_ref(), user_id)
        .await
        .map_err(|e| {
            tracing::error!("list installed agents failed: {e}");
            error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                &locale,
                "Failed to list installed agents",
            )
        })?;

    Ok(Json(InstalledAgentsResponse { agents }))
}

pub async fn publish_agent_handler(
    State(state): State<AppState>,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(agent_id): Path<Uuid>,
    Json(req): Json<PublishAgentRequest>,
) -> Result<Json<PublishAgentResponse>, (StatusCode, Json<Value>)> {
    if let Err(msg) = req.manifest.validate() {
        return Err(error_with_details(
            StatusCode::BAD_REQUEST,
            "invalid_manifest",
            &locale,
            Some(msg),
            msg,
        ));
    }

    match service::publish_agent(
        state.db.as_ref(),
        agent_id,
        user_id,
        &req.manifest,
        req.category.as_deref(),
        req.tags.as_deref(),
    )
    .await
    {
        Ok(true) => Ok(Json(PublishAgentResponse {
            published: true,
            agent_id,
        })),
        Ok(false) => Err(error(
            StatusCode::NOT_FOUND,
            "agent_not_found_or_not_owner",
            &locale,
            "Agent not found or you are not the owner",
        )),
        Err(err) => {
            tracing::error!("publish agent failed: {err}");
            Err(error(
                StatusCode::INTERNAL_SERVER_ERROR,
                "publish_failed",
                &locale,
                "Failed to publish agent",
            ))
        }
    }
}

pub async fn agent_ws_handler(
    ws: WebSocketUpgrade,
    Extension(RequestLocale(locale)): Extension<RequestLocale>,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> Response {
    let raw_token = match params.get("token") {
        Some(t) if !t.is_empty() => t.clone(),
        _ => {
            return error_response(
                StatusCode::UNAUTHORIZED,
                "agent_missing_token",
                &locale,
                "Agent token is required",
            )
            .into_response()
        }
    };

    let agent_id = match service::verify_agent_token(&state.db, &raw_token).await {
        Ok(Some(id)) => id,
        Ok(None) => {
            return error_response(
                StatusCode::UNAUTHORIZED,
                "invalid_agent_token",
                &locale,
                "Agent token is invalid",
            )
            .into_response()
        }
        Err(e) => {
            tracing::error!("agent token verification failed: {e}");
            return error_response(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                &locale,
                "Internal error",
            )
            .into_response();
        }
    };

    let nats = match &state.nats {
        Some(n) => n.clone(),
        None => {
            let locale = locale.clone();
            return ws.on_upgrade(move |mut socket| async move {
                use axum::extract::ws::Message;
                let err_frame =
                    agent_error_frame("nats_unavailable", &locale, "Agent gateway requires NATS");
                let _ = socket
                    .send(Message::Text(err_frame.to_string().into()))
                    .await;
                let _ = socket.close().await;
            });
        }
    };

    let state_for_socket = state.clone();
    ws.on_upgrade(move |socket| {
        handle_agent_socket(socket, agent_id, locale, nats, state_for_socket)
    })
}

async fn handle_agent_socket(
    socket: axum::extract::ws::WebSocket,
    agent_id: uuid::Uuid,
    locale: String,
    nats: std::sync::Arc<async_nats::Client>,
    state: AppState,
) {
    use axum::extract::ws::Message;

    let (mut ws_tx, mut ws_rx) = socket.split();
    let (outbound_tx, mut outbound_rx) = tokio::sync::mpsc::unbounded_channel::<Message>();
    let writer = tokio::spawn(async move {
        while let Some(msg) = outbound_rx.recv().await {
            if ws_tx.send(msg).await.is_err() {
                break;
            }
        }
    });

    let subject = format!("agent.inbound.{agent_id}");
    let mut nats_sub = match nats.subscribe(subject.clone()).await {
        Ok(sub) => sub,
        Err(e) => {
            tracing::error!("NATS subscribe failed for {subject}: {e}");
            let err = agent_error_frame(
                "subscribe_failed",
                &locale,
                "Failed to subscribe agent session",
            );
            let _ = outbound_tx.send(Message::Text(err.to_string().into()));
            drop(outbound_tx);
            let _ = writer.await;
            return;
        }
    };

    let _ = state
        .hub
        .try_register_with_limit(agent_id, outbound_tx.clone(), usize::MAX)
        .await;
    tracing::info!("agent {agent_id} connected, subscribed to {subject}");
    crate::metrics::ws_connection_opened();

    let outbound_tx_nats = outbound_tx.clone();
    let nats_to_ws = async {
        while let Some(msg) = nats_sub.next().await {
            let payload = String::from_utf8_lossy(&msg.payload);
            if outbound_tx_nats
                .send(Message::Text(payload.into_owned().into()))
                .is_err()
            {
                break;
            }
        }
    };

    let ws_to_server = async {
        let mut stream_states: HashMap<Uuid, StreamRelayState> = HashMap::new();
        let rate_limit_key = format!("agent:{agent_id}");

        while let Some(Ok(msg)) = ws_rx.next().await {
            match msg {
                Message::Text(text) => {
                    if text.len() > crate::ws::MAX_WS_MESSAGE_SIZE_BYTES {
                        tracing::warn!(%agent_id, "agent websocket frame exceeds max size");
                        break;
                    }

                    if !state.agent_limiter.check(&rate_limit_key).allowed {
                        tracing::warn!(%agent_id, "agent rate limit exceeded");
                        break;
                    }

                    match serde_json::from_str::<AgentStreamMsg>(&text) {
                        Ok(stream_msg) => {
                            if let Err(e) = relay_agent_stream_message(
                                &state,
                                agent_id,
                                &mut stream_states,
                                stream_msg,
                            )
                            .await
                            {
                                tracing::warn!("failed to relay stream frame from {agent_id}: {e}");
                            }
                        }
                        Err(_) => match serde_json::from_str::<paw_proto::AgentResponseMsg>(&text) {
                            Ok(agent_msg) => {
                                tracing::info!(
                                    "agent {agent_id} response for conv {}: {} bytes",
                                    agent_msg.conversation_id,
                                    agent_msg.content.len()
                                );
                            }
                            Err(e) => {
                                tracing::warn!("invalid agent message from {agent_id}: {e}");
                            }
                        },
                    }
                }
                Message::Close(_) => break,
                _ => {}
            }
        }
    };

    tokio::select! {
        _ = nats_to_ws => {}
        _ = ws_to_server => {}
    }

    state.hub.unregister(agent_id, &outbound_tx).await;
    drop(outbound_tx);
    let _ = writer.await;
    crate::metrics::ws_connection_closed();
    tracing::info!("agent {agent_id} disconnected");
}

async fn relay_agent_stream_message(
    state: &AppState,
    agent_id: Uuid,
    stream_states: &mut HashMap<Uuid, StreamRelayState>,
    stream_msg: AgentStreamMsg,
) -> anyhow::Result<()> {
    match stream_msg {
        AgentStreamMsg::StreamStart(msg) => {
            require_v(msg.v)?;
            if msg.agent_id != agent_id {
                anyhow::bail!("agent_id mismatch for stream {}", msg.stream_id);
            }

            if stream_states.len() >= MAX_CONCURRENT_STREAMS_PER_AGENT {
                tracing::warn!("agent {agent_id} exceeded max concurrent streams");
                return Ok(());
            }

            let payload = serde_json::to_string(&ServerMessage::StreamStart(msg.clone()))?;
            let bytes = payload.len();
            if bytes > MAX_STREAM_BYTES {
                anyhow::bail!("stream start frame exceeds max bytes: {}", bytes);
            }

            stream_states.insert(
                msg.stream_id,
                StreamRelayState {
                    conversation_id: msg.conversation_id,
                    thread_id: msg.thread_id,
                    started_at: Instant::now(),
                    bytes_sent: bytes,
                },
            );

            let user_ids = conversation_members(state, msg.conversation_id).await?;
            state
                .hub
                .send_to_conversation(msg.conversation_id, user_ids, &payload)
                .await;

            broadcast_agent_typing(
                state,
                msg.conversation_id,
                msg.thread_id,
                msg.agent_id,
                true,
            )
            .await?;
        }
        AgentStreamMsg::AgentTypingStart(msg) => {
            require_v(msg.v)?;
            if msg.agent_id != agent_id {
                anyhow::bail!("agent_id mismatch for typing_start in conversation {}", msg.conversation_id);
            }
            broadcast_agent_typing(
                state,
                msg.conversation_id,
                msg.thread_id,
                msg.agent_id,
                true,
            )
            .await?;
        }
        AgentStreamMsg::AgentTypingEnd(msg) => {
            require_v(msg.v)?;
            if msg.agent_id != agent_id {
                anyhow::bail!("agent_id mismatch for typing_end in conversation {}", msg.conversation_id);
            }
            broadcast_agent_typing(
                state,
                msg.conversation_id,
                msg.thread_id,
                msg.agent_id,
                false,
            )
            .await?;
        }
        AgentStreamMsg::ContentDelta(msg) => {
            require_v(msg.v)?;
            if msg.delta.len() > MAX_DELTA_SIZE {
                tracing::warn!("content_delta from {agent_id} exceeds MAX_DELTA_SIZE, dropping");
                return Ok(());
            }
            relay_stream_frame(
                state,
                stream_states,
                msg.stream_id,
                ServerMessage::ContentDelta(msg),
                false,
                agent_id,
            )
            .await?;
        }
        AgentStreamMsg::ToolCallStart(msg) => {
            require_v(msg.v)?;
            persist_tool_call_start(state, agent_id, &msg).await?;
            relay_stream_frame(
                state,
                stream_states,
                msg.stream_id,
                ServerMessage::ToolCallStart(msg),
                false,
                agent_id,
            )
            .await?;
        }
        AgentStreamMsg::ToolCallResult(msg) => {
            require_v(msg.v)?;
            persist_tool_call_result(state, &msg).await?;
            relay_stream_frame(
                state,
                stream_states,
                msg.stream_id,
                ServerMessage::ToolCallResult(msg),
                false,
                agent_id,
            )
            .await?;
        }
        AgentStreamMsg::ToolCallEnd(msg) => {
            require_v(msg.v)?;
            persist_tool_call_end(state, &msg).await?;
            relay_stream_frame(
                state,
                stream_states,
                msg.stream_id,
                ServerMessage::ToolCallEnd(msg),
                false,
                agent_id,
            )
            .await?;
        }
        AgentStreamMsg::ToolStart(msg) => {
            require_v(msg.v)?;
            relay_stream_frame(
                state,
                stream_states,
                msg.stream_id,
                ServerMessage::ToolStart(msg),
                false,
                agent_id,
            )
            .await?;
        }
        AgentStreamMsg::ToolEnd(msg) => {
            require_v(msg.v)?;
            relay_stream_frame(
                state,
                stream_states,
                msg.stream_id,
                ServerMessage::ToolEnd(msg),
                false,
                agent_id,
            )
            .await?;
        }
        AgentStreamMsg::StreamEnd(msg) => {
            require_v(msg.v)?;
            let typing_context = stream_states
                .get(&msg.stream_id)
                .map(|state| (state.conversation_id, state.thread_id));
            relay_stream_frame(
                state,
                stream_states,
                msg.stream_id,
                ServerMessage::StreamEnd(msg),
                true,
                agent_id,
            )
            .await?;
            if let Some((conversation_id, thread_id)) = typing_context {
                broadcast_agent_typing(state, conversation_id, thread_id, agent_id, false).await?;
            }
        }
    }

    Ok(())
}

fn error(status: StatusCode, code: &str, locale: &str, message: &str) -> (StatusCode, Json<Value>) {
    error_response(status, code, locale, message)
}

fn error_with_details(
    status: StatusCode,
    code: &str,
    locale: &str,
    details: Option<&str>,
    message: &str,
) -> (StatusCode, Json<Value>) {
    error_response_with_details(status, code, locale, details, message)
}

fn agent_error_frame(code: &str, locale: &str, fallback: &str) -> Value {
    json!({
        "v": PROTOCOL_VERSION,
        "type": "error",
        "code": code,
        "message": localized_message(code, locale, fallback),
    })
}

async fn relay_stream_frame(
    state: &AppState,
    stream_states: &mut HashMap<Uuid, StreamRelayState>,
    stream_id: Uuid,
    frame: ServerMessage,
    finalize: bool,
    agent_id: Uuid,
) -> anyhow::Result<()> {
    let payload = serde_json::to_string(&frame)?;
    let payload_len = payload.len();
    let now = Instant::now();

    let Some(current) = stream_states.get(&stream_id).copied() else {
        tracing::warn!("dropping frame for unknown stream {stream_id}");
        return Ok(());
    };

    let elapsed = now.duration_since(current.started_at);
    if elapsed > MAX_STREAM_DURATION
        || current.bytes_sent.saturating_add(payload_len) > MAX_STREAM_BYTES
    {
        stream_states.remove(&stream_id);

        let end_payload = serde_json::to_string(&ServerMessage::StreamEnd(StreamEndMsg {
            v: PROTOCOL_VERSION,
            stream_id,
            tokens: 0,
            duration_ms: elapsed.as_millis().min(u64::MAX as u128) as u64,
        }))?;
        let user_ids = conversation_members(state, current.conversation_id).await?;
        state
            .hub
            .send_to_conversation(current.conversation_id, user_ids, &end_payload)
            .await;
        broadcast_agent_typing(
            state,
            current.conversation_id,
            current.thread_id,
            agent_id,
            false,
        )
        .await?;
        return Ok(());
    }

    if let Some(entry) = stream_states.get_mut(&stream_id) {
        entry.bytes_sent = entry.bytes_sent.saturating_add(payload_len);
    }

    let user_ids = conversation_members(state, current.conversation_id).await?;
    state
        .hub
        .send_to_conversation(current.conversation_id, user_ids, &payload)
        .await;

    if finalize {
        stream_states.remove(&stream_id);
    }

    Ok(())
}

async fn broadcast_agent_typing(
    state: &AppState,
    conversation_id: Uuid,
    thread_id: Option<Uuid>,
    agent_id: Uuid,
    is_start: bool,
) -> anyhow::Result<()> {
    let frame = if is_start {
        ServerMessage::AgentTypingStart(paw_proto::AgentTypingEventMsg {
            v: PROTOCOL_VERSION,
            conversation_id,
            thread_id,
            agent_id,
        })
    } else {
        ServerMessage::AgentTypingEnd(paw_proto::AgentTypingEventMsg {
            v: PROTOCOL_VERSION,
            conversation_id,
            thread_id,
            agent_id,
        })
    };

    let payload = serde_json::to_string(&frame)?;
    let user_ids = conversation_members(state, conversation_id).await?;
    state
        .hub
        .send_to_conversation(conversation_id, user_ids, &payload)
        .await;
    Ok(())
}

async fn persist_tool_call_start(
    state: &AppState,
    agent_id: Uuid,
    msg: &paw_proto::ToolCallStartMsg,
) -> anyhow::Result<()> {
    sqlx::query(
        "INSERT INTO agent_tool_calls (id, message_id, agent_id, tool_name, arguments, result, status, started_at, completed_at)
         VALUES ($1, NULL, $2, $3, $4, NULL, 'started', NOW(), NULL)
         ON CONFLICT (id) DO UPDATE
         SET agent_id = EXCLUDED.agent_id,
             tool_name = EXCLUDED.tool_name,
             arguments = EXCLUDED.arguments,
             status = 'started',
             started_at = NOW(),
             completed_at = NULL",
    )
    .bind(&msg.id)
    .bind(agent_id)
    .bind(&msg.name)
    .bind(&msg.arguments_json)
    .execute(state.db.as_ref())
    .await?;
    Ok(())
}

async fn persist_tool_call_result(
    state: &AppState,
    msg: &paw_proto::ToolCallResultMsg,
) -> anyhow::Result<()> {
    let status = if msg.is_error { "error" } else { "running" };
    sqlx::query(
        "UPDATE agent_tool_calls
         SET result = $2,
             status = $3
         WHERE id = $1",
    )
    .bind(&msg.id)
    .bind(&msg.result_json)
    .bind(status)
    .execute(state.db.as_ref())
    .await?;
    Ok(())
}

async fn persist_tool_call_end(state: &AppState, msg: &paw_proto::ToolCallEndMsg) -> anyhow::Result<()> {
    sqlx::query(
        "UPDATE agent_tool_calls
         SET status = CASE WHEN status = 'error' THEN 'error' ELSE 'completed' END,
             completed_at = NOW()
         WHERE id = $1",
    )
    .bind(&msg.id)
    .execute(state.db.as_ref())
    .await?;
    Ok(())
}

fn require_v(v: u8) -> anyhow::Result<()> {
    if v == PROTOCOL_VERSION {
        Ok(())
    } else {
        anyhow::bail!("unsupported protocol version: {v}")
    }
}

async fn conversation_members(
    state: &AppState,
    conversation_id: Uuid,
) -> anyhow::Result<Vec<Uuid>> {
    let rows: Vec<sqlx::postgres::PgRow> =
        sqlx::query("SELECT user_id FROM conversation_members WHERE conversation_id = $1")
            .bind(conversation_id)
            .fetch_all(state.db.as_ref())
            .await?;

    let mut users = Vec::with_capacity(rows.len());
    for row in rows {
        users.push(row.try_get::<Uuid, _>("user_id")?);
    }

    Ok(users)
}
