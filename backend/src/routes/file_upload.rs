use crate::controllers::file_upload_controller::*;
use crate::middleware::auth::auth_middleware;
use crate::AppState;
use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

pub fn file_upload_routes() -> Router<AppState> {
    Router::new()
        // File Management
        .route("/upload", post(create_upload))
        .route("/upload/:id/complete", put(complete_upload))
        .route("/", get(list_files))
        .route("/:id", get(get_file))
        .route("/:id", delete(delete_file))
        .route("/stats", get(get_file_stats))
        // Configuration (admin only)
        .route("/config/upload", get(get_upload_config))
        .route("/config/image", get(get_image_config))
        .route("/config/video", get(get_video_config))
        .route("/config/:category/:key", put(update_system_config))
        // Apply authentication middleware to all routes
        .layer(middleware::from_fn(auth_middleware))
}
