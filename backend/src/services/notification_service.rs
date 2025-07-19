use crate::{
    config::database::DbPool,
    models::notification::*,
};
use chrono::Utc;
use sqlx::{query, query_as};
use uuid::Uuid;

pub struct NotificationService;

impl NotificationService {
    /// 创建通知
    pub async fn create_notification(
        pool: &DbPool,
        dto: CreateNotificationDto,
    ) -> Result<Notification, sqlx::Error> {
        let metadata = dto.metadata.unwrap_or(serde_json::json!({}));
        
        let notification = query_as!(
            Notification,
            r#"
            INSERT INTO notifications (user_id, type, title, content, related_id, metadata)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, user_id, type as "notification_type: NotificationType", 
                      title, content, related_id, status as "status: NotificationStatus", 
                      metadata, created_at, read_at
            "#,
            dto.user_id,
            dto.notification_type as NotificationType,
            dto.title,
            dto.content,
            dto.related_id,
            metadata
        )
        .fetch_one(pool)
        .await?;

        Ok(notification)
    }

    /// 批量创建通知（用于群发）
    pub async fn create_bulk_notifications(
        pool: &DbPool,
        user_ids: Vec<Uuid>,
        notification_type: NotificationType,
        title: String,
        content: String,
        related_id: Option<Uuid>,
    ) -> Result<Vec<Notification>, sqlx::Error> {
        let mut notifications = Vec::new();

        for user_id in user_ids {
            let dto = CreateNotificationDto {
                user_id,
                notification_type: notification_type.clone(),
                title: title.clone(),
                content: content.clone(),
                related_id,
                metadata: None,
            };

            if let Ok(notification) = Self::create_notification(pool, dto).await {
                notifications.push(notification);
            }
        }

        Ok(notifications)
    }

    /// 获取用户通知列表
    pub async fn get_user_notifications(
        pool: &DbPool,
        user_id: Uuid,
        status: Option<NotificationStatus>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<Notification>, i64), sqlx::Error> {
        let offset = (page - 1) * page_size;

        // 构建查询条件
        let status_condition = match status {
            Some(s) => format!("AND status = '{}'", match s {
                NotificationStatus::Unread => "unread",
                NotificationStatus::Read => "read",
                NotificationStatus::Deleted => "deleted",
            }),
            None => "AND status != 'deleted'".to_string(),
        };

        // 获取总数
        let count_query = format!(
            "SELECT COUNT(*) as count FROM notifications WHERE user_id = $1 {}",
            status_condition
        );
        let total: i64 = sqlx::query_scalar(&count_query)
            .bind(user_id)
            .fetch_one(pool)
            .await?;

        // 获取通知列表
        let list_query = format!(
            r#"
            SELECT id, user_id, type as "notification_type: NotificationType", 
                   title, content, related_id, status as "status: NotificationStatus", 
                   metadata, created_at, read_at
            FROM notifications
            WHERE user_id = $1 {}
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
            status_condition
        );

        let notifications = sqlx::query_as::<_, Notification>(&list_query)
            .bind(user_id)
            .bind(page_size)
            .bind(offset)
            .fetch_all(pool)
            .await?;

        Ok((notifications, total))
    }

