use crate::auth::AppState;
use crate::ws::hub::WsSender;
use axum::extract::ws::{CloseFrame, Message, WebSocket, close_code};
use chrono::{DateTime, Utc};
use futures_util::{SinkExt, StreamExt};
use paw_proto::{
    ClientMessage, HelloErrorMsg, HelloOkMsg, MessageFormat, MessageReceivedMsg, PROTOCOL_VERSION,
    ServerMessage,
};
use sqlx::Row;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use uuid::Uuid;

pub struct WsConnection {
    pub user_id: Uuid,
    pub device_id: Uuid,
    pub connected_at: DateTime<Utc>,
}

pub async fn handle_socket(socket: WebSocket, user_id: Uuid, device_id: Uuid, state: AppState) {
    let connection = WsConnection {
        user_id,
        device_id,
        connected_at: Utc::now(),
    };

    tracing::debug!(
        user_id = %connection.user_id,
        device_id = %connection.device_id,
        connected_at = %connection.connected_at,
        "websocket connected"
    );

    let (outbound_tx, mut outbound_rx) = mpsc::unbounded_channel::<Message>();
    state.hub.register(connection.user_id, outbound_tx.clone()).await;

    if let Ok(frame) = serde_json::to_string(&ServerMessage::HelloOk(HelloOkMsg {
        v: PROTOCOL_VERSION,
        user_id: connection.user_id,
        server_time: Utc::now(),
    })) {
        let _ = outbound_tx.send(Message::Text(frame.into()));
    }

    let (mut ws_sender, mut ws_receiver) = socket.split();

    let writer = tokio::spawn(async move {
        while let Some(msg) = outbound_rx.recv().await {
            if ws_sender.send(msg).await.is_err() {
                break;
            }
        }
    });

    let mut ping = tokio::time::interval(Duration::from_secs(crate::ws::HEARTBEAT_PING_SECONDS));
    let mut timeout_check =
        tokio::time::interval(Duration::from_secs(crate::ws::HEARTBEAT_PING_SECONDS));
    let mut last_pong = Instant::now();

    loop {
        tokio::select! {
            _ = ping.tick() => {
                if outbound_tx.send(Message::Ping(Vec::new().into())).is_err() {
                    break;
                }
            }
            _ = timeout_check.tick() => {
                if last_pong.elapsed() > Duration::from_secs(crate::ws::HEARTBEAT_TIMEOUT_SECONDS) {
                    let _ = outbound_tx.send(Message::Close(Some(CloseFrame {
                        code: close_code::NORMAL,
                        reason: "heartbeat timeout".into(),
                    })));
                    break;
                }
            }
            maybe_msg = ws_receiver.next() => {
                match maybe_msg {
                    Some(Ok(Message::Text(text))) => {
                        match serde_json::from_str::<ClientMessage>(&text) {
                            Ok(client_msg) => {
                                if let Err(err) = handle_client_message(&state, connection.user_id, &outbound_tx, client_msg).await {
                                    tracing::warn!(%err, user_id = %connection.user_id, "failed to handle client ws message");
                                }
                            }
                            Err(err) => {
                                let _ = send_protocol_error(&outbound_tx, "invalid_frame", format!("invalid json frame: {err}")).await;
                            }
                        }
                    }
                    Some(Ok(Message::Pong(_))) => {
                        last_pong = Instant::now();
                    }
                    Some(Ok(Message::Ping(payload))) => {
                        let _ = outbound_tx.send(Message::Pong(payload));
                    }
                    Some(Ok(Message::Close(_))) => break,
                    Some(Ok(Message::Binary(_))) => {
                        let _ = send_protocol_error(&outbound_tx, "invalid_frame", "binary frames are not supported".to_owned()).await;
                    }
                    Some(Err(err)) => {
                        tracing::debug!(%err, user_id = %connection.user_id, "websocket receive error");
                        break;
                    }
                    None => break,
                }
            }
        }
    }

    state.hub.unregister(connection.user_id, &outbound_tx).await;
    drop(outbound_tx);
    let _ = writer.await;
}

