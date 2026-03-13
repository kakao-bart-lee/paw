#![allow(clippy::empty_line_after_doc_comments)]

pub mod crypto;
pub mod db;
pub mod search;

pub use crypto::{create_account, decrypt, encrypt, AccountKeys};
pub use db::{AppDatabase, ConversationRecord, DbError, DbResult, MessageRecord};
pub use search::{SearchResult, SearchService};

pub fn ping() -> String {
    "paw-core-ok".to_string()
}

uniffi::include_scaffolding!("paw_core");
