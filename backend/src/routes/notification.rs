use crate::{
    controllers::notification_controller::*,
    middleware::auth::auth_middleware,
    AppState,
};
use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        // 通知管理
        .route("/", get(get_user_notifications))
        .route("/:id", get(get_notification_detail))
        .route("/:id/read", put(mark_notification_as_read))
        .route("/read-all", put(mark_all_as_read))
        .route("/:id", delete(delete_notification))
        .route("/stats", get(get_notification_stats))
        
        // 通知设置
        .route("/settings", get(get_notification_settings))
        .route("/settings", put(update_notification_settings))
        
        // 推送token
        .route("/push-token", post(register_push_token))
        
        // 系统公告（管理员）
        .route("/announcement", post(send_system_announcement))
        
        // 所有路由都需要认证
        .layer(middleware::from_fn(auth_middleware))
}