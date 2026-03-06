use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct InitiateBackupResponse {
    pub backup_id: Uuid,
    pub upload_url: String,
    pub s3_key: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct BackupEntry {
    pub id: Uuid,
    pub size_bytes: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ListBackupsResponse {
    pub backups: Vec<BackupEntry>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct RestoreBackupResponse {
    pub download_url: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct BackupSettings {
    pub frequency: BackupFrequency,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum BackupFrequency {
    Daily,
    Weekly,
    Never,
}

impl BackupFrequency {
    pub fn as_str(&self) -> &'static str {
        match self {
            BackupFrequency::Daily => "daily",
            BackupFrequency::Weekly => "weekly",
            BackupFrequency::Never => "never",
        }
    }
}

impl std::fmt::Display for BackupFrequency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}
