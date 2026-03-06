mod agents;
mod auth;
mod backup;
mod channels;
mod db;
mod keys;
mod media;
mod messages;
mod push;
mod users;
mod ws;

use auth::AppState;
use axum::{
    Router,
    extract::DefaultBodyLimit,
    middleware,
    routing::{delete, get, patch, post, put},
};
use std::net::SocketAddr;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "paw_server=debug".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://postgres:postgres@localhost:5432/paw".to_string());
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "dev_only_change_me_in_production".to_string());

    let db = db::create_pool(&database_url).await?;
    let hub = Arc::new(ws::hub::Hub::new());
    let media_service = Arc::new(media::service::MediaService::new_from_env().await);

    let nats_url = std::env::var("NATS_URL").unwrap_or_else(|_| "nats://localhost:4222".to_string());
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

    let state = AppState {
        db: db.clone(),
        jwt_secret,
        hub: hub.clone(),
        media_service,
        nats: nats_client,
    };

    tokio::spawn(ws::pg_listener::start_pg_listener(db.clone(), hub));

    let media_upload = Router::new()
        .route("/media/upload", post(media::handlers::upload))
        .layer(DefaultBodyLimit::max(50 * 1024 * 1024));

    let protected_routes = Router::new()
        .route("/users/me", get(users::handlers::get_me).patch(users::handlers::update_me))
        .route("/users/search", get(users::handlers::search_user))
        .route("/users/:user_id", get(users::handlers::get_user))
        .route("/conversations", get(messages::handlers::list_conversations))
        .route("/conversations", post(messages::handlers::create_conversation))
        .route(
            "/conversations/:id",
            patch(messages::handlers::update_group_name_handler),
        )
        .route(
            "/conversations/:id/members",
            post(messages::handlers::add_member_handler),
        )
        .route(
            "/conversations/:id/members/:user_id",
            delete(messages::handlers::remove_member_handler),
        )
        .route(
            "/conversations/:id/agents",
            post(agents::handlers::invite_agent_handler),
        )
        .route(
            "/conversations/:id/agents/:agent_id",
            delete(agents::handlers::remove_agent_handler),
        )
        .route(
            "/conversations/:conv_id/messages",
            post(messages::handlers::send_message),
        )
        .route(
            "/conversations/:conv_id/messages",
            get(messages::handlers::get_messages),
        )
        .route("/api/v1/channels", post(channels::handlers::create_channel))
        .route("/api/v1/channels", get(channels::handlers::list_channels))
        .route(
            "/api/v1/channels/:id/subscribe",
            post(channels::handlers::subscribe_channel),
        )
        .route(
            "/api/v1/channels/:id/subscribe",
            delete(channels::handlers::unsubscribe_channel),
        )
        .route(
            "/api/v1/channels/:id/messages",
            post(channels::handlers::send_channel_message),
        )
        .route(
            "/api/v1/channels/:id/messages",
            get(channels::handlers::get_channel_messages),
        )
        .route("/api/v1/keys/upload", post(keys::handlers::upload_keys_handler))
        .route(
            "/api/v1/keys/:user_id",
            get(keys::handlers::get_key_bundle_handler),
        )
        .route("/media/:media_id/url", get(media::handlers::get_url))
        .route("/api/v1/agents/register", post(agents::handlers::register_agent_handler))
        .route("/api/v1/agents/:agent_id", get(agents::handlers::get_agent_handler))
        .route("/api/v1/agents/:agent_id/revoke", post(agents::handlers::revoke_agent_handler))
        .route("/api/v1/backup/initiate", post(backup::handlers::initiate_backup))
        .route("/api/v1/backup/list", get(backup::handlers::list_backups))
        .route("/api/v1/backup/:id/restore", post(backup::handlers::restore_backup))
        .route("/api/v1/backup/:id", delete(backup::handlers::delete_backup))
        .route("/api/v1/backup/settings", put(backup::handlers::update_settings))
        .route("/api/v1/backup/settings", get(backup::handlers::get_settings))
        .route(
            "/api/v1/push/register",
            post(push::handlers::register_push_token),
        )
        .route(
            "/api/v1/push/register",
            delete(push::handlers::unregister_push_token),
        )
        .route(
            "/api/v1/conversations/:id/mute",
            post(push::handlers::mute_conversation),
        )
        .route(
            "/api/v1/conversations/:id/mute",
            delete(push::handlers::unmute_conversation),
        )
        .merge(media_upload)
        .route_layer(middleware::from_fn_with_state(
            state.clone(),
            auth::middleware::auth_middleware,
        ));

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/auth/request-otp", post(auth::handlers::request_otp))
        .route("/auth/verify-otp", post(auth::handlers::verify_otp))
        .route("/auth/register-device", post(auth::handlers::register_device))
        .route("/auth/refresh", post(auth::handlers::refresh_token))
        .route("/ws", get(ws::handler::ws_handler))
        .route("/agent/ws", get(agents::handlers::agent_ws_handler))
        .merge(protected_routes)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::info!("Paw server listening on {}", addr);
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn health_check() -> &'static str {
    "OK"
}
