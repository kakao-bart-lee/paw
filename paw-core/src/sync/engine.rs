use std::collections::BTreeMap;

use paw_proto::{ClientMessage, MessageReceivedMsg, SyncMsg, PROTOCOL_VERSION};
use uuid::Uuid;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ConversationSyncCursor {
    pub conversation_id: Uuid,
    pub last_seq: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ScopedSyncCursor {
    pub conversation_id: Uuid,
    pub thread_id: Option<Uuid>,
    pub last_seq: i64,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MessageSyncOutcome {
    DuplicateOrStale { ack_seq: i64 },
    GapDetected { request_from_seq: i64 },
    Applied { ack_seq: i64 },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
struct ScopeKey {
    conversation_id: Uuid,
    thread_id: Option<Uuid>,
}

impl ScopeKey {
    fn main(conversation_id: Uuid) -> Self {
        Self {
            conversation_id,
            thread_id: None,
        }
    }

    fn thread(conversation_id: Uuid, thread_id: Uuid) -> Self {
        Self {
            conversation_id,
            thread_id: Some(thread_id),
        }
    }

    fn from_message(msg: &MessageReceivedMsg) -> Self {
        Self {
            conversation_id: msg.conversation_id,
            thread_id: msg.thread_id,
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct SyncEngine {
    cursors: BTreeMap<ScopeKey, i64>,
    conversation_heads: BTreeMap<Uuid, i64>,
    pending_recoveries: BTreeMap<ScopeKey, i64>,
}

impl SyncEngine {
    pub fn new(cursors: impl IntoIterator<Item = ConversationSyncCursor>) -> Self {
        let mut engine = Self::default();
        engine.replace_cursors(cursors);
        engine
    }

    pub fn replace_cursors(&mut self, cursors: impl IntoIterator<Item = ConversationSyncCursor>) {
        self.cursors.clear();
        self.conversation_heads.clear();
        for cursor in cursors {
            let last_seq = cursor.last_seq.max(0);
            self.cursors
                .insert(ScopeKey::main(cursor.conversation_id), last_seq);
            self.conversation_heads
                .insert(cursor.conversation_id, last_seq);
        }
        self.pending_recoveries.clear();
    }

    pub fn upsert_thread_cursor(&mut self, conversation_id: Uuid, thread_id: Uuid, last_seq: i64) {
        let last_seq = last_seq.max(0);
        self.cursors
            .insert(ScopeKey::thread(conversation_id, thread_id), last_seq);
        self.conversation_heads
            .entry(conversation_id)
            .and_modify(|seq| *seq = (*seq).max(last_seq))
            .or_insert(last_seq);
    }

    pub fn last_seq(&self, conversation_id: Uuid) -> i64 {
        self.cursors
            .get(&ScopeKey::main(conversation_id))
            .copied()
            .unwrap_or_default()
    }

    pub fn last_conversation_seq(&self, conversation_id: Uuid) -> i64 {
        self.conversation_heads
            .get(&conversation_id)
            .copied()
            .unwrap_or_default()
    }

    pub fn last_thread_seq(&self, conversation_id: Uuid, thread_id: Uuid) -> i64 {
        self.cursors
            .get(&ScopeKey::thread(conversation_id, thread_id))
            .copied()
            .unwrap_or_default()
    }

    pub fn request_sync_message(&self, conversation_id: Uuid) -> ClientMessage {
        ClientMessage::Sync(SyncMsg {
            v: PROTOCOL_VERSION,
            conversation_id,
            thread_id: None,
            last_seq: self.last_seq(conversation_id),
        })
    }

    pub fn request_thread_sync_message(
        &self,
        conversation_id: Uuid,
        thread_id: Uuid,
    ) -> ClientMessage {
        ClientMessage::Sync(SyncMsg {
            v: PROTOCOL_VERSION,
            conversation_id,
            thread_id: Some(thread_id),
            last_seq: self.last_thread_seq(conversation_id, thread_id),
        })
    }

    pub fn sync_all_conversations(&self) -> Vec<ClientMessage> {
        self.cursors
            .keys()
            .filter(|scope| scope.thread_id.is_none())
            .map(|scope| self.request_sync_message(scope.conversation_id))
            .collect()
    }

    pub fn sync_all_scopes(&self) -> Vec<ClientMessage> {
        self.cursors
            .keys()
            .copied()
            .map(|scope| match scope.thread_id {
                Some(thread_id) => {
                    self.request_thread_sync_message(scope.conversation_id, thread_id)
                }
                None => self.request_sync_message(scope.conversation_id),
            })
            .collect()
    }

    pub fn ingest_message(&mut self, msg: &MessageReceivedMsg) -> MessageSyncOutcome {
        let scope = ScopeKey::from_message(msg);
        let scope_current = self.cursors.get(&scope).copied().unwrap_or_default();
        let current = self.last_conversation_seq(msg.conversation_id);

        if msg.seq <= scope_current {
            return MessageSyncOutcome::DuplicateOrStale {
                ack_seq: scope_current,
            };
        }

        if msg.seq > current + 1 {
            return MessageSyncOutcome::GapDetected {
                request_from_seq: current,
            };
        }

        self.cursors
            .entry(scope)
            .and_modify(|seq| *seq = (*seq).max(msg.seq))
            .or_insert(msg.seq);
        self.conversation_heads
            .entry(msg.conversation_id)
            .and_modify(|seq| *seq = (*seq).max(msg.seq))
            .or_insert(msg.seq);
        self.pending_recoveries.remove(&scope);
        MessageSyncOutcome::Applied { ack_seq: msg.seq }
    }

    pub fn mark_recovery_pending(&mut self, conversation_id: Uuid, request_from_seq: i64) {
        self.pending_recoveries
            .insert(ScopeKey::main(conversation_id), request_from_seq.max(0));
    }

    pub fn mark_thread_recovery_pending(
        &mut self,
        conversation_id: Uuid,
        thread_id: Uuid,
        request_from_seq: i64,
    ) {
        self.pending_recoveries.insert(
            ScopeKey::thread(conversation_id, thread_id),
            request_from_seq.max(0),
        );
    }

    pub fn apply_gap_fill(&mut self, messages: &[MessageReceivedMsg]) {
        for message in messages {
            let scope = ScopeKey::from_message(message);
            let next = self
                .cursors
                .get(&scope)
                .copied()
                .unwrap_or_default()
                .max(message.seq);
            self.cursors.insert(scope, next);
            self.conversation_heads
                .entry(message.conversation_id)
                .and_modify(|seq| *seq = (*seq).max(message.seq))
                .or_insert(message.seq);
        }
    }

    pub fn clear_recoveries(
        &mut self,
        conversations: impl IntoIterator<Item = ConversationSyncCursor>,
    ) {
        for conversation in conversations {
            self.pending_recoveries
                .remove(&ScopeKey::main(conversation.conversation_id));
        }
    }

    pub fn clear_scope_recoveries(&mut self, scopes: impl IntoIterator<Item = ScopedSyncCursor>) {
        for scope in scopes {
            self.pending_recoveries.remove(&ScopeKey {
                conversation_id: scope.conversation_id,
                thread_id: scope.thread_id,
            });
        }
    }

    pub fn cursors(&self) -> Vec<ConversationSyncCursor> {
        self.cursors
            .iter()
            .filter(|(scope, _)| scope.thread_id.is_none())
            .map(|(scope, last_seq)| ConversationSyncCursor {
                conversation_id: scope.conversation_id,
                last_seq: *last_seq,
            })
            .collect()
    }

    pub fn scope_cursors(&self) -> Vec<ScopedSyncCursor> {
        self.cursors
            .iter()
            .map(|(scope, last_seq)| ScopedSyncCursor {
                conversation_id: scope.conversation_id,
                thread_id: scope.thread_id,
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
            .filter(|(scope, _)| scope.thread_id.is_none())
            .map(|(scope, request_from_seq)| ConversationSyncCursor {
                conversation_id: scope.conversation_id,
                last_seq: *request_from_seq,
            })
            .collect()
    }

    pub fn pending_scope_recoveries(&self) -> Vec<ScopedSyncCursor> {
        self.pending_recoveries
            .iter()
            .map(|(scope, request_from_seq)| ScopedSyncCursor {
                conversation_id: scope.conversation_id,
                thread_id: scope.thread_id,
                last_seq: *request_from_seq,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use paw_proto::{MessageFormat, MessageReceivedMsg};

    use super::*;

    fn message(conversation_id: Uuid, thread_id: Option<Uuid>, seq: i64) -> MessageReceivedMsg {
        MessageReceivedMsg {
            v: PROTOCOL_VERSION,
            id: Uuid::new_v4(),
            conversation_id,
            thread_id,
            sender_id: Uuid::new_v4(),
            content: format!("msg-{seq}"),
            format: MessageFormat::Markdown,
            seq,
            created_at: Utc::now(),
            blocks: vec![],
            attachments: vec![],
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
                assert_eq!(sync.thread_id, None);
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
            engine.ingest_message(&message(conversation_id, None, 2)),
            MessageSyncOutcome::DuplicateOrStale { ack_seq: 2 }
        );
        assert_eq!(
            engine.ingest_message(&message(conversation_id, None, 4)),
            MessageSyncOutcome::GapDetected {
                request_from_seq: 2
            }
        );
        assert_eq!(
            engine.ingest_message(&message(conversation_id, None, 3)),
            MessageSyncOutcome::Applied { ack_seq: 3 }
        );
    }

    #[test]
    fn gap_fill_updates_cursor_to_highest_seq() {
        let conversation_id = Uuid::new_v4();
        let mut engine = SyncEngine::default();

        engine.apply_gap_fill(&[
            message(conversation_id, None, 1),
            message(conversation_id, None, 2),
            message(conversation_id, None, 5),
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
            engine.ingest_message(&message(conversation_id, None, 4)),
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

    #[test]
    fn thread_scopes_track_highest_seen_while_gap_detection_uses_conversation_head() {
        let conversation_id = Uuid::new_v4();
        let thread_id = Uuid::new_v4();
        let mut engine = SyncEngine::default();

        assert_eq!(
            engine.ingest_message(&message(conversation_id, Some(thread_id), 1)),
            MessageSyncOutcome::Applied { ack_seq: 1 }
        );
        assert_eq!(engine.last_conversation_seq(conversation_id), 1);
        assert_eq!(engine.last_thread_seq(conversation_id, thread_id), 1);

        assert_eq!(
            engine.ingest_message(&message(conversation_id, None, 2)),
            MessageSyncOutcome::Applied { ack_seq: 2 }
        );

        assert_eq!(
            engine.ingest_message(&message(conversation_id, Some(thread_id), 3)),
            MessageSyncOutcome::Applied { ack_seq: 3 }
        );

        assert_eq!(engine.last_conversation_seq(conversation_id), 3);
        assert_eq!(engine.last_thread_seq(conversation_id, thread_id), 3);
    }

    #[test]
    fn sync_all_scopes_includes_threads_with_scope_last_seq() {
        let conversation_id = Uuid::new_v4();
        let thread_id = Uuid::new_v4();
        let mut engine = SyncEngine::new([ConversationSyncCursor {
            conversation_id,
            last_seq: 4,
        }]);
        engine.upsert_thread_cursor(conversation_id, thread_id, 9);

        let messages = engine.sync_all_scopes();
        assert_eq!(messages.len(), 2);
        assert!(messages.iter().any(|message| matches!(
            message,
            ClientMessage::Sync(SyncMsg {
                conversation_id: sync_conversation_id,
                thread_id: None,
                last_seq: 4,
                ..
            }) if *sync_conversation_id == conversation_id
        )));
        assert!(messages.iter().any(|message| matches!(
            message,
            ClientMessage::Sync(SyncMsg {
                conversation_id: sync_conversation_id,
                thread_id: Some(sync_thread_id),
                last_seq: 9,
                ..
            }) if *sync_conversation_id == conversation_id && *sync_thread_id == thread_id
        )));
    }

    #[test]
    fn thread_recoveries_are_tracked_separately_from_main_timeline() {
        let conversation_id = Uuid::new_v4();
        let thread_id = Uuid::new_v4();
        let mut engine = SyncEngine::default();

        engine.mark_recovery_pending(conversation_id, 4);
        engine.mark_thread_recovery_pending(conversation_id, thread_id, 7);

        assert_eq!(
            engine.pending_recoveries(),
            vec![ConversationSyncCursor {
                conversation_id,
                last_seq: 4,
            }]
        );
        assert!(engine.pending_scope_recoveries().iter().any(|scope| {
            scope.conversation_id == conversation_id
                && scope.thread_id == Some(thread_id)
                && scope.last_seq == 7
        }));
    }

    #[test]
    fn interleaved_thread_message_detects_gap_from_conversation_head_not_thread_cursor() {
        let conversation_id = Uuid::new_v4();
        let thread_id = Uuid::new_v4();
        let mut engine = SyncEngine::default();

        assert_eq!(
            engine.ingest_message(&message(conversation_id, Some(thread_id), 1)),
            MessageSyncOutcome::Applied { ack_seq: 1 }
        );
        assert_eq!(
            engine.ingest_message(&message(conversation_id, Some(thread_id), 3)),
            MessageSyncOutcome::GapDetected {
                request_from_seq: 1
            }
        );

        assert_eq!(
            engine.ingest_message(&message(conversation_id, None, 2)),
            MessageSyncOutcome::Applied { ack_seq: 2 }
        );
        assert_eq!(
            engine.ingest_message(&message(conversation_id, Some(thread_id), 3)),
            MessageSyncOutcome::Applied { ack_seq: 3 }
        );
    }
}
