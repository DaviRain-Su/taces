use backend::{
    config::database, routes, services::websocket_service::WebSocketManager, AppState, Config,
};

use axum::Router;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let pool = database::create_pool().await.unwrap();
    let config = Config::from_env().unwrap();

    let state = AppState {
        config,
        pool,
        redis: None,
        s3_client: None,
        ws_manager: Arc::new(WebSocketManager::new()),
    };

    let app: Router<AppState> = Router::new()
        .nest("/api/v1", routes::create_routes())
        .with_state(state);

    // 打印所有路由信息
    println!("Routes have been configured successfully");

    // 测试路由
    println!("Review routes should be at /api/v1/reviews/*");
}
