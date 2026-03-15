use crate::db::DbPool;
use crate::ws::hub::Hub;
use anyhow::Context;
use async_trait::async_trait;
use paw_proto::{
    ContextConversationSettingsChangedMsg, ContextEvent, ContextMemberJoinedMsg,
    ContextMemberLeftMsg, ContextMessageCreatedMsg, ContextMessageDeletedMsg,
    ContextMessageEditedMsg, ContextThreadCreatedMsg, MessageReceivedMsg,
};
use std::sync::Arc;
use uuid::Uuid;

#[async_trait]
pub trait ContextEventSink: Send + Sync {
    async fn send_to_agent(&self, agent_id: Uuid, payload: &str);
}

#[async_trait]
pub trait BoundAgentLookup: Send + Sync {
    async fn bound_agents(&self, conversation_id: Uuid) -> anyhow::Result<Vec<Uuid>>;
}

#[allow(dead_code)]
#[async_trait]
pub trait ContextEngine: Send + Sync {
    async fn on_message_created(&self, event: ContextMessageCreatedMsg) -> anyhow::Result<()>;
    async fn on_message_edited(&self, event: ContextMessageEditedMsg) -> anyhow::Result<()>;
    async fn on_message_deleted(&self, event: ContextMessageDeletedMsg) -> anyhow::Result<()>;
    async fn on_member_joined(&self, event: ContextMemberJoinedMsg) -> anyhow::Result<()>;
    async fn on_member_left(&self, event: ContextMemberLeftMsg) -> anyhow::Result<()>;
    async fn on_thread_created(&self, event: ContextThreadCreatedMsg) -> anyhow::Result<()>;
    async fn on_conversation_settings_changed(
        &self,
        event: ContextConversationSettingsChangedMsg,
    ) -> anyhow::Result<()>;
}

pub struct DefaultContextEngine {
    lookup: Arc<dyn BoundAgentLookup>,
    sink: Arc<dyn ContextEventSink>,
}

impl DefaultContextEngine {
    pub fn new(db: DbPool, sink: Arc<dyn ContextEventSink>) -> Self {
        Self {
            lookup: Arc::new(SqlBoundAgentLookup::new(db)),
            sink,
        }
    }

    #[cfg(test)]
    fn with_lookup(lookup: Arc<dyn BoundAgentLookup>, sink: Arc<dyn ContextEventSink>) -> Self {
        Self { lookup, sink }
    }

    async fn dispatch(&self, conversation_id: Uuid, event: ContextEvent) -> anyhow::Result<()> {
        let agent_ids = self.lookup.bound_agents(conversation_id).await?;
        if agent_ids.is_empty() {
            return Ok(());
        }

        let payload = serde_json::to_string(&event).context("serialize context event")?;
        for agent_id in agent_ids {
            self.sink.send_to_agent(agent_id, &payload).await;
        }

        Ok(())
    }
}

#[async_trait]
impl ContextEngine for DefaultContextEngine {
    async fn on_message_created(&self, event: ContextMessageCreatedMsg) -> anyhow::Result<()> {
        self.dispatch(event.conversation_id, ContextEvent::MessageCreated(event))
            .await
    }

    async fn on_message_edited(&self, event: ContextMessageEditedMsg) -> anyhow::Result<()> {
        self.dispatch(event.conversation_id, ContextEvent::MessageEdited(event))
            .await
    }

    async fn on_message_deleted(&self, event: ContextMessageDeletedMsg) -> anyhow::Result<()> {
        self.dispatch(event.conversation_id, ContextEvent::MessageDeleted(event))
            .await
    }

    async fn on_member_joined(&self, event: ContextMemberJoinedMsg) -> anyhow::Result<()> {
        self.dispatch(event.conversation_id, ContextEvent::MemberJoined(event))
            .await
    }

    async fn on_member_left(&self, event: ContextMemberLeftMsg) -> anyhow::Result<()> {
        self.dispatch(event.conversation_id, ContextEvent::MemberLeft(event))
            .await
    }

    async fn on_thread_created(&self, event: ContextThreadCreatedMsg) -> anyhow::Result<()> {
        self.dispatch(event.conversation_id, ContextEvent::ThreadCreated(event))
            .await
    }

    async fn on_conversation_settings_changed(
        &self,
        event: ContextConversationSettingsChangedMsg,
    ) -> anyhow::Result<()> {
        self.dispatch(
            event.conversation_id,
            ContextEvent::ConversationSettingsChanged(event),
        )
        .await
    }
}

struct SqlBoundAgentLookup {
    db: DbPool,
}

