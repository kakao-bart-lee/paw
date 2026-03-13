use std::sync::Arc;

use paw_proto::{MessageFormat, MessageReceivedMsg};

use crate::db::{AppDatabase, DbResult, MessageRecord};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SyncRequest {
    pub conversation_id: String,
    pub last_seq: i64,
}

pub struct SyncService {
    db: Arc<AppDatabase>,
    request_sync: Arc<dyn Fn(String, i64) + Send + Sync>,
}

impl SyncService {
    pub fn new(
        db: Arc<AppDatabase>,
        request_sync: impl Fn(String, i64) + Send + Sync + 'static,
    ) -> Self {
        Self {
            db,
            request_sync: Arc::new(request_sync),
        }
    }

    pub fn sync_all_conversations(&self) -> DbResult<Vec<SyncRequest>> {
        let conversations = self.db.get_all_conversations()?;
        let mut requests = Vec::with_capacity(conversations.len());

        for conversation in conversations {
            let last_seq = self.db.get_last_seq(&conversation.id)?;
            (self.request_sync)(conversation.id.clone(), last_seq);
            requests.push(SyncRequest {
                conversation_id: conversation.id,
                last_seq,
            });
        }

        Ok(requests)
    }

    pub fn persist_message(&self, msg: &MessageReceivedMsg) -> DbResult<MessageRecord> {
        let record = MessageRecord {
            id: msg.id.to_string(),
            conversation_id: msg.conversation_id.to_string(),
            sender_id: msg.sender_id.to_string(),
            content: msg.content.clone(),
            format: message_format_name(&msg.format).to_string(),
            seq: msg.seq,
            created_at: msg.created_at,
            is_me: false,
            is_agent: false,
        };

        self.db.upsert_message(&record)?;
        self.db
            .update_last_seq(&record.conversation_id, record.seq)?;
        Ok(record)
    }
}

fn message_format_name(format: &MessageFormat) -> &'static str {
    match format {
        MessageFormat::Plain => "plain",
        MessageFormat::Markdown => "markdown",
    }
}

#[cfg(test)]
mod tests {
    use chrono::{TimeZone, Utc};
    use uuid::Uuid;

    use crate::db::ConversationRecord;

    use super::*;

    #[test]
    fn sync_requests_follow_conversation_sort_order() {
        let db = Arc::new(AppDatabase::open_in_memory().unwrap());
        db.upsert_conversation(&ConversationRecord {
            id: "conv-1".into(),
            name: "one".into(),
            avatar_url: None,
            last_seq: 0,
            unread_count: 0,
            updated_at: Utc.timestamp_opt(100, 0).single().unwrap(),
        })
        .unwrap();
        db.upsert_conversation(&ConversationRecord {
            id: "conv-2".into(),
            name: "two".into(),
            avatar_url: None,
            last_seq: 0,
            unread_count: 0,
            updated_at: Utc.timestamp_opt(200, 0).single().unwrap(),
        })
        .unwrap();

        let service = SyncService::new(db, |_conversation_id, _last_seq| {});
        let requests = service.sync_all_conversations().unwrap();

        assert_eq!(requests[0].conversation_id, "conv-2");
        assert_eq!(requests[1].conversation_id, "conv-1");
    }

    #[test]
    fn persist_message_updates_conversation_head() {
        let db = Arc::new(AppDatabase::open_in_memory().unwrap());
        let conversation_id = Uuid::new_v4().to_string();
        db.upsert_conversation(&ConversationRecord {
            id: conversation_id.clone(),
            name: "conv".into(),
            avatar_url: None,
            last_seq: 0,
            unread_count: 0,
            updated_at: Utc::now(),
        })
        .unwrap();

        let service = SyncService::new(db.clone(), |_conversation_id, _last_seq| {});
        let msg = MessageReceivedMsg {
            v: 1,
            id: Uuid::new_v4(),
            conversation_id: Uuid::parse_str(&conversation_id).unwrap(),
            sender_id: Uuid::new_v4(),
            content: "hello".into(),
            format: MessageFormat::Markdown,
            seq: 9,
            created_at: Utc::now(),
            blocks: vec![],
        };

        service.persist_message(&msg).unwrap();
        assert_eq!(db.get_all_conversations().unwrap()[0].last_seq, 9);
    }
}
