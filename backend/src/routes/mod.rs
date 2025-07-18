use axum::Router;

pub mod auth;
pub mod user;
pub mod doctor;
pub mod appointment;
pub mod prescription;

pub fn create_routes() -> Router {
    Router::new()
        .nest("/api/v1/auth", auth::routes())
        .nest("/api/v1/users", user::routes())
        .nest("/api/v1/doctors", doctor::routes())
        .nest("/api/v1/appointments", appointment::routes())
        .nest("/api/v1/prescriptions", prescription::routes())
}