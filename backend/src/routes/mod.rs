use crate::AppState;
use axum::Router;

pub mod appointment;
pub mod auth;
pub mod circle;
pub mod circle_post;
pub mod content;
pub mod department;
pub mod doctor;
pub mod live_stream;
pub mod patient_group;
pub mod patient_profile;
pub mod prescription;
pub mod user;

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
        .nest("/content", content::routes())
        .nest("/", live_stream::routes())
        .nest("/", circle::circle_routes())
        .nest("/", circle_post::circle_post_routes())
}
