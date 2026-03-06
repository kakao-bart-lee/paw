mod auth;
mod db;
mod ws;

use auth::AppState;
use axum::{
    Router,
    routing::{get, post},
};
use std::net::SocketAddr;
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
    let hub = std::sync::Arc::new(ws::hub::Hub::new());
    let state = AppState {
        db: db.clone(),
        jwt_secret,
        hub: hub.clone(),
    };

    tokio::spawn(ws::pg_listener::start_pg_listener(db.clone(), hub));

    let app = Router::new()
        .route("/health", get(health_check))
        .route("/auth/request-otp", post(auth::handlers::request_otp))
        .route("/auth/verify-otp", post(auth::handlers::verify_otp))
        .route("/auth/register-device", post(auth::handlers::register_device))
        .route("/auth/refresh", post(auth::handlers::refresh_token))
        .route("/ws", get(ws::handler::ws_handler))
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
