use crate::controllers::video_consultation_controller::*;
use crate::middleware::auth::auth_middleware;
use crate::AppState;
use axum::{
    middleware,
    routing::{get, post, put},
    Router,
};

pub fn video_consultation_routes() -> Router<AppState> {
    Router::new()
        // Consultation Management
        .route("/", post(create_consultation))
        .route("/", get(list_consultations))
        .route("/:id", get(get_consultation))
        .route("/:id", put(update_consultation))
        .route("/:id/start", put(start_consultation))
        .route("/:id/end", put(end_consultation))
        .route("/:id/rate", post(rate_consultation))
        
        // Room Management
        .route("/room/:room_id/join", post(join_room))
        
        // WebRTC Signaling
        .route("/signal", post(send_signal))
        .route("/signal/:room_id", get(receive_signals))
        
        // Recording Management
        .route("/:id/recording/start", post(start_recording))
        .route("/recording/:id/complete", put(complete_recording))
        .route("/recording/:id", get(get_recording))
        .route("/:id/recordings", get(get_consultation_recordings))
        
        // Template Management
        .route("/templates", post(create_template))
        .route("/templates", get(list_doctor_templates))
        .route("/templates/:id", get(get_template))
        .route("/templates/:id/use", post(use_template))
        
        // Statistics
        .route("/statistics", get(get_consultation_statistics))
        
        // Apply authentication middleware to all routes
        .layer(middleware::from_fn(auth_middleware))
}