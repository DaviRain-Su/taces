use axum::{
    routing::{get, post, put},
    Router,
    middleware,
};
use crate::{
    controllers::doctor_controller,
    middleware::auth::auth_middleware,
};

pub fn routes() -> Router {
    Router::new()
        .route("/", get(doctor_controller::list_doctors))
        .route("/:id", get(doctor_controller::get_doctor))
        .route("/", post(doctor_controller::create_doctor))
        .route("/:id", put(doctor_controller::update_doctor))
        .route("/:id/photos", put(doctor_controller::update_doctor_photos))
        .route("/by-user/:user_id", get(doctor_controller::get_doctor_by_user_id))
        .layer(middleware::from_fn(auth_middleware))
}