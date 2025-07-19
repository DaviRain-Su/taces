use crate::{controllers::user_controller, middleware::auth::auth_middleware, AppState};
use axum::{
    middleware,
    routing::{delete, get, put},
    Router,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(user_controller::list_users))
        .route("/:id", get(user_controller::get_user))
        .route("/:id", put(user_controller::update_user))
        .route("/:id", delete(user_controller::delete_user))
        .route("/batch/delete", delete(user_controller::batch_delete_users))
        .route("/batch/export", get(user_controller::export_users))
        .layer(middleware::from_fn(auth_middleware))
}
