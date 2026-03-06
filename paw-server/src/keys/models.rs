use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct UploadKeysRequest {
    pub identity_key: String,
    pub signed_prekey: String,
    pub signed_prekey_sig: String,
    pub one_time_prekeys: Vec<OneTimeKeyEntry>,
}

#[derive(Debug, Deserialize)]
pub struct OneTimeKeyEntry {
    pub key_id: i64,
    pub key: String,
}

#[derive(Debug, Serialize)]
pub struct KeyBundle {
    pub user_id: Uuid,
    pub identity_key: String,
    pub signed_prekey: String,
    pub signed_prekey_sig: String,
    pub one_time_prekey: Option<OneTimeKeyResponse>,
    pub replenish_prekeys: bool,
}

#[derive(Debug, Serialize)]
pub struct OneTimeKeyResponse {
    pub key_id: i64,
    pub key: String,
}
