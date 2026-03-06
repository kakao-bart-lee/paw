use axum::{
    Json,
    extract::{Path, Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension,
};
use futures_util::{SinkExt, StreamExt};
use paw_proto::{AgentStreamMsg, PROTOCOL_VERSION, ServerMessage, StreamEndMsg};
use serde::Serialize;
use serde_json::{Value, json};
use sqlx::Row;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use uuid::Uuid;

use crate::auth::AppState;
use crate::auth::middleware::UserId;
use crate::messages::service::{Membership, check_member};
use super::models::{
    AgentProfile, InviteAgentRequest, InviteAgentResponse, RegisterAgentRequest,
    RegisterAgentResponse, RevokeAgentResponse,
};
use super::service;

const MAX_STREAM_DURATION: Duration = Duration::from_secs(300);
const MAX_STREAM_BYTES: usize = 1_048_576;
pub const MAX_CONCURRENT_STREAMS_PER_AGENT: usize = 10;
pub const MAX_DELTA_SIZE: usize = 4096;

#[derive(Clone, Copy)]
struct StreamRelayState {
    conversation_id: Uuid,
    started_at: Instant,
    bytes_sent: usize,
}

#[derive(Debug, Serialize)]
pub struct RemoveAgentResponse {
    pub removed: bool,
}

pub async fn register_agent_handler(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Json(req): Json<RegisterAgentRequest>,
) -> Result<Json<RegisterAgentResponse>, (StatusCode, Json<serde_json::Value>)> {
    let resp = service::register_agent(&state.db, user_id.0, req)
        .await
        .map_err(|e| {
            tracing::error!("failed to register agent: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "registration_failed" })),
            )
        })?;

    Ok(Json(resp))
}

pub async fn get_agent_handler(
    State(state): State<AppState>,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<AgentProfile>, (StatusCode, Json<serde_json::Value>)> {
    let profile = service::get_agent_profile(&state.db, agent_id)
        .await
        .map_err(|e| {
            tracing::error!("failed to fetch agent profile: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "internal_error" })),
            )
        })?;

    match profile {
        Some(p) => Ok(Json(p)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "agent_not_found" })),
        )),
    }
}

pub async fn revoke_agent_handler(
    State(state): State<AppState>,
    Extension(user_id): Extension<UserId>,
    Path(agent_id): Path<Uuid>,
) -> Result<Json<RevokeAgentResponse>, (StatusCode, Json<serde_json::Value>)> {
    let result = service::revoke_agent_token(&state.db, agent_id, user_id.0)
        .await
        .map_err(|e| {
            tracing::error!("failed to revoke agent token: {e}");
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({ "error": "internal_error" })),
            )
        })?;

    match result {
        Some(r) => Ok(Json(r)),
        None => Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({ "error": "agent_not_found_or_not_owner" })),
        )),
    }
}

pub async fn invite_agent_handler(
    State(state): State<AppState>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path(conversation_id): Path<Uuid>,
    Json(payload): Json<InviteAgentRequest>,
) -> Result<Json<InviteAgentResponse>, (StatusCode, Json<Value>)> {
    match check_member(&state.db, conversation_id, user_id).await {
        Ok(Membership::Member) => {}
        Ok(Membership::NotMember) => {
            return Err((
                StatusCode::FORBIDDEN,
                Json(json!({
                    "error": "forbidden",
                    "message": "User is not a member of this conversation"
                })),
            ));
        }
        Ok(Membership::ConversationNotFound) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(json!({
                    "error": "conversation_not_found",
                    "message": "Conversation not found"
                })),
            ));
        }
        Err(err) => {
            tracing::error!(%err, conversation_id = %conversation_id, user_id = %user_id, "failed membership check");
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "membership_check_failed",
                    "message": "Could not validate conversation membership"
                })),
            ));
        }
    }

    match service::invite_agent_to_conversation(&state.db, conversation_id, payload.agent_id, user_id).await {
        Ok(true) => Ok(Json(InviteAgentResponse { invited: true })),
        Ok(false) => Err((
            StatusCode::CONFLICT,
            Json(json!({
                "error": "already_invited",
                "message": "Agent is already invited to this conversation"
            })),
        )),
        Err(err) if err.to_string() == "agent_not_found_or_revoked" => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "agent_not_found",
                "message": "Agent not found or revoked"
            })),
        )),
        Err(err) => {
            tracing::error!(%err, conversation_id = %conversation_id, agent_id = %payload.agent_id, "failed to invite agent");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "invite_failed",
                    "message": "Failed to invite agent"
                })),
            ))
        }
    }
}

