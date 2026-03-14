mod agents;
mod auth;
mod backup;
mod channels;
mod db;
mod i18n;
mod keys;
mod media;
mod messages;
mod metrics;
mod moderation;
mod observability;
mod push;
mod rate_limit;
mod users;
mod ws;

use auth::AppState;
use axum::{
    extract::DefaultBodyLimit,
    middleware,
    routing::{delete, get, patch, post, put},
    Extension, Router,
};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use rate_limit::{ProtectedLimiter, PublicLimiter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();

    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "paw_server=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:35432/paw".to_string());
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "dev_only_change_me_in_production".to_string());
    let default_locale = std::env::var("PAW_DEFAULT_LOCALE")
        .ok()
        .and_then(|value| i18n::normalize_locale(&value))
        .unwrap_or_else(|| "ko-KR".to_string());

    let db = db::create_pool(&database_url).await?;
    let hub = Arc::new(ws::hub::Hub::new());
    let media_service = Arc::new(media::service::MediaService::new_from_env().await);

    let nats_url =
        std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:34223".to_string());
    let nats_client = match async_nats::connect(&nats_url).await {
        Ok(client) => {
            tracing::info!("NATS connected at {}", nats_url);
            Some(Arc::new(client))
        }
        Err(e) => {
            tracing::warn!("NATS unavailable ({}): agent gateway degraded", e);
            None
        }
    };

    let prometheus_handle = metrics::init_metrics();

    let state = AppState {
        db: db.clone(),
        jwt_secret,
        default_locale,
        hub: hub.clone(),
        media_service,
        nats: nats_client,
    };

    tokio::spawn(ws::pg_listener::start_pg_listener(db.clone(), hub));

    let public_limiter = rate_limit::public_limiter_from_env();
    let protected_limiter = rate_limit::protected_limiter_from_env();
    rate_limit::spawn_cleanup_task(vec![public_limiter.clone(), protected_limiter.clone()]);

    let media_upload = Router::new()
        .route("/media/upload", post(media::handlers::upload))
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024));

    let protected_routes = Router::new()
        .route(
            "/users/me",
            get(users::handlers::get_me).patch(users::handlers::update_me),
        )
        .route("/users/search", get(users::handlers::search_user))
        .route("/users/{user_id}", get(users::handlers::get_user))
        .route(
            "/conversations",
            get(messages::handlers::list_conversations),
        )
        .route(
            "/conversations",
            post(messages::handlers::create_conversation),
        )
        .route(
            "/conversations/{id}",
            patch(messages::handlers::update_group_name_handler),
        )
        .route(
            "/conversations/{id}/members",
            post(messages::handlers::add_member_handler),
        )
        .route(
            "/conversations/{id}/members/{user_id}",
            delete(messages::handlers::remove_member_handler),
        )
        .route(
            "/conversations/{id}/agents",
            post(agents::handlers::invite_agent_handler),
        )
        .route(
            "/conversations/{id}/agents/{agent_id}",
            delete(agents::handlers::remove_agent_handler),
        )
        .route(
            "/conversations/{conv_id}/messages",
            post(messages::handlers::send_message),
        )
        .route(
            "/conversations/{conv_id}/messages",
            get(messages::handlers::get_messages),
        )
        .route("/api/v1/channels", post(channels::handlers::create_channel))
        .route("/api/v1/channels", get(channels::handlers::list_channels))
        .route(
            "/api/v1/channels/{id}/subscribe",
            post(channels::handlers::subscribe_channel),
        )
        .route(
            "/api/v1/channels/{id}/subscribe",
            delete(channels::handlers::unsubscribe_channel),
        )
        .route(
            "/api/v1/channels/{id}/messages",
            post(channels::handlers::send_channel_message),
        )
        .route(
            "/api/v1/channels/{id}/messages",
            get(channels::handlers::get_channel_messages),
        )
        .route(
            "/api/v1/keys/upload",
            post(keys::handlers::upload_keys_handler),
        )
        .route(
            "/api/v1/keys/{user_id}",
            get(keys::handlers::get_key_bundle_handler),
        )
        .route("/media/{media_id}/url", get(media::handlers::get_url))
        .route(
            "/api/v1/agents/register",
            post(agents::handlers::register_agent_handler),
        )
        .route(
            "/api/v1/agents/{agent_id}",
            get(agents::handlers::get_agent_handler),
        )
        .route(
            "/api/v1/agents/{agent_id}/revoke",
            post(agents::handlers::revoke_agent_handler),
        )
        .route(
            "/api/v1/agents/{agent_id}/publish",
            put(agents::handlers::publish_agent_handler),
        )
        .route(
            "/api/v1/marketplace/agents",
            get(agents::handlers::marketplace_search_handler),
        )
        .route(
            "/api/v1/marketplace/agents/{agent_id}",
            get(agents::handlers::marketplace_agent_detail_handler),
        )
        .route(
            "/api/v1/marketplace/agents/{agent_id}/install",
            post(agents::handlers::install_agent_handler),
        )
        .route(
            "/api/v1/marketplace/agents/{agent_id}/install",
            delete(agents::handlers::uninstall_agent_handler),
        )
        .route(
            "/api/v1/marketplace/installed",
            get(agents::handlers::list_installed_agents_handler),
        )
        .route(
            "/api/v1/backup/initiate",
            post(backup::handlers::initiate_backup),
        )
        .route("/api/v1/backup/list", get(backup::handlers::list_backups))
        .route(
            "/api/v1/backup/{id}/restore",
            post(backup::handlers::restore_backup),
        )
        .route(
            "/api/v1/backup/{id}",
            delete(backup::handlers::delete_backup),
        )
        .route(
            "/api/v1/backup/settings",
            put(backup::handlers::update_settings),
        )
        .route(
            "/api/v1/backup/settings",
            get(backup::handlers::get_settings),
        )
        .route("/api/v1/reports", post(moderation::handlers::create_report))
        .route(
            "/api/v1/users/{id}/block",
            post(moderation::handlers::block_user),
        )
        .route(
            "/api/v1/users/{id}/block",
            delete(moderation::handlers::unblock_user),
        )
        .route(
            "/api/v1/users/blocked",
            get(moderation::handlers::list_blocked_users),
        )
        .route(
            "/api/v1/admin/users/{id}/suspend",
            post(moderation::handlers::suspend_user),
        )
        .route(
            "/api/v1/admin/users/{id}/suspend",
            delete(moderation::handlers::unsuspend_user),
        )
        .route(
            "/api/v1/admin/reports",
            get(moderation::handlers::list_pending_reports),
        )
        .route(
            "/api/v1/push/register",
            post(push::handlers::register_push_token),
        )
        .route(
            "/api/v1/push/register",
            delete(push::handlers::unregister_push_token),
        )
        .route(
            "/api/v1/conversations/{id}/mute",
            post(push::handlers::mute_conversation),
        )
        .route(
            "/api/v1/conversations/{id}/mute",
            delete(push::handlers::unmute_conversation),
        )
        .merge(media_upload)
        .layer(middleware::from_fn(rate_limit::protected_rate_limit))
        .layer(Extension(ProtectedLimiter(protected_limiter)))
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::middleware::auth_middleware,
        ));

    let public_routes = Router::new()
        .route("/auth/request-otp", post(auth::handlers::request_otp))
        .route("/auth/verify-otp", post(auth::handlers::verify_otp))
        .route(
            "/auth/register-device",
            post(auth::handlers::register_device),
        )
        .route("/auth/refresh", post(auth::handlers::refresh_token))
        .layer(middleware::from_fn(rate_limit::public_rate_limit))
        .layer(Extension(PublicLimiter(public_limiter)));

    let metrics_route = Router::new()
        .route("/metrics", get(metrics::metrics_handler))
        .with_state(prometheus_handle);

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/ws", get(ws::handler::ws_handler))
        .route("/agent/ws", get(agents::handlers::agent_ws_handler))
        .merge(public_routes)
        .merge(protected_routes)
        .layer(middleware::from_fn_with_state(
            state.clone(),
            i18n::locale_middleware,
        ))
        .layer(middleware::from_fn(observability::request_id_middleware))
        .layer(CorsLayer::permissive())
        .with_state(state)
        .merge(metrics_route)
        .layer(middleware::from_fn(metrics::metrics_middleware));

    let host = std::env::var("PAW_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("PAW_PORT")
        .or_else(|_| std::env::var("PORT"))
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(38173);
    let addr: SocketAddr = format!("{}:{}", host, port).parse()?;
    tracing::info!("Paw server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}
