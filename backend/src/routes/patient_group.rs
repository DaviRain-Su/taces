use crate::{controllers::patient_group_controller, middleware::auth::auth_middleware, AppState};
use axum::{
    routing::{get, post},
    Router,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        // All routes require authentication and doctor role
        .route(
            "/",
            get(patient_group_controller::list_groups).post(patient_group_controller::create_group),
        )
        .route(
            "/:id",
            get(patient_group_controller::get_group)
                .put(patient_group_controller::update_group)
                .delete(patient_group_controller::delete_group),
        )
        .route(
            "/:id/members",
            post(patient_group_controller::add_members)
                .delete(patient_group_controller::remove_members),
        )
        .route("/:id/message", post(patient_group_controller::send_message))
        .layer(axum::middleware::from_fn(auth_middleware))
}
