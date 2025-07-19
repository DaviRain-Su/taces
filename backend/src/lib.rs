pub mod config;
pub mod controllers;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod services;
pub mod utils;

pub use config::{database, redis, storage, Config};

use std::sync::Arc;
use services::websocket_service::WebSocketManager;
use aws_sdk_s3::Client as S3Client;

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub pool: database::DbPool,
    pub redis: Option<redis::RedisPool>,
    pub ws_manager: Arc<WebSocketManager>,
    pub s3_client: Option<S3Client>,
}
