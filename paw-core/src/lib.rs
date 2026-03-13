#![allow(clippy::empty_line_after_doc_comments)]

pub mod crypto;

pub use crypto::{create_account, decrypt, encrypt, AccountKeys};

pub fn ping() -> String {
    "paw-core-ok".to_string()
}

uniffi::include_scaffolding!("paw_core");
