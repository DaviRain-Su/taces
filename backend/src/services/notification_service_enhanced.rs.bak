use crate::{
    config::database::DbPool,
    models::notification::*,
    services::websocket_service::WebSocketManager,
    utils::errors::AppError,
};
use chrono::Utc;
use sqlx::Row;
use std::sync::Arc;
use uuid::Uuid;

pub struct NotificationServiceEnhanced;

impl NotificationServiceEnhanced {
    /// Send notification and push via WebSocket
    pub async fn send_notification_with_ws(
        db: &DbPool,
        ws_manager: &Arc<WebSocketManager>,
        notification_dto: SendNotificationDto,
    ) -> Result<Notification, AppError> {
        // Create notification in database
        let notification_id = Uuid::new_v4();
        let now = Utc::now();
        
        let query = r#"
            INSERT INTO notifications (
                id, user_id, type, title, content, 
                related_id, metadata, status, created_at, updated_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, 'unread', ?, ?)
        "#;
        
        sqlx::query(query)
            .bind(notification_id.to_string())
            .bind(notification_dto.user_id.to_string())
            .bind(notification_dto.notification_type.to_string())
            .bind(&notification_dto.title)
            .bind(&notification_dto.content)
            .bind(notification_dto.related_id.map(|id| id.to_string()))
            .bind(notification_dto.metadata.as_ref().map(|m| serde_json::to_string(m).unwrap_or_default()))
            .bind(&now)
            .bind(&now)
            .execute(db)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to create notification: {}", e)))?;
        
        // Get the created notification
        let notification = Self::get_notification_by_id(db, notification_id).await?;
        
        // Send via WebSocket if user is online
        ws_manager.send_notification(notification_dto.user_id, notification.clone()).await;
        
        Ok(notification)
    }
    
    /// Batch send notifications with WebSocket
    pub async fn batch_send_with_ws(
        db: &DbPool,
        ws_manager: &Arc<WebSocketManager>,
        batch_dto: BatchNotificationDto,
    ) -> Result<Vec<Notification>, AppError> {
        let mut notifications = Vec::new();
        
        for user_id in batch_dto.user_ids {
            let notification_dto = SendNotificationDto {
                user_id,
                notification_type: batch_dto.notification_type.clone(),
                title: batch_dto.title.clone(),
                content: batch_dto.content.clone(),
                related_id: batch_dto.related_id,
                metadata: batch_dto.metadata.clone(),
            };
            
            if let Ok(notification) = Self::send_notification_with_ws(db, ws_manager, notification_dto).await {
                notifications.push(notification);
            }
        }
        
        Ok(notifications)
    }
    
    /// Send notification to all users with a specific role
    pub async fn send_to_role_with_ws(
        db: &DbPool,
        ws_manager: &Arc<WebSocketManager>,
        role: &str,
        notification_type: NotificationType,
        title: String,
        content: String,
        related_id: Option<Uuid>,
    ) -> Result<u64, AppError> {
        // Get all users with the specified role
        let query = "SELECT id FROM users WHERE role = ? AND status = 'active'";
        let rows = sqlx::query(query)
            .bind(role)
            .fetch_all(db)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to fetch users: {}", e)))?;
        
        let mut count = 0;
        for row in rows {
            let user_id: String = row.get("id");
            if let Ok(user_uuid) = Uuid::parse_str(&user_id) {
                let notification_dto = SendNotificationDto {
                    user_id: user_uuid,
                    notification_type: notification_type.clone(),
                    title: title.clone(),
                    content: content.clone(),
                    related_id,
                    metadata: None,
                };
                
                if Self::send_notification_with_ws(db, ws_manager, notification_dto).await.is_ok() {
                    count += 1;
                }
            }
        }
        
        Ok(count)
    }
    
    /// Helper function to get notification by ID
    async fn get_notification_by_id(db: &DbPool, id: Uuid) -> Result<Notification, AppError> {
        let query = r#"
            SELECT id, user_id, type, title, content, related_id, metadata, 
                   status, read_at, created_at, updated_at
            FROM notifications
            WHERE id = ?
        "#;
        
        let row = sqlx::query(query)
            .bind(id.to_string())
            .fetch_one(db)
            .await
            .map_err(|e| AppError::NotFound(format!("Notification not found: {}", e)))?;
        
        Ok(Notification {
            id: Uuid::parse_str(row.get("id")).unwrap(),
            user_id: Uuid::parse_str(row.get("user_id")).unwrap(),
            notification_type: match row.get::<String, _>("type").as_str() {
                "appointment_reminder" => NotificationType::AppointmentReminder,
                "prescription_ready" => NotificationType::PrescriptionReady,
                "new_message" => NotificationType::NewMessage,
                "system_announcement" => NotificationType::SystemAnnouncement,
                "live_stream_start" => NotificationType::LiveStreamStart,
                "doctor_online" => NotificationType::DoctorOnline,
                _ => NotificationType::SystemAnnouncement,
            },
            title: row.get("title"),
            content: row.get("content"),
            related_id: row.get::<Option<String>, _>("related_id")
                .and_then(|s| Uuid::parse_str(&s).ok()),
            metadata: row.get::<Option<String>, _>("metadata")
                .and_then(|s| serde_json::from_str(&s).ok()),
            status: match row.get::<String, _>("status").as_str() {
                "unread" => NotificationStatus::Unread,
                "read" => NotificationStatus::Read,
                "deleted" => NotificationStatus::Deleted,
                _ => NotificationStatus::Unread,
            },
            read_at: row.get("read_at"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }
}