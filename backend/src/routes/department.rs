use axum::{
    Router,
    routing::{get, post, put, delete},
};
use crate::{
    AppState,
    controllers::department_controller,
    middleware::auth::auth_middleware,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        // Public routes - anyone can view departments
        .route("/", get(department_controller::list_departments))
        .route("/:id", get(department_controller::get_department))
        .route("/code/:code", get(department_controller::get_department_by_code))
        // Protected routes - admin only
        .route("/", post(department_controller::create_department)
            .layer(axum::middleware::from_fn(auth_middleware)))
        .route("/:id", put(department_controller::update_department)
            .layer(axum::middleware::from_fn(auth_middleware)))
        .route("/:id", delete(department_controller::delete_department)
            .layer(axum::middleware::from_fn(auth_middleware)))
}