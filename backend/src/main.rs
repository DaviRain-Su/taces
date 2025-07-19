use axum::{routing::get, Router};
use dotenv::dotenv;
use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tracing_subscriber;

use backend::{
    config::{database, Config},
    routes, AppState,
};

#[tokio::main]
async fn main() {
    dotenv().ok();

    tracing_subscriber::fmt()
        .with_target(false)
        .compact()
        .init();

    let config = Config::from_env().expect("Failed to load configuration");
    let pool = database::create_pool()
        .await
        .expect("Failed to create database pool");

    // Run migrations
    if let Err(e) = database::run_migrations(&pool).await {
        tracing::error!("Failed to run migrations: {}", e);
    }

    let server_port = config.server_port;
    let app = create_app(config, pool).await;

    let addr = SocketAddr::from(([127, 0, 0, 1], server_port));
    tracing::info!("TCM Telemedicine Platform listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind to address");

    axum::serve(listener, app)
        .await
        .expect("Failed to start server");
}

async fn create_app(config: Config, pool: database::DbPool) -> Router {
    let state = AppState { config, pool };

    Router::new()
        .route("/", get(root))
        .route("/health", get(health_check))
        .nest("/api/v1", routes::create_routes())
        .layer(CorsLayer::permissive())
        .with_state(state)
}

async fn root() -> &'static str {
    "TCM Telemedicine Platform API"
}

async fn health_check() -> &'static str {
    "OK"
}
