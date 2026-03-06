use axum::{
    Json,
    extract::{Query, State, WebSocketUpgrade},
    http::StatusCode,
    response::{IntoResponse, Response},
    Extension,
};
use futures_util::{SinkExt, StreamExt};
use std::collections::HashMap;

use crate::auth::AppState;
use crate::auth::middleware::UserId;
use super::models::{RegisterAgentRequest, RegisterAgentResponse};
use super::service;

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

    ws.on_upgrade(move |socket| handle_agent_socket(socket, agent_id, nats))
}

async fn handle_agent_socket(
    socket: axum::extract::ws::WebSocket,
    agent_id: uuid::Uuid,
    nats: std::sync::Arc<async_nats::Client>,
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
        while let Some(Ok(msg)) = ws_rx.next().await {
            match msg {
                Message::Text(text) => {
                    match serde_json::from_str::<paw_proto::AgentResponseMsg>(&text) {
                        Ok(agent_msg) => {
                            tracing::info!(
                                "agent {agent_id} response for conv {}: {} bytes",
                                agent_msg.conversation_id,
                                agent_msg.content.len()
                            );
                            // T29: insert into messages table via bot user system
                        }
                        Err(e) => {
                            tracing::warn!("invalid agent message from {agent_id}: {e}");
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
