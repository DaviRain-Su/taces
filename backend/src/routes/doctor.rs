use crate::{controllers::doctor_controller, middleware::auth::auth_middleware, AppState};
use axum::{
    middleware,
    routing::{get, post, put},
    Router,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        // Public routes (no authentication required)
        .route("/", get(doctor_controller::list_doctors))
        .route("/:id", get(doctor_controller::get_doctor))
        // Protected routes (authentication required)
        .route(
            "/",
            post(doctor_controller::create_doctor).layer(middleware::from_fn(auth_middleware)),
        )
        .route(
            "/:id",
            put(doctor_controller::update_doctor).layer(middleware::from_fn(auth_middleware)),
        )
        .route(
            "/:id/photos",
            put(doctor_controller::update_doctor_photos)
                .layer(middleware::from_fn(auth_middleware)),
        )
        .route(
            "/by-user/:user_id",
            get(doctor_controller::get_doctor_by_user_id)
                .layer(middleware::from_fn(auth_middleware)),
        )
}
