use crate::db::DbPool;
use crate::context_engine::ContextEngine;
use crate::link_preview::service::LinkPreviewService;
use crate::media::service::MediaService;
use crate::rate_limit::RateLimiter;
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
    pub context_engine: Arc<ContextEngine>,
    pub agent_limiter: RateLimiter,
    pub media_service: Arc<MediaService>,
    pub link_preview_service: Arc<LinkPreviewService>,
    pub nats: Option<Arc<async_nats::Client>>,
}
