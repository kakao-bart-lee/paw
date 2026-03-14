use std::collections::BTreeMap;

use paw_proto::{ClientMessage, MessageReceivedMsg, SyncMsg, PROTOCOL_VERSION};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConversationSyncCursor {
    pub conversation_id: Uuid,
    pub last_seq: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MessageSyncOutcome {
    DuplicateOrStale { ack_seq: i64 },
    GapDetected { request_from_seq: i64 },
    Applied { ack_seq: i64 },
}

#[derive(Clone, Debug, Default)]
pub struct SyncEngine {
    cursors: BTreeMap<Uuid, i64>,
    pending_recoveries: BTreeMap<Uuid, i64>,
}

impl SyncEngine {
    pub fn new(cursors: impl IntoIterator<Item = ConversationSyncCursor>) -> Self {
        let mut engine = Self::default();
        engine.replace_cursors(cursors);
        engine
    }

    pub fn replace_cursors(&mut self, cursors: impl IntoIterator<Item = ConversationSyncCursor>) {
        self.cursors.clear();
        for cursor in cursors {
            self.cursors
                .insert(cursor.conversation_id, cursor.last_seq.max(0));
        }
        self.pending_recoveries.clear();
    }

    pub fn last_seq(&self, conversation_id: Uuid) -> i64 {
        self.cursors
            .get(&conversation_id)
            .copied()
            .unwrap_or_default()
    }

    pub fn request_sync_message(&self, conversation_id: Uuid) -> ClientMessage {
        ClientMessage::Sync(SyncMsg {
            v: PROTOCOL_VERSION,
            conversation_id,
            last_seq: self.last_seq(conversation_id),
        })
    }

    pub fn sync_all_conversations(&self) -> Vec<ClientMessage> {
        self.cursors
            .keys()
            .copied()
            .map(|conversation_id| self.request_sync_message(conversation_id))
            .collect()
    }

    pub fn ingest_message(&mut self, msg: &MessageReceivedMsg) -> MessageSyncOutcome {
        let current = self.last_seq(msg.conversation_id);

        if msg.seq <= current {
            return MessageSyncOutcome::DuplicateOrStale { ack_seq: current };
        }

        if msg.seq > current + 1 {
            return MessageSyncOutcome::GapDetected {
                request_from_seq: current,
            };
        }

        self.cursors.insert(msg.conversation_id, msg.seq);
        self.pending_recoveries.remove(&msg.conversation_id);
        MessageSyncOutcome::Applied { ack_seq: msg.seq }
    }

    pub fn mark_recovery_pending(&mut self, conversation_id: Uuid, request_from_seq: i64) {
        self.pending_recoveries
            .insert(conversation_id, request_from_seq.max(0));
    }

    pub fn apply_gap_fill(&mut self, messages: &[MessageReceivedMsg]) {
        for message in messages {
            let next = self.last_seq(message.conversation_id).max(message.seq);
            self.cursors.insert(message.conversation_id, next);
        }
    }

    pub fn clear_recoveries(
        &mut self,
        conversations: impl IntoIterator<Item = ConversationSyncCursor>,
    ) {
        for conversation in conversations {
            self.pending_recoveries
                .remove(&conversation.conversation_id);
        }
    }

    pub fn cursors(&self) -> Vec<ConversationSyncCursor> {
        self.cursors
            .iter()
            .map(|(conversation_id, last_seq)| ConversationSyncCursor {
                conversation_id: *conversation_id,
                last_seq: *last_seq,
            })
            .collect()
    }

    /// Pending recovery markers are cleared on:
    /// - `replace_cursors()` after a fresh HelloOk/bootstrap cursor reload
    /// - `clear_recoveries()` when a DeviceSyncResponse arrives for a conversation
    pub fn pending_recoveries(&self) -> Vec<ConversationSyncCursor> {
        self.pending_recoveries
            .iter()
            .map(
                |(conversation_id, request_from_seq)| ConversationSyncCursor {
                    conversation_id: *conversation_id,
                    last_seq: *request_from_seq,
                },
            )
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use paw_proto::{MessageFormat, MessageReceivedMsg};

    use super::*;

    fn message(conversation_id: Uuid, seq: i64) -> MessageReceivedMsg {
        MessageReceivedMsg {
            v: PROTOCOL_VERSION,
            id: Uuid::new_v4(),
            conversation_id,
            sender_id: Uuid::new_v4(),
            content: format!("msg-{seq}"),
            format: MessageFormat::Markdown,
            seq,
            created_at: Utc::now(),
            blocks: vec![],
        }
    }

    #[test]
    fn sync_all_uses_current_cursors() {
        let conversation_id = Uuid::new_v4();
        let engine = SyncEngine::new([ConversationSyncCursor {
            conversation_id,
            last_seq: 7,
        }]);

        let messages = engine.sync_all_conversations();
        assert_eq!(messages.len(), 1);

        match &messages[0] {
            ClientMessage::Sync(sync) => {
                assert_eq!(sync.conversation_id, conversation_id);
                assert_eq!(sync.last_seq, 7);
            }
            other => panic!("expected sync message, got {other:?}"),
        }
    }

    #[test]
    fn ingest_message_matches_flutter_gap_detection() {
        let conversation_id = Uuid::new_v4();
        let mut engine = SyncEngine::new([ConversationSyncCursor {
            conversation_id,
            last_seq: 2,
        }]);

        assert_eq!(
            engine.ingest_message(&message(conversation_id, 2)),
            MessageSyncOutcome::DuplicateOrStale { ack_seq: 2 }
        );
        assert_eq!(
            engine.ingest_message(&message(conversation_id, 4)),
            MessageSyncOutcome::GapDetected {
                request_from_seq: 2
            }
        );
        assert_eq!(
            engine.ingest_message(&message(conversation_id, 3)),
            MessageSyncOutcome::Applied { ack_seq: 3 }
        );
    }

    #[test]
    fn gap_fill_updates_cursor_to_highest_seq() {
        let conversation_id = Uuid::new_v4();
        let mut engine = SyncEngine::default();

        engine.apply_gap_fill(&[
            message(conversation_id, 1),
            message(conversation_id, 2),
            message(conversation_id, 5),
        ]);

        assert_eq!(engine.last_seq(conversation_id), 5);
    }

    #[test]
    fn recovery_tracking_marks_gap_and_clears_after_response() {
        let conversation_id = Uuid::new_v4();
        let mut engine = SyncEngine::new([ConversationSyncCursor {
            conversation_id,
            last_seq: 2,
        }]);

        assert_eq!(
            engine.ingest_message(&message(conversation_id, 4)),
            MessageSyncOutcome::GapDetected {
                request_from_seq: 2
            }
        );
        engine.mark_recovery_pending(conversation_id, 2);
        assert_eq!(engine.pending_recoveries().len(), 1);

        engine.clear_recoveries([ConversationSyncCursor {
            conversation_id,
            last_seq: 2,
        }]);
        assert!(engine.pending_recoveries().is_empty());
    }
}
