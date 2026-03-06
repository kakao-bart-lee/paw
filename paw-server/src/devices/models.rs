use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Device {
    pub id: Uuid,
    pub user_id: Uuid,
    pub device_name: String,
    pub platform: String,
    pub last_active_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn device_list_model_serializes_expected_shape() {
        let device = Device {
            id: Uuid::new_v4(),
            user_id: Uuid::new_v4(),
            device_name: "MacBook Pro".to_owned(),
            platform: "desktop".to_owned(),
            last_active_at: None,
            created_at: Utc::now(),
        };

        let body = json!({ "devices": [device] });
        assert!(body["devices"].is_array());
        assert_eq!(body["devices"][0]["device_name"], "MacBook Pro");
        assert_eq!(body["devices"][0]["platform"], "desktop");
    }
}
