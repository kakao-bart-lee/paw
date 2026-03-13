use axum::extract::ws::Message;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use uuid::Uuid;

pub type WsSender = mpsc::UnboundedSender<Message>;

#[derive(Clone)]
pub struct Hub {
    connections: Arc<RwLock<HashMap<Uuid, Vec<WsSender>>>>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn sends_to_all_connections_for_same_user() {
        let hub = Hub::new();
        let user_id = Uuid::new_v4();

        let (tx1, mut rx1) = mpsc::unbounded_channel::<Message>();
        let (tx2, mut rx2) = mpsc::unbounded_channel::<Message>();

        hub.register(user_id, tx1).await;
        hub.register(user_id, tx2).await;

        hub.send_to_user(user_id, "{\"v\":1,\"type\":\"ping\"}")
            .await;

        let msg1 = rx1
            .recv()
            .await
            .expect("first connection should receive message");
        let msg2 = rx2
            .recv()
            .await
            .expect("second connection should receive message");

        assert!(matches!(msg1, Message::Text(_)));
        assert!(matches!(msg2, Message::Text(_)));
    }
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

    /// Non-blocking send — drops message if receiver buffer is full.
    /// Used for streaming frames where dropping is preferable to blocking.
    pub async fn send_to_user_nonblocking(&self, user_id: Uuid, msg: &str) {
        let mut guard = self.connections.write().await;
        if let Some(senders) = guard.get_mut(&user_id) {
            senders.retain(|tx| match tx.send(Message::Text(msg.to_owned().into())) {
                Ok(_) => true,
                Err(err) => {
                    tracing::warn!(%user_id, error = %err, "dropping websocket message for slow/disconnected client");
                    false
                }
            });
            if senders.is_empty() {
                guard.remove(&user_id);
            }
        }
    }

    pub async fn is_user_connected(&self, user_id: Uuid) -> bool {
        let guard = self.connections.read().await;
        guard
            .get(&user_id)
            .map(|senders| !senders.is_empty())
            .unwrap_or(false)
    }

    pub async fn broadcast_to_conversation(&self, user_ids: Vec<Uuid>, msg: &str) {
        for user_id in user_ids {
            self.send_to_user(user_id, msg).await;
        }
    }

    pub async fn send_to_conversation(
        &self,
        _conversation_id: Uuid,
        user_ids: Vec<Uuid>,
        msg: &str,
    ) {
        for user_id in user_ids {
            self.send_to_user_nonblocking(user_id, msg).await;
        }
    }
}
