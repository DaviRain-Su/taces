use crate::AppState;
use axum::Router;

pub mod appointment;
pub mod auth;
pub mod circle;
pub mod circle_post;
pub mod content;
pub mod department;
pub mod doctor;
pub mod file_upload;
pub mod live_stream;
pub mod notification;
pub mod patient_group;
pub mod patient_profile;
pub mod payment;
pub mod prescription;
pub mod review;
pub mod statistics;
pub mod template;
pub mod user;
pub mod video_consultation;
pub mod websocket;

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
        .nest("/templates", template::routes())
        .nest("/reviews", review::routes())
        .nest("/notifications", notification::routes())
        .nest("/statistics", statistics::routes())
        .nest("/payment", payment::routes())
        .nest(
            "/video-consultations",
            video_consultation::video_consultation_routes(),
        )
        .nest("/files", file_upload::file_upload_routes())
        .nest("/", payment::public_routes())
        .nest("/", live_stream::routes())
        .nest("/", circle::circle_routes())
        .nest("/", circle_post::circle_post_routes())
        .nest("/", websocket::routes())
}
