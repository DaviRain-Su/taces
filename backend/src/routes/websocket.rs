use crate::{
    services::websocket_service::websocket_handler,
    AppState,
};
use axum::{routing::get, Router};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/ws", get(websocket_handler))
}