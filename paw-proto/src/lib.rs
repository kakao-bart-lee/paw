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
    SendThreadMessage(ThreadMessageSendMsg),
    TypingStart(TypingMsg),
    TypingStop(TypingMsg),
    TypingThreadStart(ThreadTypingMsg),
    TypingThreadEnd(ThreadTypingMsg),
    MessageAck(MessageAckMsg),
    Sync(SyncMsg),
    DeviceSync(DeviceSyncRequest),
    ThreadSubscribe(ThreadSubscriptionMsg),
    ThreadUnsubscribe(ThreadSubscriptionMsg),
    ThreadCreate(ThreadCreateMsg),
    ThreadBindAgent(ThreadBindAgentMsg),
    ThreadUnbindAgent(ThreadUnbindAgentMsg),
    ThreadDelete(ThreadDeleteMsg),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ServerMessage {
    HelloOk(HelloOkMsg),
    HelloError(HelloErrorMsg),
    Error(ErrorMsg),
    MessageReceived(MessageReceivedMsg),
    ThreadMessageReceived(ThreadMessageReceivedMsg),
    MessageForwarded(MessageForwardedMsg),
    DeviceSyncResponse(DeviceSyncResponse),
    TypingStart(TypingMsg),
    TypingStop(TypingMsg),
    TypingThreadStart(ThreadTypingMsg),
    TypingThreadEnd(ThreadTypingMsg),
    PresenceUpdate(PresenceUpdateMsg),
    ThreadCreated(ThreadCreatedMsg),
    ThreadAgentBound(ThreadAgentBoundMsg),
    ThreadAgentUnbound(ThreadAgentUnboundMsg),
    ThreadDeleted(ThreadDeletedMsg),
    // Phase 2 streaming (reserved, not implemented in Phase 1)
    StreamStart(StreamStartMsg),
    AgentTypingStart(AgentTypingEventMsg),
    AgentTypingEnd(AgentTypingEventMsg),
    ContentDelta(ContentDeltaMsg),
    ToolCallStart(ToolCallStartMsg),
    ToolCallResult(ToolCallResultMsg),
    ToolCallEnd(ToolCallEndMsg),
    ToolStart(ToolStartMsg),
    ToolEnd(ToolEndMsg),
    StreamEnd(StreamEndMsg),
}

