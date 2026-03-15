use super::models::{
    ContextEventType, ConversationSettingsChangedHook, MemberJoinedData, MemberJoinedHook,
    MemberLeftData, MemberLeftHook, MessageCreatedData, MessageCreatedHook, MessageDeletedData,
    MessageDeletedHook, MessageEditedData, MessageEditedHook, ThreadCreatedData, ThreadCreatedHook,
};
use crate::db::DbPool;
use crate::ws::hub::Hub;
use async_trait::async_trait;
use chrono::{DateTime, Utc};
use paw_proto::ContextEvent;
use serde_json::Value;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Clone)]
pub struct ContextEngine {
    db: DbPool,
    hub: Arc<Hub>,
}

impl ContextEngine {
    pub fn new(db: DbPool, hub: Arc<Hub>) -> Self {
        Self { db, hub }
    }

    fn dispatch_event(
        &self,
        event_type: ContextEventType,
        conversation_id: Uuid,
        data: Value,
        timestamp: DateTime<Utc>,
    ) {
        let db = self.db.clone();
        let hub = self.hub.clone();

        tokio::spawn(async move {
            let agent_ids = match sqlx::query_scalar::<_, Uuid>(
                "SELECT agent_id FROM conversation_agents AS agent_conversations WHERE conversation_id = $1",
            )
            .bind(conversation_id)
            .fetch_all(db.as_ref())
            .await
            {
                Ok(ids) => ids,
                Err(err) => {
                    tracing::error!(%err, %conversation_id, "context engine failed to load agent conversations");
                    return;
                }
            };

            if agent_ids.is_empty() {
                return;
            }

            let event = ContextEvent {
                event_type: event_type.as_str().to_owned(),
                conversation_id,
                data,
                timestamp,
            };

            if let Err(err) = deliver_context_event(hub, agent_ids, event).await {
                tracing::warn!(%err, %conversation_id, "context engine event delivery failed");
            }
        });
    }
}

#[async_trait]
#[allow(dead_code)]
pub trait LifecycleHooks: Send + Sync {
    async fn on_message_created(&self, hook: MessageCreatedHook);

    async fn on_message_edited(&self, hook: MessageEditedHook);

    async fn on_message_deleted(&self, hook: MessageDeletedHook);

    async fn on_member_joined(&self, hook: MemberJoinedHook);

    async fn on_member_left(&self, hook: MemberLeftHook);

    async fn on_thread_created(&self, hook: ThreadCreatedHook);

    async fn on_conversation_settings_changed(&self, hook: ConversationSettingsChangedHook);
}

#[async_trait]
impl LifecycleHooks for ContextEngine {
    async fn on_message_created(&self, hook: MessageCreatedHook) {
        let payload = serde_json::to_value(MessageCreatedData {
            message_id: hook.message_id,
            thread_id: hook.thread_id,
            sender_id: hook.sender_id,
            content: hook.content,
            format: hook.format,
            seq: hook.seq,
        })
        .unwrap_or(Value::Null);

        self.dispatch_event(
            ContextEventType::MessageCreated,
            hook.conversation_id,
            payload,
            hook.timestamp,
        );
    }

    async fn on_message_edited(&self, hook: MessageEditedHook) {
        let payload = serde_json::to_value(MessageEditedData {
            message_id: hook.message_id,
            thread_id: hook.thread_id,
            edited_by: hook.edited_by,
            content: hook.content,
            format: hook.format,
        })
        .unwrap_or(Value::Null);

        self.dispatch_event(
            ContextEventType::MessageEdited,
            hook.conversation_id,
            payload,
            hook.timestamp,
        );
    }

    async fn on_message_deleted(&self, hook: MessageDeletedHook) {
        let payload = serde_json::to_value(MessageDeletedData {
            message_id: hook.message_id,
            thread_id: hook.thread_id,
            deleted_by: hook.deleted_by,
        })
        .unwrap_or(Value::Null);

        self.dispatch_event(
            ContextEventType::MessageDeleted,
            hook.conversation_id,
            payload,
            hook.timestamp,
        );
    }

