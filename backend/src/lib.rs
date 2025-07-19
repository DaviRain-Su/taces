pub mod config;
pub mod controllers;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod services;
pub mod utils;

pub use config::{database, redis, Config};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub pool: database::DbPool,
    pub redis: Option<redis::RedisPool>,
}