pub async fn remove_agent_handler(
    State(state): State<AppState>,
    Extension(UserId(user_id)): Extension<UserId>,
    Path((conversation_id, agent_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<RemoveAgentResponse>, (StatusCode, Json<Value>)> {
    match service::remove_agent_from_conversation(&state.db, conversation_id, agent_id, user_id).await {
        Ok(true) => Ok(Json(RemoveAgentResponse { removed: true })),
        Ok(false) => Err((
            StatusCode::NOT_FOUND,
            Json(json!({
                "error": "agent_not_found",
                "message": "Agent is not in this conversation"
            })),
        )),
        Err(err) if err.to_string() == "not_owner" => Err((
            StatusCode::FORBIDDEN,
            Json(json!({
                "error": "forbidden",
                "message": "Only conversation owners can remove agents"
            })),
        )),
        Err(err) => {
            tracing::error!(%err, conversation_id = %conversation_id, agent_id = %agent_id, user_id = %user_id, "failed to remove agent");
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(json!({
                    "error": "remove_failed",
                    "message": "Failed to remove agent"
                })),
            ))
        }
    }
}

pub async fn agent_ws_handler(
    ws: WebSocketUpgrade,
    Query(params): Query<HashMap<String, String>>,
    State(state): State<AppState>,
) -> Response {
    let raw_token = match params.get("token") {
        Some(t) if !t.is_empty() => t.clone(),
        _ => return (StatusCode::UNAUTHORIZED, "missing token").into_response(),
    };

    let agent_id = match service::verify_agent_token(&state.db, &raw_token).await {
        Ok(Some(id)) => id,
        Ok(None) => return (StatusCode::UNAUTHORIZED, "invalid agent token").into_response(),
        Err(e) => {
            tracing::error!("agent token verification failed: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "internal error").into_response();
        }
    };

    let nats = match &state.nats {
        Some(n) => n.clone(),
        None => {
            return ws.on_upgrade(move |mut socket| async move {
                use axum::extract::ws::Message;
                let err_frame = serde_json::json!({
                    "v": 1,
                    "type": "error",
                    "code": "nats_unavailable",
                    "message": "Agent gateway requires NATS"
                });
                let _ = socket
                    .send(Message::Text(err_frame.to_string().into()))
                    .await;
                let _ = socket.close().await;
            });
        }
    };

    let state_for_socket = state.clone();
    ws.on_upgrade(move |socket| handle_agent_socket(socket, agent_id, nats, state_for_socket))
}

async fn handle_agent_socket(
    socket: axum::extract::ws::WebSocket,
    agent_id: uuid::Uuid,
    nats: std::sync::Arc<async_nats::Client>,
    state: AppState,
) {
    use axum::extract::ws::Message;

    let (mut ws_tx, mut ws_rx) = socket.split();

    let subject = format!("agent.inbound.{agent_id}");
    let mut nats_sub = match nats.subscribe(subject.clone()).await {
        Ok(sub) => sub,
        Err(e) => {
            tracing::error!("NATS subscribe failed for {subject}: {e}");
            let err = serde_json::json!({
                "v": 1,
                "type": "error",
                "code": "subscribe_failed"
            });
            let _ = ws_tx.send(Message::Text(err.to_string().into())).await;
            return;
        }
    };

    tracing::info!("agent {agent_id} connected, subscribed to {subject}");

    let nats_to_ws = async {
        while let Some(msg) = nats_sub.next().await {
            let payload = String::from_utf8_lossy(&msg.payload);
            if ws_tx
                .send(Message::Text(payload.into_owned().into()))
                .await
                .is_err()
            {
                break;
            }
        }
    };

    let ws_to_server = async {
        let mut stream_states: HashMap<Uuid, StreamRelayState> = HashMap::new();

        while let Some(Ok(msg)) = ws_rx.next().await {
            match msg {
                Message::Text(text) => {
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
                        }
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
                    started_at: Instant::now(),
                    bytes_sent: bytes,
                },
            );

            let user_ids = conversation_members(state, msg.conversation_id).await?;
            state
                .hub
                .send_to_conversation(msg.conversation_id, user_ids, &payload)
                .await;
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
            )
            .await?;
        }
        AgentStreamMsg::StreamEnd(msg) => {
            require_v(msg.v)?;
            relay_stream_frame(
                state,
                stream_states,
                msg.stream_id,
                ServerMessage::StreamEnd(msg),
                true,
            )
            .await?;
        }
    }

    Ok(())
}

async fn relay_stream_frame(
    state: &AppState,
    stream_states: &mut HashMap<Uuid, StreamRelayState>,
    stream_id: Uuid,
    frame: ServerMessage,
    finalize: bool,
) -> anyhow::Result<()> {
    let payload = serde_json::to_string(&frame)?;
    let payload_len = payload.len();
    let now = Instant::now();

    let Some(current) = stream_states.get(&stream_id).copied() else {
        tracing::warn!("dropping frame for unknown stream {stream_id}");
        return Ok(());
    };

    let elapsed = now.duration_since(current.started_at);
    if elapsed > MAX_STREAM_DURATION || current.bytes_sent.saturating_add(payload_len) > MAX_STREAM_BYTES {
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

fn require_v(v: u8) -> anyhow::Result<()> {
    if v == PROTOCOL_VERSION {
        Ok(())
    } else {
        anyhow::bail!("unsupported protocol version: {v}")
    }
}

async fn conversation_members(state: &AppState, conversation_id: Uuid) -> anyhow::Result<Vec<Uuid>> {
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
