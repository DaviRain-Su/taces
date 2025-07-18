use axum::{
    routing::post,
    Router,
};
use crate::controllers::auth_controller;

pub fn routes() -> Router {
    Router::new()
        .route("/register", post(auth_controller::register))
        .route("/login", post(auth_controller::login))
}