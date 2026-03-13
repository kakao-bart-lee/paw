use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use paw_proto::{
    ContentDeltaMsg, MessageFormat, StreamEndMsg, StreamStartMsg, ToolEndMsg, ToolStartMsg,
};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ToolCallRecord {
    pub tool: String,
    pub label: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct StreamingSession {
    pub stream_id: Uuid,
    pub conversation_id: Uuid,
    pub agent_id: Uuid,
    pub content: String,
    pub current_tool: Option<String>,
    pub current_tool_label: Option<String>,
    pub tool_complete: bool,
    pub is_complete: bool,
    pub tool_history: Vec<ToolCallRecord>,
}

#[derive(Clone, Debug)]
pub struct FinalizedStreamMessage {
    pub id: Uuid,
    pub conversation_id: Uuid,
    pub sender_id: Uuid,
    pub content: String,
    pub format: MessageFormat,
    pub seq: i64,
    pub created_at: DateTime<Utc>,
    pub tool_calls: Vec<ToolCallRecord>,
    pub tokens: u32,
    pub duration_ms: u64,
}

impl PartialEq for FinalizedStreamMessage {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
            && self.conversation_id == other.conversation_id
            && self.sender_id == other.sender_id
            && self.content == other.content
            && std::mem::discriminant(&self.format) == std::mem::discriminant(&other.format)
            && self.seq == other.seq
            && self.created_at == other.created_at
            && self.tool_calls == other.tool_calls
            && self.tokens == other.tokens
            && self.duration_ms == other.duration_ms
    }
}

impl Eq for FinalizedStreamMessage {}

#[derive(Clone, Debug, Default)]
pub struct StreamingState {
    active: BTreeMap<Uuid, StreamingSession>,
}

impl StreamingState {
    pub fn handle_stream_start(&mut self, msg: &StreamStartMsg) {
        self.active.insert(
            msg.stream_id,
            StreamingSession {
                stream_id: msg.stream_id,
                conversation_id: msg.conversation_id,
                agent_id: msg.agent_id,
                content: String::new(),
                current_tool: None,
                current_tool_label: None,
                tool_complete: false,
                is_complete: false,
                tool_history: Vec::new(),
            },
        );
    }

    pub fn handle_content_delta(&mut self, msg: &ContentDeltaMsg) -> bool {
        if let Some(stream) = self.active.get_mut(&msg.stream_id) {
            stream.content.push_str(&msg.delta);
            return true;
        }
        false
    }

    pub fn handle_tool_start(&mut self, msg: &ToolStartMsg) -> bool {
        if let Some(stream) = self.active.get_mut(&msg.stream_id) {
            stream.current_tool = Some(msg.tool.clone());
            stream.current_tool_label = Some(msg.label.clone());
            stream.tool_complete = false;
            stream.tool_history.push(ToolCallRecord {
                tool: msg.tool.clone(),
                label: msg.label.clone(),
            });
            return true;
        }
        false
    }

    pub fn handle_tool_end(&mut self, msg: &ToolEndMsg) -> bool {
        if let Some(stream) = self.active.get_mut(&msg.stream_id) {
            if stream.current_tool.as_deref() == Some(msg.tool.as_str()) {
                stream.tool_complete = true;
            }
            return true;
        }
        false
    }

    pub fn handle_stream_end(
        &mut self,
        msg: &StreamEndMsg,
        next_seq: i64,
        created_at: DateTime<Utc>,
    ) -> Option<FinalizedStreamMessage> {
        let mut stream = self.active.remove(&msg.stream_id)?;
        stream.is_complete = true;

        Some(FinalizedStreamMessage {
            id: msg.stream_id,
            conversation_id: stream.conversation_id,
            sender_id: stream.agent_id,
            content: stream.content,
            format: MessageFormat::Markdown,
            seq: next_seq,
            created_at,
            tool_calls: stream.tool_history,
            tokens: msg.tokens,
            duration_ms: msg.duration_ms,
        })
    }

    pub fn active_session(&self, stream_id: Uuid) -> Option<&StreamingSession> {
        self.active.get(&stream_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use paw_proto::PROTOCOL_VERSION;

    #[test]
    fn streaming_flow_accumulates_content_and_tool_history() {
        let conversation_id = Uuid::new_v4();
        let agent_id = Uuid::new_v4();
        let stream_id = Uuid::new_v4();
        let mut state = StreamingState::default();

        state.handle_stream_start(&StreamStartMsg {
            v: PROTOCOL_VERSION,
            conversation_id,
            agent_id,
            stream_id,
        });
        assert!(state.handle_content_delta(&ContentDeltaMsg {
            v: PROTOCOL_VERSION,
            stream_id,
            delta: "Hello".into(),
        }));
        assert!(state.handle_tool_start(&ToolStartMsg {
            v: PROTOCOL_VERSION,
            stream_id,
            tool: "search".into(),
            label: "Searching".into(),
        }));
        assert!(state.handle_tool_end(&ToolEndMsg {
            v: PROTOCOL_VERSION,
            stream_id,
            tool: "search".into(),
        }));
        assert!(state.handle_content_delta(&ContentDeltaMsg {
            v: PROTOCOL_VERSION,
            stream_id,
            delta: " world".into(),
        }));

        let message = state
            .handle_stream_end(
                &StreamEndMsg {
                    v: PROTOCOL_VERSION,
                    stream_id,
                    tokens: 12,
                    duration_ms: 400,
                },
                8,
                Utc::now(),
            )
            .expect("stream should finalize");

        assert_eq!(message.conversation_id, conversation_id);
        assert_eq!(message.sender_id, agent_id);
        assert_eq!(message.content, "Hello world");
        assert_eq!(message.seq, 8);
        assert_eq!(message.tool_calls.len(), 1);
        assert_eq!(message.tool_calls[0].tool, "search");
        assert!(state.active_session(stream_id).is_none());
    }
}
