use crate::AppState;
use crate::controllers::circle_controller::*;
use crate::middleware::auth::auth_middleware;
use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

pub fn circle_routes() -> Router<AppState> {
    Router::new()
        // Public routes (require authentication)
        .route("/circles", post(create_circle))
        .route("/circles", get(get_circles))
        .route("/circles/:id", get(get_circle_by_id))
        .route("/circles/:id", put(update_circle))
        .route("/circles/:id", delete(delete_circle))
        .route("/circles/:id/join", post(join_circle))
        .route("/circles/:id/leave", post(leave_circle))
        .route("/circles/:id/members", get(get_circle_members))
        .route("/circles/:id/members/:user_id/role", put(update_member_role))
        .route("/circles/:id/members/:user_id", delete(remove_member))
        .route("/my-circles", get(get_user_circles))
        .layer(middleware::from_fn(auth_middleware))
}