async fn handle_client_message(
    state: &AppState,
    user_id: Uuid,
    outbound_tx: &WsSender,
    msg: ClientMessage,
) -> anyhow::Result<()> {
    match msg {
        ClientMessage::Connect(connect) => {
            require_v(connect.v)?;
        }
        ClientMessage::MessageSend(message_send) => {
            require_v(message_send.v)?;
            tracing::debug!(
                user_id = %user_id,
                conversation_id = %message_send.conversation_id,
                "message_send received; persistence is handled by HTTP flow"
            );
        }
        ClientMessage::TypingStart(mut typing) => {
            require_v(typing.v)?;
            typing.user_id = Some(user_id);
            let others: Vec<Uuid> = conversation_members(state, typing.conversation_id)
                .await?
                .into_iter()
                .filter(|&m| m != user_id)
                .collect();
            let payload = serde_json::to_string(&ServerMessage::TypingStart(typing))?;
            state.hub.broadcast_to_conversation(others, &payload).await;
        }
        ClientMessage::TypingStop(mut typing) => {
            require_v(typing.v)?;
            typing.user_id = Some(user_id);
            let others: Vec<Uuid> = conversation_members(state, typing.conversation_id)
                .await?
                .into_iter()
                .filter(|&m| m != user_id)
                .collect();
            let payload = serde_json::to_string(&ServerMessage::TypingStop(typing))?;
            state.hub.broadcast_to_conversation(others, &payload).await;
        }
        ClientMessage::MessageAck(ack) => {
            require_v(ack.v)?;
            sqlx::query(
                "UPDATE conversation_members SET last_read_seq = GREATEST(last_read_seq, $3) WHERE conversation_id = $1 AND user_id = $2",
            )
            .bind(ack.conversation_id)
            .bind(user_id)
            .bind(ack.last_seq)
            .execute(state.db.as_ref())
            .await?;
        }
        ClientMessage::Sync(sync) => {
            require_v(sync.v)?;
            let rows: Vec<sqlx::postgres::PgRow> = sqlx::query(
                "SELECT id, conversation_id, sender_id, content, format, seq, created_at, blocks \
                 FROM messages \
                 WHERE conversation_id = $1 AND seq > $2 \
                 ORDER BY seq ASC",
            )
            .bind(sync.conversation_id)
            .bind(sync.last_seq)
            .fetch_all(state.db.as_ref())
            .await?;

            for row in rows {
                let format_raw: Option<String> = row.try_get::<Option<String>, _>("format")?;
                let format = match format_raw
                    .unwrap_or_else(|| "markdown".to_owned())
                    .to_ascii_lowercase()
                    .as_str()
                {
                    "plain" => MessageFormat::Plain,
                    _ => MessageFormat::Markdown,
                };

                let blocks: Option<serde_json::Value> = row.try_get::<Option<serde_json::Value>, _>("blocks")?;
                let blocks = match blocks {
                    Some(serde_json::Value::Array(values)) => values,
                    _ => Vec::new(),
                };

                let server_frame = ServerMessage::MessageReceived(MessageReceivedMsg {
                    v: PROTOCOL_VERSION,
                    id: row.try_get("id")?,
                    conversation_id: row.try_get("conversation_id")?,
                    sender_id: row.try_get("sender_id")?,
                    content: row.try_get("content")?,
                    format,
                    seq: row.try_get("seq")?,
                    created_at: row.try_get("created_at")?,
                    blocks,
                });

                let payload = serde_json::to_string(&server_frame)?;
                if outbound_tx.send(Message::Text(payload.into())).is_err() {
                    break;
                }
            }
        }
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

async fn send_protocol_error(outbound_tx: &WsSender, code: &str, message: String) -> anyhow::Result<()> {
    let payload = serde_json::to_string(&ServerMessage::HelloError(HelloErrorMsg {
        v: PROTOCOL_VERSION,
        code: code.to_owned(),
        message,
    }))?;
    let _ = outbound_tx.send(Message::Text(payload.into()));
    Ok(())
}
