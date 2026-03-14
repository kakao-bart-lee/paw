//! Paw WebSocket Protocol Types v1
//!
//! All messages MUST include the `v` field (currently `1`).
//! This enables future protocol evolution without breaking clients.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub const PROTOCOL_VERSION: u8 = 1;

/// All WebSocket frames implement this trait
pub trait PawMessage {
    fn version(&self) -> u8 {
        PROTOCOL_VERSION
    }
    fn message_type(&self) -> &str;
}

// ─── Envelope ────────────────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ClientMessage {
    Connect(ConnectMsg),
    MessageSend(MessageSendMsg),
    TypingStart(TypingMsg),
    TypingStop(TypingMsg),
    MessageAck(MessageAckMsg),
    Sync(SyncMsg),
    DeviceSync(DeviceSyncRequest),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    HelloOk(HelloOkMsg),
    HelloError(HelloErrorMsg),
    MessageReceived(MessageReceivedMsg),
    DeviceSyncResponse(DeviceSyncResponse),
    TypingStart(TypingMsg),
    TypingStop(TypingMsg),
    PresenceUpdate(PresenceUpdateMsg),
    // Phase 2 streaming (reserved, not implemented in Phase 1)
    StreamStart(StreamStartMsg),
    ContentDelta(ContentDeltaMsg),
    ToolStart(ToolStartMsg),
    ToolEnd(ToolEndMsg),
    StreamEnd(StreamEndMsg),
}

// ─── Client Messages ─────────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectMsg {
    pub v: u8,
    pub token: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSendMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    pub content: String,
    pub format: MessageFormat,
    #[serde(default)]
    pub blocks: Vec<serde_json::Value>,
    pub idempotency_key: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypingMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    /// Injected by server before fan-out; absent in client→server direction.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAckMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    pub last_seq: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    pub last_seq: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvSyncState {
    pub conversation_id: Uuid,
    pub last_seq: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSyncRequest {
    pub v: u8,
    pub conversations: Vec<ConvSyncState>,
}

// ─── Server Messages ──────────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloOkMsg {
    pub v: u8,
    pub user_id: Uuid,
    pub server_time: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloErrorMsg {
    pub v: u8,
    pub code: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReceivedMsg {
    pub v: u8,
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub sender_id: Uuid,
    pub content: String,
    pub format: MessageFormat,
    pub seq: i64,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub blocks: Vec<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSyncResponse {
    pub v: u8,
    pub messages: Vec<MessageReceivedMsg>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceUpdateMsg {
    pub v: u8,
    pub user_id: Uuid,
    pub online: bool,
}

// ─── Phase 2 Streaming (types reserved, not used in Phase 1) ─────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamStartMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    pub agent_id: Uuid,
    pub stream_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentDeltaMsg {
    pub v: u8,
    pub stream_id: Uuid,
    pub delta: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolStartMsg {
    pub v: u8,
    pub stream_id: Uuid,
    pub tool: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolEndMsg {
    pub v: u8,
    pub stream_id: Uuid,
    pub tool: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamEndMsg {
    pub v: u8,
    pub stream_id: Uuid,
    pub tokens: u32,
    pub duration_ms: u64,
}

// ─── Agent Gateway ───────────────────────────────────────────────────────

/// Agent Gateway: context sent to agent when a user sends a message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboundContext {
    pub v: u8,
    pub message: MessageReceivedMsg,
    pub conversation_id: Uuid,
    pub recent_messages: Vec<MessageReceivedMsg>,
}

/// Agent → Server: agent's response message
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResponseMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    pub content: String,
    pub format: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentStreamMsg {
    StreamStart(StreamStartMsg),
    ContentDelta(ContentDeltaMsg),
    ToolStart(ToolStartMsg),
    ToolEnd(ToolEndMsg),
    StreamEnd(StreamEndMsg),
}

// ─── Shared Enums ────────────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MessageFormat {
    #[default]
    Markdown,
    Plain,
}

// ─── Tests ───────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect_msg_has_version() {
        let msg = ConnectMsg {
            v: 1,
            token: "test".into(),
        };
        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["v"], 1, "All messages must include v field");
    }

    #[test]
    fn test_message_send_roundtrip() {
        let msg = MessageSendMsg {
            v: 1,
            conversation_id: Uuid::new_v4(),
            content: "Hello, Paw!".into(),
            format: MessageFormat::Markdown,
            blocks: vec![],
            idempotency_key: Uuid::new_v4(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let _: MessageSendMsg = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn test_server_message_version_required() {
        let json = r#"{"type": "hello_ok", "user_id": "550e8400-e29b-41d4-a716-446655440000", "server_time": "2026-01-01T00:00:00Z"}"#;
        // Without v field - should fail (v is required)
        let result: Result<HelloOkMsg, _> = serde_json::from_str(json);
        assert!(result.is_err(), "Messages without v field must be rejected");
    }

    #[test]
    fn test_device_sync_request_roundtrip() {
        let conversation_id = Uuid::new_v4();
        let msg = ClientMessage::DeviceSync(DeviceSyncRequest {
            v: PROTOCOL_VERSION,
            conversations: vec![ConvSyncState {
                conversation_id,
                last_seq: 17,
            }],
        });

        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["type"], "device_sync");
        assert_eq!(json["v"], PROTOCOL_VERSION);

        let parsed: ClientMessage = serde_json::from_value(json).unwrap();
        match parsed {
            ClientMessage::DeviceSync(request) => {
                assert_eq!(request.v, PROTOCOL_VERSION);
                assert_eq!(request.conversations.len(), 1);
                assert_eq!(request.conversations[0].conversation_id, conversation_id);
                assert_eq!(request.conversations[0].last_seq, 17);
            }
            _ => panic!("expected DeviceSync variant"),
        }
    }
}
