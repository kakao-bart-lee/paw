use crate::context_engine::ContextEngine;
use crate::db::DbPool;
use crate::media::service::MediaService;
use crate::ws::hub::Hub;
use std::sync::Arc;

pub mod device;
pub mod handlers;
pub mod jwt;
pub mod middleware;
pub mod otp;
pub mod otp_attempts;

#[derive(Clone)]
pub struct AppState {
    pub db: DbPool,
    pub jwt_secret: String,
    pub default_locale: String,
    pub hub: Arc<Hub>,
    pub context_engine: Arc<dyn ContextEngine>,
    pub media_service: Arc<MediaService>,
    pub nats: Option<Arc<async_nats::Client>>,
}
