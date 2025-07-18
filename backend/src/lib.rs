pub mod config;
pub mod controllers;
pub mod middleware;
pub mod models;
pub mod routes;
pub mod services;
pub mod utils;

use config::{Config, database};

#[derive(Clone)]
pub struct AppState {
    pub config: Config,
    pub pool: database::DbPool,
}