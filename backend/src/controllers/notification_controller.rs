use crate::{
    models::{notification::*, ApiResponse},
    services::notification_service::NotificationService,
    middleware::auth::AuthUser,
    AppState,
};
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct NotificationQuery {
    pub status: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Serialize)]
pub struct NotificationListResponse {
    pub items: Vec<NotificationResponse>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// 获取用户通知列表
pub async fn get_user_notifications(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<NotificationQuery>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).min(100);
    
    // 解析状态参数
    let status = match query.status.as_deref() {
        Some("unread") => Some(NotificationStatus::Unread),
        Some("read") => Some(NotificationStatus::Read),
        _ => None,
    };

    match NotificationService::get_user_notifications(
        &state.pool,
        auth_user.user_id,
        status,
        page,
        page_size,
    )
    .await
    {
        Ok((notifications, total)) => {
            let response = NotificationListResponse {
                items: notifications.into_iter().map(|n| n.into()).collect(),
                total,
                page,
                page_size,
            };
            
            Json(ApiResponse::success("获取通知列表成功", response)).into_response()
        }
        Err(e) => {
            eprintln!("获取通知列表失败: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("获取通知列表失败")),
            )
                .into_response()
        }
    }
}

/// 获取单个通知详情
pub async fn get_notification_detail(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match NotificationService::get_notification_by_id(&state.pool, id, auth_user.user_id).await {
        Ok(Some(notification)) => {
            let response: NotificationResponse = notification.into();
            Json(ApiResponse::success("获取通知详情成功", response)).into_response()
        }
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("通知不存在")),
        )
            .into_response(),
        Err(e) => {
            eprintln!("获取通知详情失败: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("获取通知详情失败")),
            )
                .into_response()
        }
    }
}

/// 标记通知为已读
pub async fn mark_notification_as_read(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match NotificationService::mark_as_read(&state.pool, id, auth_user.user_id).await {
        Ok(true) => Json(ApiResponse::success("标记已读成功", json!({ "success": true }))).into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("通知不存在或已读")),
        )
            .into_response(),
        Err(e) => {
            eprintln!("标记已读失败: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("标记已读失败")),
            )
                .into_response()
        }
    }
}

/// 标记所有通知为已读
pub async fn mark_all_as_read(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> impl IntoResponse {
    match NotificationService::mark_all_as_read(&state.pool, auth_user.user_id).await {
        Ok(count) => Json(ApiResponse::success(
            "标记所有通知已读成功",
            json!({ "count": count }),
        ))
        .into_response(),
        Err(e) => {
            eprintln!("标记所有通知已读失败: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("标记所有通知已读失败")),
            )
                .into_response()
        }
    }
}

/// 删除通知
pub async fn delete_notification(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match NotificationService::delete_notification(&state.pool, id, auth_user.user_id).await {
        Ok(true) => Json(ApiResponse::success("删除通知成功", json!({ "success": true }))).into_response(),
        Ok(false) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<()>::error("通知不存在")),
        )
            .into_response(),
        Err(e) => {
            eprintln!("删除通知失败: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("删除通知失败")),
            )
                .into_response()
        }
    }
}

/// 获取通知统计
pub async fn get_notification_stats(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> impl IntoResponse {
    match NotificationService::get_notification_stats(&state.pool, auth_user.user_id).await {
        Ok(stats) => Json(ApiResponse::success("获取通知统计成功", stats)).into_response(),
        Err(e) => {
            eprintln!("获取通知统计失败: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("获取通知统计失败")),
            )
                .into_response()
        }
    }
}

/// 获取通知设置
pub async fn get_notification_settings(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
) -> impl IntoResponse {
    match NotificationService::get_user_notification_settings(&state.pool, auth_user.user_id).await {
        Ok(settings) => Json(ApiResponse::success("获取通知设置成功", settings)).into_response(),
        Err(e) => {
            eprintln!("获取通知设置失败: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("获取通知设置失败")),
            )
                .into_response()
        }
    }
}

/// 更新通知设置
pub async fn update_notification_settings(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(dto): Json<UpdateNotificationSettingsDto>,
) -> impl IntoResponse {
    match NotificationService::update_notification_settings(&state.pool, auth_user.user_id, dto).await {
        Ok(settings) => Json(ApiResponse::success("更新通知设置成功", settings)).into_response(),
        Err(e) => {
            eprintln!("更新通知设置失败: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("更新通知设置失败")),
            )
                .into_response()
        }
    }
}

/// 注册推送token
pub async fn register_push_token(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(dto): Json<RegisterPushTokenDto>,
) -> impl IntoResponse {
    match NotificationService::register_push_token(&state.pool, auth_user.user_id, dto).await {
        Ok(token) => Json(ApiResponse::success("注册推送token成功", token)).into_response(),
        Err(e) => {
            eprintln!("注册推送token失败: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("注册推送token失败")),
            )
                .into_response()
        }
    }
}

/// 发送系统公告（仅管理员）
pub async fn send_system_announcement(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(dto): Json<CreateNotificationDto>,
) -> impl IntoResponse {
    // 检查是否为管理员
    if auth_user.role != "admin" {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error("无权限发送系统公告")),
        )
            .into_response();
    }

    // 获取所有用户ID
    let user_ids_result: Result<Vec<(Uuid,)>, sqlx::Error> = sqlx::query_as(
        "SELECT id FROM users WHERE status = 'active'"
    )
    .fetch_all(&state.pool)
    .await;

    match user_ids_result {
        Ok(users) => {
            let user_ids: Vec<Uuid> = users.into_iter().map(|(id,)| id).collect();
            
            match NotificationService::create_bulk_notifications(
                &state.pool,
                user_ids,
                NotificationType::SystemAnnouncement,
                dto.title,
                dto.content,
                dto.related_id,
            )
            .await
            {
                Ok(notifications) => Json(ApiResponse::success(
                    "发送系统公告成功",
                    json!({ "count": notifications.len() }),
                ))
                .into_response(),
                Err(e) => {
                    eprintln!("发送系统公告失败: {:?}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<()>::error("发送系统公告失败")),
                    )
                        .into_response()
                }
            }
        }
        Err(e) => {
            eprintln!("获取用户列表失败: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("获取用户列表失败")),
            )
                .into_response()
        }
    }
}

/// 创建通知（内部使用）
pub async fn create_notification_internal(
    pool: &DbPool,
    dto: CreateNotificationDto,
) -> Result<Notification, String> {
    NotificationService::create_notification(pool, dto)
        .await
        .map_err(|e| format!("创建通知失败: {:?}", e))
}

use crate::config::database::DbPool;