    /// 获取单个通知
    pub async fn get_notification_by_id(
        pool: &DbPool,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Notification>, sqlx::Error> {
        let notification = query_as!(
            Notification,
            r#"
            SELECT id, user_id, type as "notification_type: NotificationType", 
                   title, content, related_id, status as "status: NotificationStatus", 
                   metadata, created_at, read_at
            FROM notifications
            WHERE id = $1 AND user_id = $2 AND status != 'deleted'
            "#,
            id,
            user_id
        )
        .fetch_optional(pool)
        .await?;

        Ok(notification)
    }

    /// 标记通知为已读
    pub async fn mark_as_read(
        pool: &DbPool,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = query!(
            r#"
            UPDATE notifications
            SET status = 'read', read_at = $3
            WHERE id = $1 AND user_id = $2 AND status = 'unread'
            "#,
            id,
            user_id,
            Utc::now()
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 批量标记为已读
    pub async fn mark_all_as_read(pool: &DbPool, user_id: Uuid) -> Result<u64, sqlx::Error> {
        let result = query!(
            r#"
            UPDATE notifications
            SET status = 'read', read_at = $2
            WHERE user_id = $1 AND status = 'unread'
            "#,
            user_id,
            Utc::now()
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected())
    }

    /// 删除通知（软删除）
    pub async fn delete_notification(
        pool: &DbPool,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = query!(
            r#"
            UPDATE notifications
            SET status = 'deleted'
            WHERE id = $1 AND user_id = $2 AND status != 'deleted'
            "#,
            id,
            user_id
        )
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 获取用户通知统计
    pub async fn get_notification_stats(
        pool: &DbPool,
        user_id: Uuid,
    ) -> Result<NotificationStats, sqlx::Error> {
        let stats = query_as!(
            NotificationStats,
            r#"
            SELECT 
                COUNT(*) FILTER (WHERE status != 'deleted') as "total_count!",
                COUNT(*) FILTER (WHERE status = 'unread') as "unread_count!",
                COUNT(*) FILTER (WHERE status = 'read') as "read_count!"
            FROM notifications
            WHERE user_id = $1
            "#,
            user_id
        )
        .fetch_one(pool)
        .await?;

        Ok(stats)
    }

    /// 获取用户通知设置
    pub async fn get_user_notification_settings(
        pool: &DbPool,
        user_id: Uuid,
    ) -> Result<Vec<NotificationSettings>, sqlx::Error> {
        let settings = query_as!(
            NotificationSettings,
            r#"
            SELECT id, user_id, notification_type as "notification_type: NotificationType", 
                   enabled, email_enabled, sms_enabled, push_enabled, 
                   created_at, updated_at
            FROM notification_settings
            WHERE user_id = $1
            ORDER BY notification_type
            "#,
            user_id
        )
        .fetch_all(pool)
        .await?;

        Ok(settings)
    }

    /// 更新通知设置
    pub async fn update_notification_settings(
        pool: &DbPool,
        user_id: Uuid,
        dto: UpdateNotificationSettingsDto,
    ) -> Result<NotificationSettings, sqlx::Error> {
        // 先检查是否存在该设置
        let exists = query!(
            r#"
            SELECT COUNT(*) as count
            FROM notification_settings
            WHERE user_id = $1 AND notification_type = $2
            "#,
            user_id,
            dto.notification_type as NotificationType
        )
        .fetch_one(pool)
        .await?;

        let settings = if exists.count.unwrap_or(0) > 0 {
            // 更新现有设置
            query_as!(
                NotificationSettings,
                r#"
                UPDATE notification_settings
                SET enabled = COALESCE($3, enabled),
                    email_enabled = COALESCE($4, email_enabled),
                    sms_enabled = COALESCE($5, sms_enabled),
                    push_enabled = COALESCE($6, push_enabled),
                    updated_at = CURRENT_TIMESTAMP
                WHERE user_id = $1 AND notification_type = $2
                RETURNING id, user_id, notification_type as "notification_type: NotificationType", 
                          enabled, email_enabled, sms_enabled, push_enabled, 
                          created_at, updated_at
                "#,
                user_id,
                dto.notification_type as NotificationType,
                dto.enabled,
                dto.email_enabled,
                dto.sms_enabled,
                dto.push_enabled
            )
            .fetch_one(pool)
            .await?
        } else {
            // 创建新设置
            query_as!(
                NotificationSettings,
                r#"
                INSERT INTO notification_settings 
                (user_id, notification_type, enabled, email_enabled, sms_enabled, push_enabled)
                VALUES ($1, $2, $3, $4, $5, $6)
                RETURNING id, user_id, notification_type as "notification_type: NotificationType", 
                          enabled, email_enabled, sms_enabled, push_enabled, 
                          created_at, updated_at
                "#,
                user_id,
                dto.notification_type as NotificationType,
                dto.enabled.unwrap_or(true),
                dto.email_enabled.unwrap_or(false),
                dto.sms_enabled.unwrap_or(false),
                dto.push_enabled.unwrap_or(true)
            )
            .fetch_one(pool)
            .await?
        };

        Ok(settings)
    }

    /// 注册推送token
    pub async fn register_push_token(
        pool: &DbPool,
        user_id: Uuid,
        dto: RegisterPushTokenDto,
    ) -> Result<PushToken, sqlx::Error> {
        let device_info = dto.device_info.unwrap_or(serde_json::json!({}));

        // 先禁用该用户该设备类型的其他token
        query!(
            r#"
            UPDATE push_tokens
            SET active = false, updated_at = CURRENT_TIMESTAMP
            WHERE user_id = $1 AND device_type = $2
            "#,
            user_id,
            dto.device_type
        )
        .execute(pool)
        .await?;

        // 创建新token
        let token = query_as!(
            PushToken,
            r#"
            INSERT INTO push_tokens (user_id, device_type, token, device_info, active)
            VALUES ($1, $2, $3, $4, true)
            RETURNING id, user_id, device_type, token, device_info, active, 
                      created_at, updated_at
            "#,
            user_id,
            dto.device_type,
            dto.token,
            device_info
        )
        .fetch_one(pool)
        .await?;

        Ok(token)
    }

    /// 记录短信发送日志
    pub async fn log_sms(
        pool: &DbPool,
        user_id: Option<Uuid>,
        dto: SendSmsDto,
        status: &str,
        error_message: Option<String>,
        provider: Option<String>,
    ) -> Result<SmsLog, sqlx::Error> {
        let log = query_as!(
            SmsLog,
            r#"
            INSERT INTO sms_logs (user_id, phone, template_code, template_params, 
                                  status, error_message, provider, sent_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, user_id, phone, template_code, template_params, 
                      status, error_message, provider, created_at, sent_at
            "#,
            user_id,
            dto.phone,
            dto.template_code,
            dto.template_params,
            status,
            error_message,
            provider,
            if status == "sent" { Some(Utc::now()) } else { None }
        )
        .fetch_one(pool)
        .await?;

        Ok(log)
    }

    /// 记录邮件发送日志
    pub async fn log_email(
        pool: &DbPool,
        user_id: Option<Uuid>,
        dto: SendEmailDto,
        status: &str,
        error_message: Option<String>,
        provider: Option<String>,
    ) -> Result<EmailLog, sqlx::Error> {
        let log = query_as!(
            EmailLog,
            r#"
            INSERT INTO email_logs (user_id, email, subject, body, 
                                    status, error_message, provider, sent_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, user_id, email, subject, body, 
                      status, error_message, provider, created_at, sent_at
            "#,
            user_id,
            dto.email,
            dto.subject,
            dto.body,
            status,
            error_message,
            provider,
            if status == "sent" { Some(Utc::now()) } else { None }
        )
        .fetch_one(pool)
        .await?;

        Ok(log)
    }

    /// 检查用户是否应该接收某类型通知
    pub async fn should_send_notification(
        pool: &DbPool,
        user_id: Uuid,
        notification_type: &NotificationType,
    ) -> Result<(bool, bool, bool, bool), sqlx::Error> {
        let settings = query!(
            r#"
            SELECT enabled, email_enabled, sms_enabled, push_enabled
            FROM notification_settings
            WHERE user_id = $1 AND notification_type = $2
            "#,
            user_id,
            notification_type as &NotificationType
        )
        .fetch_optional(pool)
        .await?;

        match settings {
            Some(s) => Ok((s.enabled, s.email_enabled, s.sms_enabled, s.push_enabled)),
            None => Ok((true, false, false, true)), // 默认启用通知和推送，禁用邮件和短信
        }
    }
}