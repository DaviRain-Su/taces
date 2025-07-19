use axum::Router;
use crate::AppState;

pub mod auth;
pub mod user;
pub mod doctor;
pub mod appointment;
pub mod prescription;
pub mod department;
pub mod patient_group;
pub mod patient_profile;

pub fn create_routes() -> Router<AppState> {
    Router::new()
        .nest("/auth", auth::routes())
        .nest("/users", user::routes())
        .nest("/doctors", doctor::routes())
        .nest("/appointments", appointment::routes())
        .nest("/prescriptions", prescription::routes())
        .nest("/departments", department::routes())
        .nest("/patient-groups", patient_group::routes())
        .nest("/patient-profiles", patient_profile::routes())
}