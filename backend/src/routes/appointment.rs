use axum::{
    routing::{get, post, put},
    Router,
    middleware,
};
use crate::{
    controllers::appointment_controller,
    middleware::auth::auth_middleware,
    AppState,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(appointment_controller::list_appointments))
        .route("/:id", get(appointment_controller::get_appointment))
        .route("/", post(appointment_controller::create_appointment))
        .route("/:id", put(appointment_controller::update_appointment))
        .route("/:id/cancel", put(appointment_controller::cancel_appointment))
        .route("/doctor/:doctor_id", get(appointment_controller::get_doctor_appointments))
        .route("/patient/:patient_id", get(appointment_controller::get_patient_appointments))
        .route("/available-slots", get(appointment_controller::get_available_slots))
        .layer(middleware::from_fn(auth_middleware))
}