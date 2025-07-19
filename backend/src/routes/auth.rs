use crate::{controllers::auth_controller, middleware::auth::auth_middleware, AppState};
use axum::{routing::post, Router};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/register", post(auth_controller::register))
        .route("/login", post(auth_controller::login))
        .route("/logout", post(auth_controller::logout).layer(axum::middleware::from_fn(auth_middleware)))
}
