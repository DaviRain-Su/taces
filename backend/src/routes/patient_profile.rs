use crate::{controllers::patient_profile_controller, middleware::auth::auth_middleware, AppState};
use axum::{
    routing::{get, put},
    Router,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        // All routes require authentication and patient role
        .route(
            "/",
            get(patient_profile_controller::list_profiles)
                .post(patient_profile_controller::create_profile),
        )
        .route(
            "/default",
            get(patient_profile_controller::get_default_profile),
        )
        .route(
            "/:id",
            get(patient_profile_controller::get_profile)
                .put(patient_profile_controller::update_profile)
                .delete(patient_profile_controller::delete_profile),
        )
        .route("/:id/default", put(patient_profile_controller::set_default))
        .layer(axum::middleware::from_fn(auth_middleware))
}
