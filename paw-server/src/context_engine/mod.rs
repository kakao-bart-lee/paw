use crate::db::DbPool;
use crate::ws::hub::Hub;
use async_trait::async_trait;
use paw_proto::{
    ContextConversationSettingsChangedMsg, ContextMemberJoinedMsg, ContextMemberLeftMsg,
    ContextMessageCreatedMsg, ContextMessageDeletedMsg, ContextThreadCreatedMsg,
    MessageReceivedMsg, PROTOCOL_VERSION,
};
use std::sync::Arc;

#[async_trait]
pub trait ContextEngine: Send + Sync {
    async fn on_message_created(&self, _event: ContextMessageCreatedMsg) -> anyhow::Result<()>;
    async fn on_message_deleted(&self, _event: ContextMessageDeletedMsg) -> anyhow::Result<()>;
    async fn on_member_joined(&self, _event: ContextMemberJoinedMsg) -> anyhow::Result<()>;
    async fn on_member_left(&self, _event: ContextMemberLeftMsg) -> anyhow::Result<()>;
    async fn on_thread_created(&self, _event: ContextThreadCreatedMsg) -> anyhow::Result<()>;
    async fn on_conversation_settings_changed(
        &self,
        _event: ContextConversationSettingsChangedMsg,
    ) -> anyhow::Result<()>;
}

pub struct DefaultContextEngine {
    _db: DbPool,
    _hub: Arc<Hub>,
}

impl DefaultContextEngine {
    pub fn new(db: DbPool, hub: Arc<Hub>) -> Self {
        Self { _db: db, _hub: hub }
    }
}

#[async_trait]
impl ContextEngine for DefaultContextEngine {
    async fn on_message_created(&self, _event: ContextMessageCreatedMsg) -> anyhow::Result<()> {
        Ok(())
    }

    async fn on_message_deleted(&self, _event: ContextMessageDeletedMsg) -> anyhow::Result<()> {
        Ok(())
    }

    async fn on_member_joined(&self, _event: ContextMemberJoinedMsg) -> anyhow::Result<()> {
        Ok(())
    }

    async fn on_member_left(&self, _event: ContextMemberLeftMsg) -> anyhow::Result<()> {
        Ok(())
    }

    async fn on_thread_created(&self, _event: ContextThreadCreatedMsg) -> anyhow::Result<()> {
        Ok(())
    }

    async fn on_conversation_settings_changed(
        &self,
        _event: ContextConversationSettingsChangedMsg,
    ) -> anyhow::Result<()> {
        Ok(())
    }
}

pub fn message_created_event(message: MessageReceivedMsg) -> ContextMessageCreatedMsg {
    ContextMessageCreatedMsg {
        v: PROTOCOL_VERSION,
        conversation_id: message.conversation_id,
        occurred_at: message.created_at,
        message,
    }
}