    async fn on_member_joined(&self, hook: MemberJoinedHook) {
        let payload = serde_json::to_value(MemberJoinedData {
            member_id: hook.member_id,
            joined_by: hook.joined_by,
        })
        .unwrap_or(Value::Null);

        self.dispatch_event(
            ContextEventType::MemberJoined,
            hook.conversation_id,
            payload,
            hook.timestamp,
        );
    }

    async fn on_member_left(&self, hook: MemberLeftHook) {
        let payload = serde_json::to_value(MemberLeftData {
            member_id: hook.member_id,
            left_by: hook.left_by,
        })
        .unwrap_or(Value::Null);

        self.dispatch_event(
            ContextEventType::MemberLeft,
            hook.conversation_id,
            payload,
            hook.timestamp,
        );
    }

    async fn on_thread_created(&self, hook: ThreadCreatedHook) {
        let payload = serde_json::to_value(ThreadCreatedData {
            thread_id: hook.thread_id,
            root_message_id: hook.root_message_id,
            created_by: hook.created_by,
            title: hook.title,
        })
        .unwrap_or(Value::Null);

        self.dispatch_event(
            ContextEventType::ThreadCreated,
            hook.conversation_id,
            payload,
            hook.timestamp,
        );
    }

    async fn on_conversation_settings_changed(&self, hook: ConversationSettingsChangedHook) {
        let payload = serde_json::json!({
            "changed_by": hook.changed_by,
            "changes": hook.changes,
        });

        self.dispatch_event(
            ContextEventType::ConversationSettingsChanged,
            hook.conversation_id,
            payload,
            hook.timestamp,
        );
    }
}

async fn deliver_context_event(
    hub: Arc<Hub>,
    agent_ids: Vec<Uuid>,
    event: ContextEvent,
) -> anyhow::Result<()> {
    let payload = serde_json::to_string(&event)?;

    for agent_id in agent_ids {
        hub.send_to_user_nonblocking(agent_id, &payload).await;
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::extract::ws::Message;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn deliver_context_event_sends_payload_to_registered_agent() {
        let hub = Arc::new(Hub::new());
        let agent_id = Uuid::new_v4();
        let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
        assert!(hub.try_register_with_limit(agent_id, tx, usize::MAX).await);

        let conversation_id = Uuid::new_v4();
        let event = ContextEvent {
            event_type: ContextEventType::MemberJoined.as_str().to_owned(),
            conversation_id,
            data: serde_json::json!({ "member_id": Uuid::new_v4() }),
            timestamp: Utc::now(),
        };

        deliver_context_event(hub, vec![agent_id], event.clone())
            .await
            .expect("deliver succeeds");

        let frame = rx.recv().await.expect("frame should be delivered");
        let Message::Text(payload) = frame else {
            panic!("expected text websocket frame")
        };
        let parsed: ContextEvent = serde_json::from_str(payload.as_ref()).expect("valid payload");
        assert_eq!(parsed.event_type, event.event_type);
        assert_eq!(parsed.conversation_id, event.conversation_id);
        assert_eq!(parsed.data["member_id"], event.data["member_id"]);
    }

    #[tokio::test]
    async fn deliver_context_event_broadcasts_to_all_agents() {
        let hub = Arc::new(Hub::new());
        let first_agent = Uuid::new_v4();
        let second_agent = Uuid::new_v4();

        let (tx1, mut rx1) = mpsc::unbounded_channel::<Message>();
        let (tx2, mut rx2) = mpsc::unbounded_channel::<Message>();

        assert!(
            hub.try_register_with_limit(first_agent, tx1, usize::MAX)
                .await
        );
        assert!(
            hub.try_register_with_limit(second_agent, tx2, usize::MAX)
                .await
        );

        let event = ContextEvent {
            event_type: ContextEventType::ThreadCreated.as_str().to_owned(),
            conversation_id: Uuid::new_v4(),
            data: serde_json::json!({ "thread_id": Uuid::new_v4() }),
            timestamp: Utc::now(),
        };

        deliver_context_event(hub, vec![first_agent, second_agent], event)
            .await
            .expect("deliver succeeds");

        assert!(matches!(rx1.recv().await, Some(Message::Text(_))));
        assert!(matches!(rx2.recv().await, Some(Message::Text(_))));
    }
}
