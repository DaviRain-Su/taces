use crate::{controllers::prescription_controller, middleware::auth::auth_middleware, AppState};
use axum::{
    middleware,
    routing::{get, post},
    Router,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(prescription_controller::list_prescriptions))
        .route("/:id", get(prescription_controller::get_prescription))
        .route("/", post(prescription_controller::create_prescription))
        .route(
            "/code/:code",
            get(prescription_controller::get_prescription_by_code),
        )
        .route(
            "/doctor/:doctor_id",
            get(prescription_controller::get_doctor_prescriptions),
        )
        .route(
            "/patient/:patient_id",
            get(prescription_controller::get_patient_prescriptions),
        )
        .layer(middleware::from_fn(auth_middleware))
}