impl SqlBoundAgentLookup {
    fn new(db: DbPool) -> Self {
        Self { db }
    }
}

#[async_trait]
impl BoundAgentLookup for SqlBoundAgentLookup {
    async fn bound_agents(&self, conversation_id: Uuid) -> anyhow::Result<Vec<Uuid>> {
        sqlx::query_scalar::<_, Uuid>(
            "SELECT agent_id FROM conversation_agents WHERE conversation_id = $1",
        )
        .bind(conversation_id)
        .fetch_all(self.db.as_ref())
        .await
        .context("load bound agents")
    }
}

#[async_trait]
impl ContextEventSink for Hub {
    async fn send_to_agent(&self, agent_id: Uuid, payload: &str) {
        self.send_to_user_nonblocking(agent_id, payload).await;
    }
}

pub fn message_created_event(message: MessageReceivedMsg) -> ContextMessageCreatedMsg {
    ContextMessageCreatedMsg {
        v: paw_proto::PROTOCOL_VERSION,
        conversation_id: message.conversation_id,
        occurred_at: message.created_at,
        message,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Utc;
    use paw_proto::{ContextConversationSettingsChangedMsg, MessageAttachment, MessageFormat};
    use serde_json::json;
    use std::sync::Mutex;

    struct MockBoundAgentLookup {
        agent_ids: Vec<Uuid>,
    }

    #[async_trait]
    impl BoundAgentLookup for MockBoundAgentLookup {
        async fn bound_agents(&self, _conversation_id: Uuid) -> anyhow::Result<Vec<Uuid>> {
            Ok(self.agent_ids.clone())
        }
    }

    #[derive(Default)]
    struct MockSink {
        sent: Mutex<Vec<(Uuid, String)>>,
    }

    #[async_trait]
    impl ContextEventSink for MockSink {
        async fn send_to_agent(&self, agent_id: Uuid, payload: &str) {
            self.sent
                .lock()
                .expect("sink mutex poisoned")
                .push((agent_id, payload.to_owned()));
        }
    }

    fn build_engine(agent_ids: Vec<Uuid>, sink: Arc<MockSink>) -> DefaultContextEngine {
        DefaultContextEngine::with_lookup(Arc::new(MockBoundAgentLookup { agent_ids }), sink)
    }

    fn sample_message(conversation_id: Uuid) -> MessageReceivedMsg {
        MessageReceivedMsg {
            v: paw_proto::PROTOCOL_VERSION,
            id: Uuid::new_v4(),
            conversation_id,
            thread_id: Some(Uuid::new_v4()),
            sender_id: Uuid::new_v4(),
            content: "hook fired".into(),
            format: MessageFormat::Markdown,
            seq: 3,
            created_at: Utc::now(),
            blocks: Vec::new(),
            attachments: vec![MessageAttachment {
                id: Uuid::new_v4(),
                file_type: "image".into(),
                file_url: "https://example.com/file.png".into(),
                file_size: 42,
                mime_type: "image/png".into(),
                thumbnail_url: Some("https://example.com/thumb.png".into()),
            }],
        }
    }

    fn sent_payloads(sink: &MockSink) -> Vec<(Uuid, String)> {
        sink.sent.lock().expect("sink mutex poisoned").clone()
    }

    #[tokio::test]
    async fn message_created_dispatches_payload_to_all_bound_agents() {
        let conversation_id = Uuid::new_v4();
        let agent_a = Uuid::new_v4();
        let agent_b = Uuid::new_v4();
        let sink = Arc::new(MockSink::default());
        let engine = build_engine(vec![agent_a, agent_b], sink.clone());
        let message = sample_message(conversation_id);

        engine
            .on_message_created(message_created_event(message.clone()))
            .await
            .expect("message_created hook should succeed");

        let sent = sent_payloads(&sink);
        assert_eq!(sent.len(), 2);
        assert_eq!(sent[0].0, agent_a);
        assert_eq!(sent[1].0, agent_b);

        let payload: ContextEvent = serde_json::from_str(&sent[0].1).unwrap();
        match payload {
            ContextEvent::MessageCreated(event) => {
                assert_eq!(event.conversation_id, conversation_id);
                assert_eq!(event.message.id, message.id);
                assert_eq!(event.message.attachments.len(), 1);
            }
            _ => panic!("expected message_created event"),
        }
    }

    #[tokio::test]
    async fn message_edited_dispatches_payload() {
        let conversation_id = Uuid::new_v4();
        let sink = Arc::new(MockSink::default());
        let engine = build_engine(vec![Uuid::new_v4()], sink.clone());

        engine
            .on_message_edited(ContextMessageEditedMsg {
                v: paw_proto::PROTOCOL_VERSION,
                conversation_id,
                thread_id: Some(Uuid::new_v4()),
                message_id: Uuid::new_v4(),
                edited_by: Uuid::new_v4(),
                content: "updated".into(),
                format: MessageFormat::Plain,
                occurred_at: Utc::now(),
            })
            .await
            .expect("message_edited hook should succeed");

        let sent = sent_payloads(&sink);
        let payload: ContextEvent = serde_json::from_str(&sent[0].1).unwrap();
        assert!(matches!(payload, ContextEvent::MessageEdited(_)));
    }

    #[tokio::test]
    async fn message_deleted_dispatches_payload() {
        let conversation_id = Uuid::new_v4();
        let sink = Arc::new(MockSink::default());
        let engine = build_engine(vec![Uuid::new_v4()], sink.clone());

        engine
            .on_message_deleted(ContextMessageDeletedMsg {
                v: paw_proto::PROTOCOL_VERSION,
                conversation_id,
                thread_id: None,
                message_id: Uuid::new_v4(),
                deleted_by: Uuid::new_v4(),
                occurred_at: Utc::now(),
            })
            .await
            .expect("message_deleted hook should succeed");

        let sent = sent_payloads(&sink);
        let payload: ContextEvent = serde_json::from_str(&sent[0].1).unwrap();
        assert!(matches!(payload, ContextEvent::MessageDeleted(_)));
    }

    #[tokio::test]
    async fn member_joined_dispatches_payload() {
        let conversation_id = Uuid::new_v4();
        let sink = Arc::new(MockSink::default());
        let engine = build_engine(vec![Uuid::new_v4()], sink.clone());

        engine
            .on_member_joined(ContextMemberJoinedMsg {
                v: paw_proto::PROTOCOL_VERSION,
                conversation_id,
                member_id: Uuid::new_v4(),
                joined_by: Uuid::new_v4(),
                occurred_at: Utc::now(),
            })
            .await
            .expect("member_joined hook should succeed");

        let sent = sent_payloads(&sink);
        let payload: ContextEvent = serde_json::from_str(&sent[0].1).unwrap();
        assert!(matches!(payload, ContextEvent::MemberJoined(_)));
    }

    #[tokio::test]
    async fn member_left_dispatches_payload() {
        let conversation_id = Uuid::new_v4();
        let sink = Arc::new(MockSink::default());
        let engine = build_engine(vec![Uuid::new_v4()], sink.clone());

        engine
            .on_member_left(ContextMemberLeftMsg {
                v: paw_proto::PROTOCOL_VERSION,
                conversation_id,
                member_id: Uuid::new_v4(),
                left_by: Uuid::new_v4(),
                occurred_at: Utc::now(),
            })
            .await
            .expect("member_left hook should succeed");

        let sent = sent_payloads(&sink);
        let payload: ContextEvent = serde_json::from_str(&sent[0].1).unwrap();
        assert!(matches!(payload, ContextEvent::MemberLeft(_)));
    }

    #[tokio::test]
    async fn thread_created_dispatches_payload() {
        let conversation_id = Uuid::new_v4();
        let sink = Arc::new(MockSink::default());
        let engine = build_engine(vec![Uuid::new_v4()], sink.clone());

        engine
            .on_thread_created(ContextThreadCreatedMsg {
                v: paw_proto::PROTOCOL_VERSION,
                conversation_id,
                thread_id: Uuid::new_v4(),
                root_message_id: Uuid::new_v4(),
                title: Some("Thread".into()),
                created_by: Uuid::new_v4(),
                occurred_at: Utc::now(),
            })
            .await
            .expect("thread_created hook should succeed");

        let sent = sent_payloads(&sink);
        let payload: ContextEvent = serde_json::from_str(&sent[0].1).unwrap();
        assert!(matches!(payload, ContextEvent::ThreadCreated(_)));
    }

    #[tokio::test]
    async fn settings_changed_skips_delivery_when_no_agents_bound() {
        let sink = Arc::new(MockSink::default());
        let engine = build_engine(Vec::new(), sink.clone());

        engine
            .on_conversation_settings_changed(ContextConversationSettingsChangedMsg {
                v: paw_proto::PROTOCOL_VERSION,
                conversation_id: Uuid::new_v4(),
                changed_by: Uuid::new_v4(),
                occurred_at: Utc::now(),
                changes: json!({ "title": "New name" }),
            })
            .await
            .expect("settings hook should succeed");

        let sent = sent_payloads(&sink);
        assert!(sent.is_empty());
    }
}