// ─── Client Messages ─────────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectMsg {
    pub v: u8,
    pub token: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub capabilities: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageSendMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub thread_id: Option<Uuid>,
    pub content: String,
    pub format: MessageFormat,
    #[serde(default)]
    pub blocks: Vec<serde_json::Value>,
    pub idempotency_key: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadMessageSendMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    pub thread_id: Uuid,
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
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub thread_id: Option<Uuid>,
    /// Injected by server before fan-out; absent in client→server direction.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadTypingMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    pub thread_id: Uuid,
    /// Injected by server before fan-out; absent in client→server direction.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub user_id: Option<Uuid>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadSubscriptionMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    pub thread_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAckMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub thread_id: Option<Uuid>,
    pub last_seq: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub thread_id: Option<Uuid>,
    pub last_seq: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadSyncEntry {
    pub thread_id: Uuid,
    pub last_seq: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvSyncState {
    pub conversation_id: Uuid,
    pub last_seq: i64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub threads: Vec<ThreadSyncEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSyncRequest {
    pub v: u8,
    pub conversations: Vec<ConvSyncState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadCreateMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    pub root_message_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub title: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadBindAgentMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    pub thread_id: Uuid,
    pub agent_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadUnbindAgentMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    pub thread_id: Uuid,
    pub agent_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadDeleteMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    pub thread_id: Uuid,
}

// ─── Server Messages ──────────────────────────────────────────────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloOkMsg {
    pub v: u8,
    pub user_id: Uuid,
    pub server_time: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub capabilities: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelloErrorMsg {
    pub v: u8,
    pub code: String,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub details: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMsg {
    pub v: u8,
    pub code: String,
    pub ref_type: String,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageAttachment {
    pub id: Uuid,
    pub file_type: String,
    pub file_url: String,
    pub file_size: i64,
    pub mime_type: String,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub thumbnail_url: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageReceivedMsg {
    pub v: u8,
    pub id: Uuid,
    pub conversation_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub thread_id: Option<Uuid>,
    pub sender_id: Uuid,
    pub content: String,
    pub format: MessageFormat,
    pub seq: i64,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub blocks: Vec<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<MessageAttachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadMessageReceivedMsg {
    pub v: u8,
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub thread_id: Uuid,
    pub sender_id: Uuid,
    pub content: String,
    pub format: MessageFormat,
    pub seq: i64,
    pub conversation_seq: i64,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub blocks: Vec<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<MessageAttachment>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardedFrom {
    pub original_message_id: Uuid,
    pub source_conversation_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageForwardedMsg {
    pub v: u8,
    pub id: Uuid,
    pub conversation_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub thread_id: Option<Uuid>,
    pub sender_id: Uuid,
    pub content: String,
    pub format: MessageFormat,
    pub seq: i64,
    pub created_at: DateTime<Utc>,
    #[serde(default)]
    pub blocks: Vec<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub attachments: Vec<MessageAttachment>,
    pub forwarded_from: ForwardedFrom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSyncResponse {
    pub v: u8,
    #[serde(default)]
    pub conversations: Vec<ConvSyncState>,
    pub messages: Vec<MessageReceivedMsg>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PresenceUpdateMsg {
    pub v: u8,
    pub user_id: Uuid,
    pub online: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadCreatedMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    pub thread_id: Uuid,
    pub root_message_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub title: Option<String>,
    pub created_by: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadAgentBoundMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    pub thread_id: Uuid,
    pub agent_id: Uuid,
    pub bound_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadAgentUnboundMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    pub thread_id: Uuid,
    pub agent_id: Uuid,
    pub unbound_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreadDeletedMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    pub thread_id: Uuid,
    pub deleted_by: Uuid,
    pub deleted_at: DateTime<Utc>,
}

// ─── Phase 2 Streaming (types reserved, not used in Phase 1) ─────────────
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamStartMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub thread_id: Option<Uuid>,
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
pub struct AgentTypingEventMsg {
    pub v: u8,
    pub conversation_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub thread_id: Option<Uuid>,
    pub agent_id: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallStartMsg {
    pub v: u8,
    pub stream_id: Uuid,
    pub id: String,
    pub name: String,
    pub arguments_json: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallResultMsg {
    pub v: u8,
    pub stream_id: Uuid,
    pub id: String,
    pub result_json: serde_json::Value,
    pub is_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolCallEndMsg {
    pub v: u8,
    pub stream_id: Uuid,
    pub id: String,
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
pub struct ContextEvent {
    pub event_type: String,
    pub conversation_id: Uuid,
    pub data: serde_json::Value,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AgentStreamMsg {
    StreamStart(StreamStartMsg),
    AgentTypingStart(AgentTypingEventMsg),
    AgentTypingEnd(AgentTypingEventMsg),
    ContentDelta(ContentDeltaMsg),
    ToolCallStart(ToolCallStartMsg),
    ToolCallResult(ToolCallResultMsg),
    ToolCallEnd(ToolCallEndMsg),
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
    use serde_json::json;

    #[test]
    fn test_connect_msg_has_version() {
        let msg = ConnectMsg {
            v: 1,
            token: "test".into(),
            capabilities: None,
        };
        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["v"], 1, "All messages must include v field");
    }

    #[test]
    fn test_message_send_roundtrip() {
        let msg = MessageSendMsg {
            v: 1,
            conversation_id: Uuid::new_v4(),
            thread_id: None,
            content: "Hello, Paw!".into(),
            format: MessageFormat::Markdown,
            blocks: vec![],
            idempotency_key: Uuid::new_v4(),
        };
        let json = serde_json::to_string(&msg).unwrap();
        let _: MessageSendMsg = serde_json::from_str(&json).unwrap();
    }

    #[test]
    fn test_server_message_forwarded_roundtrip() {
        let conversation_id = Uuid::new_v4();
        let source_conversation_id = Uuid::new_v4();
        let original_message_id = Uuid::new_v4();
        let msg = ServerMessage::MessageForwarded(MessageForwardedMsg {
            v: PROTOCOL_VERSION,
            id: Uuid::new_v4(),
            conversation_id,
            thread_id: None,
            sender_id: Uuid::new_v4(),
            content: "Forwarded hello".to_owned(),
            format: MessageFormat::Plain,
            seq: 7,
            created_at: Utc::now(),
            blocks: Vec::new(),
            attachments: Vec::new(),
            forwarded_from: ForwardedFrom {
                original_message_id,
                source_conversation_id,
            },
        });

        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["type"], "message_forwarded");
        assert_eq!(
            json["forwarded_from"]["original_message_id"],
            original_message_id.to_string()
        );
        assert_eq!(
            json["forwarded_from"]["source_conversation_id"],
            source_conversation_id.to_string()
        );

        let parsed: ServerMessage = serde_json::from_value(json).unwrap();
        match parsed {
            ServerMessage::MessageForwarded(forwarded) => {
                assert_eq!(forwarded.conversation_id, conversation_id);
                assert_eq!(
                    forwarded.forwarded_from.original_message_id,
                    original_message_id
                );
                assert_eq!(
                    forwarded.forwarded_from.source_conversation_id,
                    source_conversation_id
                );
            }
            _ => panic!("expected MessageForwarded variant"),
        }
    }

    #[test]
    fn test_message_received_attachments_roundtrip() {
        let attachment_id = Uuid::new_v4();
        let message = ServerMessage::MessageReceived(MessageReceivedMsg {
            v: PROTOCOL_VERSION,
            id: Uuid::new_v4(),
            conversation_id: Uuid::new_v4(),
            thread_id: None,
            sender_id: Uuid::new_v4(),
            content: "look at this".to_owned(),
            format: MessageFormat::Markdown,
            seq: 2,
            created_at: Utc::now(),
            blocks: Vec::new(),
            attachments: vec![MessageAttachment {
                id: attachment_id,
                file_type: "image".to_owned(),
                file_url: "media/user/image.png".to_owned(),
                file_size: 1234,
                mime_type: "image/png".to_owned(),
                thumbnail_url: Some("media/user/image-thumb.png".to_owned()),
            }],
        });

        let json = serde_json::to_value(&message).unwrap();
        assert_eq!(json["type"], "message_received");
        assert_eq!(json["attachments"][0]["id"], attachment_id.to_string());
        assert_eq!(json["attachments"][0]["mime_type"], "image/png");

        let parsed: ServerMessage = serde_json::from_value(json).unwrap();
        match parsed {
            ServerMessage::MessageReceived(frame) => {
                assert_eq!(frame.attachments.len(), 1);
                assert_eq!(frame.attachments[0].id, attachment_id);
                assert_eq!(frame.attachments[0].file_type, "image");
            }
            _ => panic!("expected MessageReceived variant"),
        }
    }

    #[test]
    fn test_send_thread_message_roundtrip() {
        let conversation_id = Uuid::new_v4();
        let thread_id = Uuid::new_v4();
        let msg = ClientMessage::SendThreadMessage(ThreadMessageSendMsg {
            v: PROTOCOL_VERSION,
            conversation_id,
            thread_id,
            content: "Hello thread".to_owned(),
            format: MessageFormat::Markdown,
            blocks: Vec::new(),
            idempotency_key: Uuid::new_v4(),
        });

        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["type"], "send_thread_message");
        assert_eq!(json["conversation_id"], conversation_id.to_string());
        assert_eq!(json["thread_id"], thread_id.to_string());

        let parsed: ClientMessage = serde_json::from_value(json).unwrap();
        match parsed {
            ClientMessage::SendThreadMessage(frame) => {
                assert_eq!(frame.v, PROTOCOL_VERSION);
                assert_eq!(frame.conversation_id, conversation_id);
                assert_eq!(frame.thread_id, thread_id);
                assert_eq!(frame.content, "Hello thread");
            }
            _ => panic!("expected SendThreadMessage variant"),
        }
    }

    #[test]
    fn test_thread_typing_roundtrip() {
        let conversation_id = Uuid::new_v4();
        let thread_id = Uuid::new_v4();
        let msg = ClientMessage::TypingThreadStart(ThreadTypingMsg {
            v: PROTOCOL_VERSION,
            conversation_id,
            thread_id,
            user_id: None,
        });

        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["type"], "typing_thread_start");
        assert_eq!(json["thread_id"], thread_id.to_string());

        let parsed: ClientMessage = serde_json::from_value(json).unwrap();
        match parsed {
            ClientMessage::TypingThreadStart(frame) => {
                assert_eq!(frame.v, PROTOCOL_VERSION);
                assert_eq!(frame.conversation_id, conversation_id);
                assert_eq!(frame.thread_id, thread_id);
                assert!(frame.user_id.is_none());
            }
            _ => panic!("expected TypingThreadStart variant"),
        }
    }

    #[test]
    fn test_thread_typing_server_event_roundtrip() {
        let conversation_id = Uuid::new_v4();
        let thread_id = Uuid::new_v4();
        let user_id = Uuid::new_v4();
        let msg = ServerMessage::TypingThreadEnd(ThreadTypingMsg {
            v: PROTOCOL_VERSION,
            conversation_id,
            thread_id,
            user_id: Some(user_id),
        });

        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["type"], "typing_thread_end");
        assert_eq!(json["thread_id"], thread_id.to_string());
        assert_eq!(json["user_id"], user_id.to_string());

        let parsed: ServerMessage = serde_json::from_value(json).unwrap();
        match parsed {
            ServerMessage::TypingThreadEnd(frame) => {
                assert_eq!(frame.conversation_id, conversation_id);
                assert_eq!(frame.thread_id, thread_id);
                assert_eq!(frame.user_id, Some(user_id));
            }
            _ => panic!("expected TypingThreadEnd variant"),
        }
    }

    #[test]
    fn test_thread_subscribe_roundtrip() {
        let conversation_id = Uuid::new_v4();
        let thread_id = Uuid::new_v4();
        let msg = ClientMessage::ThreadSubscribe(ThreadSubscriptionMsg {
            v: PROTOCOL_VERSION,
            conversation_id,
            thread_id,
        });

        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["type"], "thread_subscribe");
        assert_eq!(json["thread_id"], thread_id.to_string());

        let parsed: ClientMessage = serde_json::from_value(json).unwrap();
        match parsed {
            ClientMessage::ThreadSubscribe(frame) => {
                assert_eq!(frame.v, PROTOCOL_VERSION);
                assert_eq!(frame.conversation_id, conversation_id);
                assert_eq!(frame.thread_id, thread_id);
            }
            _ => panic!("expected ThreadSubscribe variant"),
        }
    }

    #[test]
    fn test_thread_message_received_roundtrip() {
        let conversation_id = Uuid::new_v4();
        let thread_id = Uuid::new_v4();
        let attachment_id = Uuid::new_v4();
        let msg = ServerMessage::ThreadMessageReceived(ThreadMessageReceivedMsg {
            v: PROTOCOL_VERSION,
            id: Uuid::new_v4(),
            conversation_id,
            thread_id,
            sender_id: Uuid::new_v4(),
            content: "thread update".to_owned(),
            format: MessageFormat::Plain,
            seq: 4,
            conversation_seq: 19,
            created_at: Utc::now(),
            blocks: Vec::new(),
            attachments: vec![MessageAttachment {
                id: attachment_id,
                file_type: "image".to_owned(),
                file_url: "media/thread/image.png".to_owned(),
                file_size: 88,
                mime_type: "image/png".to_owned(),
                thumbnail_url: None,
            }],
        });

        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["type"], "thread_message_received");
        assert_eq!(json["thread_id"], thread_id.to_string());
        assert_eq!(json["seq"], 4);
        assert_eq!(json["conversation_seq"], 19);

        let parsed: ServerMessage = serde_json::from_value(json).unwrap();
        match parsed {
            ServerMessage::ThreadMessageReceived(frame) => {
                assert_eq!(frame.conversation_id, conversation_id);
                assert_eq!(frame.thread_id, thread_id);
                assert_eq!(frame.seq, 4);
                assert_eq!(frame.conversation_seq, 19);
                assert_eq!(frame.attachments.len(), 1);
                assert_eq!(frame.attachments[0].id, attachment_id);
            }
            _ => panic!("expected ThreadMessageReceived variant"),
        }
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
                threads: vec![ThreadSyncEntry {
                    thread_id: Uuid::new_v4(),
                    last_seq: 3,
                }],
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
                assert_eq!(request.conversations[0].threads.len(), 1);
                assert_eq!(request.conversations[0].threads[0].last_seq, 3);
            }
            _ => panic!("expected DeviceSync variant"),
        }
    }

    #[test]
    fn test_thread_create_roundtrip() {
        let conversation_id = Uuid::new_v4();
        let root_message_id = Uuid::new_v4();
        let msg = ClientMessage::ThreadCreate(ThreadCreateMsg {
            v: PROTOCOL_VERSION,
            conversation_id,
            root_message_id,
            title: Some("A thread".into()),
        });

        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["type"], "thread_create");
        assert_eq!(json["conversation_id"], conversation_id.to_string());
        assert_eq!(json["root_message_id"], root_message_id.to_string());
        assert_eq!(json["title"], "A thread");

        let parsed: ClientMessage = serde_json::from_value(json).unwrap();
        match parsed {
            ClientMessage::ThreadCreate(thread_create) => {
                assert_eq!(thread_create.v, PROTOCOL_VERSION);
                assert_eq!(thread_create.conversation_id, conversation_id);
                assert_eq!(thread_create.root_message_id, root_message_id);
                assert_eq!(thread_create.title.as_deref(), Some("A thread"));
            }
            _ => panic!("expected ThreadCreate variant"),
        }
    }

    #[test]
    fn test_thread_created_roundtrip() {
        let conversation_id = Uuid::new_v4();
        let thread_id = Uuid::new_v4();
        let root_message_id = Uuid::new_v4();
        let created_by = Uuid::new_v4();
        let created_at = "2026-03-15T00:00:00Z".parse::<DateTime<Utc>>().unwrap();
        let msg = ServerMessage::ThreadCreated(ThreadCreatedMsg {
            v: PROTOCOL_VERSION,
            conversation_id,
            thread_id,
            root_message_id,
            title: Some("Threads".into()),
            created_by,
            created_at,
        });

        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["type"], "thread_created");
        assert_eq!(json["thread_id"], thread_id.to_string());
        assert_eq!(json["created_by"], created_by.to_string());

        let parsed: ServerMessage = serde_json::from_value(json).unwrap();
        match parsed {
            ServerMessage::ThreadCreated(thread_created) => {
                assert_eq!(thread_created.v, PROTOCOL_VERSION);
                assert_eq!(thread_created.conversation_id, conversation_id);
                assert_eq!(thread_created.thread_id, thread_id);
                assert_eq!(thread_created.root_message_id, root_message_id);
                assert_eq!(thread_created.title.as_deref(), Some("Threads"));
                assert_eq!(thread_created.created_by, created_by);
                assert_eq!(thread_created.created_at, created_at);
            }
            _ => panic!("expected ThreadCreated variant"),
        }
    }

    #[test]
    fn test_error_roundtrip() {
        let msg = ServerMessage::Error(ErrorMsg {
            v: PROTOCOL_VERSION,
            code: "agent_already_bound".into(),
            ref_type: "thread_bind_agent".into(),
            message: "Agent is already bound to another thread in this conversation".into(),
        });

        let json = serde_json::to_value(&msg).unwrap();
        assert_eq!(json["type"], "error");
        assert_eq!(json["code"], "agent_already_bound");
        assert_eq!(json["ref_type"], "thread_bind_agent");

        let parsed: ServerMessage = serde_json::from_value(json).unwrap();
        match parsed {
            ServerMessage::Error(error) => {
                assert_eq!(error.v, PROTOCOL_VERSION);
                assert_eq!(error.code, "agent_already_bound");
                assert_eq!(error.ref_type, "thread_bind_agent");
            }
            _ => panic!("expected Error variant"),
        }
    }

    #[test]
    fn test_optional_thread_id_and_capabilities_deserialize_when_absent() {
        let connect_json = r#"{"v":1,"token":"jwt-token"}"#;
        let connect: ConnectMsg = serde_json::from_str(connect_json).unwrap();
        assert!(connect.capabilities.is_none());

        let sync_json =
            r#"{"v":1,"conversation_id":"550e8400-e29b-41d4-a716-446655440000","last_seq":42}"#;
        let sync: SyncMsg = serde_json::from_str(sync_json).unwrap();
        assert!(sync.thread_id.is_none());
    }

    #[test]
    fn test_context_event_message_created_roundtrip() {
        let conversation_id = Uuid::new_v4();
        let event = ContextEvent {
            event_type: "message_created".to_owned(),
            conversation_id,
            data: json!({
                "message_id": Uuid::new_v4(),
                "sender_id": Uuid::new_v4(),
                "content": "hello agent"
            }),
            timestamp: "2026-03-16T00:00:00Z".parse::<DateTime<Utc>>().unwrap(),
        };

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["event_type"], "message_created");
        assert_eq!(json["conversation_id"], conversation_id.to_string());
        assert_eq!(json["data"]["content"], "hello agent");

        let parsed: ContextEvent = serde_json::from_value(json).unwrap();
        assert_eq!(parsed.event_type, "message_created");
        assert_eq!(parsed.conversation_id, conversation_id);
        assert_eq!(parsed.data["content"], "hello agent");
    }

    #[test]
    fn test_context_event_settings_changed_roundtrip() {
        let conversation_id = Uuid::new_v4();
        let changed_by = Uuid::new_v4().to_string();
        let event = ContextEvent {
            event_type: "conversation_settings_changed".to_owned(),
            conversation_id,
            data: json!({
                "changed_by": changed_by,
                "changes": { "title": "Design Sync" }
            }),
            timestamp: "2026-03-16T00:00:00Z".parse::<DateTime<Utc>>().unwrap(),
        };

        let json = serde_json::to_value(&event).unwrap();
        assert_eq!(json["event_type"], "conversation_settings_changed");
        assert_eq!(json["data"]["changes"]["title"], "Design Sync");

        let parsed: ContextEvent = serde_json::from_value(json).unwrap();
        assert_eq!(parsed.event_type, "conversation_settings_changed");
        assert_eq!(parsed.conversation_id, conversation_id);
        assert_eq!(parsed.data["changed_by"], changed_by);
        assert_eq!(parsed.data["changes"]["title"], "Design Sync");
    }

    #[test]
    fn test_agent_stream_structured_tool_call_roundtrip() {
        let stream_id = Uuid::new_v4();
        let frame = AgentStreamMsg::ToolCallStart(ToolCallStartMsg {
            v: PROTOCOL_VERSION,
            stream_id,
            id: "call_1".to_owned(),
            name: "web_search".to_owned(),
            arguments_json: serde_json::json!({"q": "paw messenger"}),
        });

        let json = serde_json::to_value(&frame).unwrap();
        assert_eq!(json["type"], "tool_call_start");
        assert_eq!(json["id"], "call_1");

        let parsed: AgentStreamMsg = serde_json::from_value(json).unwrap();
        match parsed {
            AgentStreamMsg::ToolCallStart(msg) => {
                assert_eq!(msg.v, PROTOCOL_VERSION);
                assert_eq!(msg.stream_id, stream_id);
                assert_eq!(msg.id, "call_1");
                assert_eq!(msg.name, "web_search");
                assert_eq!(msg.arguments_json["q"], "paw messenger");
            }
            _ => panic!("expected ToolCallStart variant"),
        }
    }

    #[test]
    fn test_server_message_agent_typing_event_tags() {
        let frame = ServerMessage::AgentTypingStart(AgentTypingEventMsg {
            v: PROTOCOL_VERSION,
            conversation_id: Uuid::new_v4(),
            thread_id: None,
            agent_id: Uuid::new_v4(),
        });

        let json = serde_json::to_value(&frame).unwrap();
        assert_eq!(json["type"], "agent_typing_start");
        assert!(json["conversation_id"].is_string());
        assert!(json["agent_id"].is_string());
    }
}
