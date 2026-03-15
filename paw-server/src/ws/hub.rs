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

impl Hub {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn try_register_with_limit(
        &self,
        user_id: Uuid,
        sender: WsSender,
        max_connections: usize,
    ) -> bool {
        let mut guard = self.connections.write().await;
        let senders = guard.entry(user_id).or_default();
        if senders.len() >= max_connections {
            return false;
        }
        senders.push(sender);
        true
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

    #[cfg(test)]
    pub async fn connection_count(&self, user_id: Uuid) -> usize {
        let guard = self.connections.read().await;
        guard
            .get(&user_id)
            .map(|senders| senders.len())
            .unwrap_or(0)
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

        assert!(hub.try_register_with_limit(user_id, tx1, usize::MAX).await);
        assert!(hub.try_register_with_limit(user_id, tx2, usize::MAX).await);

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

    #[tokio::test]
    async fn tracks_connection_count_per_user() {
        let hub = Hub::new();
        let user_id = Uuid::new_v4();
        let (tx1, _rx1) = mpsc::unbounded_channel::<Message>();
        let (tx2, _rx2) = mpsc::unbounded_channel::<Message>();

        assert_eq!(hub.connection_count(user_id).await, 0);
        assert!(
            hub.try_register_with_limit(user_id, tx1.clone(), usize::MAX)
                .await
        );
        assert_eq!(hub.connection_count(user_id).await, 1);
        assert!(hub.try_register_with_limit(user_id, tx2, usize::MAX).await);
        assert_eq!(hub.connection_count(user_id).await, 2);

        hub.unregister(user_id, &tx1).await;
        assert_eq!(hub.connection_count(user_id).await, 1);
    }

    #[tokio::test]
    async fn registration_respects_per_user_limit() {
        let hub = Hub::new();
        let user_id = Uuid::new_v4();
        let max_connections = 2;
        let (tx1, _rx1) = mpsc::unbounded_channel::<Message>();
        let (tx2, _rx2) = mpsc::unbounded_channel::<Message>();
        let (tx3, _rx3) = mpsc::unbounded_channel::<Message>();

        assert!(
            hub.try_register_with_limit(user_id, tx1, max_connections)
                .await
        );
        assert!(
            hub.try_register_with_limit(user_id, tx2, max_connections)
                .await
        );
        assert!(
            !hub.try_register_with_limit(user_id, tx3, max_connections)
                .await
        );
    }
}
