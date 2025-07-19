use crate::{controllers::auth_controller, AppState};
use axum::{routing::post, Router};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(auth_controller::register))
        .route("/login", post(auth_controller::login))
}
