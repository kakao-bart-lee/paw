pub mod connection;
pub mod handler;
pub mod hub;
pub mod pg_listener;

pub const HEARTBEAT_PING_SECONDS: u64 = 30;
pub const HEARTBEAT_TIMEOUT_SECONDS: u64 = 90;
pub const MAX_WS_MESSAGE_SIZE_BYTES: usize = 64 * 1024;
pub const MAX_WS_CONNECTIONS_PER_USER: usize = 5;
