use crate::db::DbPool;
use crate::media::service::MediaService;
use crate::rate_limit::RateLimiter;
use crate::ws::hub::Hub;
use std::sync::Arc;

use self::otp_attempts::OtpAttemptGuard;

pub mod device;
pub mod handlers;
pub mod jwt;
pub mod middleware;
pub mod otp;
pub mod otp_attempts;

#[derive(Clone)]
#[allow(dead_code)]
pub struct AppState {
    pub db: DbPool,
    pub jwt_secret: String,
    pub default_locale: String,
    pub hub: Arc<Hub>,
    pub agent_limiter: RateLimiter,
    pub media_service: Arc<MediaService>,
    pub nats: Option<Arc<async_nats::Client>>,
    pub otp_attempt_guard: OtpAttemptGuard,
}
