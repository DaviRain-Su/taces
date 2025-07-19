use crate::{
    controllers::live_stream_controller::*,
    middleware::auth::auth_middleware,
    AppState,
};
use axum::{
    middleware,
    routing::{get, post, put},
    Router,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        // Public routes
        .route("/live-streams", get(list_live_streams))
        .route("/live-streams/upcoming", get(get_upcoming_live_streams))
        .route("/live-streams/:id", get(get_live_stream))
        // Protected routes - must be authenticated
        .route(
            "/live-streams/my",
            get(get_my_live_streams).layer(middleware::from_fn(auth_middleware)),
        )
        .route(
            "/live-streams",
            post(create_live_stream).layer(middleware::from_fn(auth_middleware)),
        )
        .route(
            "/live-streams/:id",
            put(update_live_stream)
                .delete(delete_live_stream)
                .layer(middleware::from_fn(auth_middleware)),
        )
        .route(
            "/live-streams/:id/start",
            post(start_live_stream).layer(middleware::from_fn(auth_middleware)),
        )
        .route(
            "/live-streams/:id/end",
            post(end_live_stream).layer(middleware::from_fn(auth_middleware)),
        )
}