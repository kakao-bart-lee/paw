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
        self.send_to_agent_nonblocking(agent_id, payload).await;
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
    use paw_proto::{ContextConversationSettingsChangedMsg, MessageFormat};
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

    #[tokio::test]
    async fn message_created_dispatches_payload_to_all_bound_agents() {
        let conversation_id = Uuid::new_v4();
        let agent_a = Uuid::new_v4();
        let agent_b = Uuid::new_v4();
        let sink = Arc::new(MockSink::default());
        let engine = build_engine(vec![agent_a, agent_b], sink.clone());
        let message = MessageReceivedMsg {
            v: paw_proto::PROTOCOL_VERSION,
            id: Uuid::new_v4(),
            conversation_id,
            thread_id: None,
            sender_id: Uuid::new_v4(),
            content: "hook fired".into(),
            format: MessageFormat::Markdown,
            seq: 3,
            created_at: Utc::now(),
            blocks: Vec::new(),
        };

        engine
            .on_message_created(message_created_event(message.clone()))
            .await
            .expect("message_created hook should succeed");

        let sent = sink.sent.lock().expect("sink mutex poisoned");
        assert_eq!(sent.len(), 2);
        assert_eq!(sent[0].0, agent_a);
        assert_eq!(sent[1].0, agent_b);

        let payload: ContextEvent = serde_json::from_str(&sent[0].1).unwrap();
        match payload {
            ContextEvent::MessageCreated(event) => {
                assert_eq!(event.conversation_id, conversation_id);
                assert_eq!(event.message.id, message.id);
                assert_eq!(event.message.content, "hook fired");
            }
            _ => panic!("expected message_created event"),
        }
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

        let sent = sink.sent.lock().expect("sink mutex poisoned");
        assert!(sent.is_empty());
    }
}
