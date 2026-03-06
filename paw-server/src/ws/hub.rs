use axum::extract::ws::Message;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use uuid::Uuid;

pub type WsSender = mpsc::UnboundedSender<Message>;

#[derive(Clone)]
pub struct Hub {
    connections: Arc<RwLock<HashMap<Uuid, Vec<WsSender>>>>,
}

impl Hub {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn register(&self, user_id: Uuid, sender: WsSender) {
        let mut guard = self.connections.write().await;
        guard.entry(user_id).or_default().push(sender);
    }

    pub async fn unregister(&self, user_id: Uuid, sender: &WsSender) {
        let mut guard = self.connections.write().await;
        if let Some(senders) = guard.get_mut(&user_id) {
            senders.retain(|tx| !tx.same_channel(sender));
            if senders.is_empty() {
                guard.remove(&user_id);
            }
        }
    }

    pub async fn send_to_user(&self, user_id: Uuid, msg: &str) {
        let mut guard = self.connections.write().await;
        if let Some(senders) = guard.get_mut(&user_id) {
            senders.retain(|tx| tx.send(Message::Text(msg.to_owned().into())).is_ok());
            if senders.is_empty() {
                guard.remove(&user_id);
            }
        }
    }

    pub async fn broadcast_to_conversation(&self, user_ids: Vec<Uuid>, msg: &str) {
        for user_id in user_ids {
            self.send_to_user(user_id, msg).await;
        }
    }
}
