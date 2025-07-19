use crate::{
    config::database::DbPool,
    models::notification::*,
};
use chrono::Utc;
use sqlx::{query, query_as};
use uuid::Uuid;

pub struct NotificationService;

impl NotificationService {
    fn parse_notification_from_row(row: &sqlx::mysql::MySqlRow) -> Result<Notification, sqlx::Error> {
        use sqlx::Row;
        
        Ok(Notification {
            id: Uuid::parse_str(row.get("id")).map_err(|_| sqlx::Error::ColumnDecode {
                index: "id".to_string(),
                source: Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UUID")),
            })?,
            user_id: Uuid::parse_str(row.get("user_id")).map_err(|_| sqlx::Error::ColumnDecode {
                index: "user_id".to_string(),
                source: Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UUID")),
            })?,
            notification_type: row.get("notification_type"),
            title: row.get("title"),
            content: row.get("content"),
            related_id: row.get::<Option<String>, _>("related_id")
                .map(|s| Uuid::parse_str(&s).ok())
                .flatten(),
            status: row.get("status"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
            read_at: row.get("read_at"),
        })
    }
    
    fn parse_notification_settings_from_row(row: &sqlx::mysql::MySqlRow) -> Result<NotificationSettings, sqlx::Error> {
        use sqlx::Row;
        
        Ok(NotificationSettings {
            id: Uuid::parse_str(row.get("id")).map_err(|_| sqlx::Error::ColumnDecode {
                index: "id".to_string(),
                source: Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UUID")),
            })?,
            user_id: Uuid::parse_str(row.get("user_id")).map_err(|_| sqlx::Error::ColumnDecode {
                index: "user_id".to_string(),
                source: Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UUID")),
            })?,
            notification_type: row.get("notification_type"),
            enabled: row.get("enabled"),
            email_enabled: row.get("email_enabled"),
            sms_enabled: row.get("sms_enabled"),
            push_enabled: row.get("push_enabled"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }
    /// 创建通知
    pub async fn create_notification(
        pool: &DbPool,
        dto: CreateNotificationDto,
    ) -> Result<Notification, sqlx::Error> {
        let metadata = dto.metadata.unwrap_or(serde_json::json!({}));
        let notification_id = Uuid::new_v4();
        
        // Insert the notification
        let result = sqlx::query(
            r#"
            INSERT INTO notifications (id, user_id, type, title, content, related_id, metadata, status, created_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, 'unread', NOW())
            "#
        )
        .bind(notification_id.to_string())
        .bind(dto.user_id.to_string())
        .bind(&dto.notification_type.to_string())
        .bind(&dto.title)
        .bind(&dto.content)
        .bind(dto.related_id.map(|id| id.to_string()))
        .bind(&metadata)
        .execute(pool)
        .await?;

        // Fetch the created notification
        let query = r#"
            SELECT id, user_id, type as notification_type, 
                   title, content, related_id, status, 
                   metadata, created_at, read_at
            FROM notifications
            WHERE id = ?
        "#;
        
        let row = sqlx::query(query)
            .bind(notification_id.to_string())
            .fetch_one(pool)
            .await?;

        Self::parse_notification_from_row(&row)
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
            "SELECT COUNT(*) as count FROM notifications WHERE user_id = ? {}",
            status_condition
        );
        let total: i64 = sqlx::query_scalar(&count_query)
            .bind(user_id.to_string())
            .fetch_one(pool)
            .await?;

        // 获取通知列表
        let list_query = format!(
            r#"
            SELECT id, user_id, type as "notification_type: NotificationType", 
                   title, content, related_id, status as "status: NotificationStatus", 
                   metadata, created_at, read_at
            FROM notifications
            WHERE user_id = ? {}
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
            status_condition
        );

        let rows = sqlx::query(&list_query)
            .bind(user_id.to_string())
            .bind(page_size)
            .bind(offset)
            .fetch_all(pool)
            .await?;
        
        let mut notifications = Vec::new();
        for row in rows {
            notifications.push(Self::parse_notification_from_row(&row)?);
        }

        Ok((notifications, total))
    }

    /// 获取单个通知
    pub async fn get_notification_by_id(
        pool: &DbPool,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Notification>, sqlx::Error> {
        let query = r#"
            SELECT id, user_id, type as notification_type, 
                   title, content, related_id, status, 
                   metadata, created_at, read_at
            FROM notifications
            WHERE id = ? AND user_id = ? AND status != 'deleted'
        "#;
        
        let row = sqlx::query(query)
            .bind(id.to_string())
            .bind(user_id.to_string())
            .fetch_optional(pool)
            .await?;
        
        match row {
            Some(row) => Ok(Some(Self::parse_notification_from_row(&row)?)),
            None => Ok(None)
        }
    }

    /// 标记通知为已读
    pub async fn mark_as_read(
        pool: &DbPool,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<bool, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE notifications
            SET status = 'read', read_at = ?
            WHERE id = ? AND user_id = ? AND status = 'unread'
            "#
        )
        .bind(Utc::now())
        .bind(id.to_string())
        .bind(user_id.to_string())
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 批量标记为已读
    pub async fn mark_all_as_read(pool: &DbPool, user_id: Uuid) -> Result<u64, sqlx::Error> {
        let result = sqlx::query(
            r#"
            UPDATE notifications
            SET status = 'read', read_at = ?
            WHERE user_id = ? AND status = 'unread'
            "#
        )
        .bind(Utc::now())
        .bind(user_id.to_string())
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
        let result = sqlx::query(
            r#"
            UPDATE notifications
            SET status = 'deleted'
            WHERE id = ? AND user_id = ? AND status != 'deleted'
            "#
        )
        .bind(id.to_string())
        .bind(user_id.to_string())
        .execute(pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    /// 获取用户通知统计
    pub async fn get_notification_stats(
        pool: &DbPool,
        user_id: Uuid,
    ) -> Result<NotificationStats, sqlx::Error> {
        let query = r#"
            SELECT 
                SUM(CASE WHEN status != 'deleted' THEN 1 ELSE 0 END) as total_count,
                SUM(CASE WHEN status = 'unread' THEN 1 ELSE 0 END) as unread_count,
                SUM(CASE WHEN status = 'read' THEN 1 ELSE 0 END) as read_count
            FROM notifications
            WHERE user_id = ?
        "#;
        
        let row = sqlx::query(query)
            .bind(user_id.to_string())
            .fetch_one(pool)
            .await?;
        
        use sqlx::Row;
        Ok(NotificationStats {
            total_count: row.get::<Option<i64>, _>("total_count").unwrap_or(0),
            unread_count: row.get::<Option<i64>, _>("unread_count").unwrap_or(0),
            read_count: row.get::<Option<i64>, _>("read_count").unwrap_or(0),
        })
    }

    /// 获取用户通知设置
    pub async fn get_user_notification_settings(
        pool: &DbPool,
        user_id: Uuid,
    ) -> Result<Vec<NotificationSettings>, sqlx::Error> {
        let query = r#"
            SELECT id, user_id, notification_type, 
                   enabled, email_enabled, sms_enabled, push_enabled, 
                   created_at, updated_at
            FROM notification_settings
            WHERE user_id = ?
            ORDER BY notification_type
        "#;
        
        let rows = sqlx::query(query)
            .bind(user_id.to_string())
            .fetch_all(pool)
            .await?;
        
        let mut settings = Vec::new();
        for row in rows {
            settings.push(Self::parse_notification_settings_from_row(&row)?);
        }

        Ok(settings)
    }

    /// 更新通知设置
    pub async fn update_notification_settings(
        pool: &DbPool,
        user_id: Uuid,
        dto: UpdateNotificationSettingsDto,
    ) -> Result<NotificationSettings, sqlx::Error> {
        // 先检查是否存在该设置
        let exists = sqlx::query(
            r#"
            SELECT COUNT(*) as count
            FROM notification_settings
            WHERE user_id = ? AND notification_type = ?
            "#
        )
        .bind(user_id.to_string())
        .bind(dto.notification_type.to_string())
        .fetch_one(pool)
        .await?;

        use sqlx::Row;
        let count: i64 = exists.get::<i64, _>("count");
        let settings = if count > 0 {
            // 更新现有设置
            sqlx::query(
                r#"
                UPDATE notification_settings
                SET enabled = COALESCE(?, enabled),
                    email_enabled = COALESCE(?, email_enabled),
                    sms_enabled = COALESCE(?, sms_enabled),
                    push_enabled = COALESCE(?, push_enabled),
                    updated_at = CURRENT_TIMESTAMP
                WHERE user_id = ? AND notification_type = ?
                "#
            )
            .bind(dto.enabled)
            .bind(dto.email_enabled)
            .bind(dto.sms_enabled)
            .bind(dto.push_enabled)
            .bind(user_id.to_string())
            .bind(dto.notification_type.to_string())
            .execute(pool)
            .await?;

            let query = r#"
                SELECT id, user_id, notification_type, 
                       enabled, email_enabled, sms_enabled, push_enabled, 
                       created_at, updated_at
                FROM notification_settings
                WHERE user_id = ? AND notification_type = ?
            "#;
            
            let row = sqlx::query(query)
                .bind(user_id.to_string())
                .bind(dto.notification_type.to_string())
                .fetch_one(pool)
                .await?;
                
            Self::parse_notification_settings_from_row(&row)?
        } else {
            // 创建新设置
            let settings_id = Uuid::new_v4();
            
            let result = sqlx::query(
                r#"
                INSERT INTO notification_settings 
                (id, user_id, notification_type, enabled, email_enabled, sms_enabled, push_enabled)
                VALUES (?, ?, ?, ?, ?, ?, ?)
                "#
            )
            .bind(settings_id.to_string())
            .bind(user_id.to_string())
            .bind(dto.notification_type.to_string())
            .bind(dto.enabled.unwrap_or(true))
            .bind(dto.email_enabled.unwrap_or(false))
            .bind(dto.sms_enabled.unwrap_or(false))
            .bind(dto.push_enabled.unwrap_or(true))
            .execute(pool)
            .await?;

            let query = r#"
                SELECT id, user_id, notification_type, 
                       enabled, email_enabled, sms_enabled, push_enabled, 
                       created_at, updated_at
                FROM notification_settings
                WHERE id = ?
            "#;
            
            let row = sqlx::query(query)
                .bind(settings_id.to_string())
                .fetch_one(pool)
                .await?;
                
            Self::parse_notification_settings_from_row(&row)?
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
        sqlx::query(
            r#"
            UPDATE push_tokens
            SET active = false, updated_at = CURRENT_TIMESTAMP
            WHERE user_id = ? AND device_type = ?
            "#
        )
        .bind(user_id.to_string())
        .bind(&dto.device_type)
        .execute(pool)
        .await?;

        // 创建新token
        let token_id = Uuid::new_v4();
        let now = Utc::now();
        
        let result = sqlx::query(
            r#"
            INSERT INTO push_tokens (id, user_id, device_type, token, device_info, active, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, true, ?, ?)
            "#
        )
        .bind(token_id.to_string())
        .bind(user_id.to_string())
        .bind(&dto.device_type)
        .bind(&dto.token)
        .bind(&device_info)
        .bind(&now)
        .bind(&now)
        .execute(pool)
        .await?;

        let query = r#"
            SELECT id, user_id, device_type, token, device_info, active, 
                   created_at, updated_at
            FROM push_tokens
            WHERE id = ?
        "#;
        
        let row = sqlx::query(query)
            .bind(token_id.to_string())
            .fetch_one(pool)
            .await?;
        
        use sqlx::Row;
        let token = PushToken {
            id: Uuid::parse_str(row.get("id")).map_err(|_| sqlx::Error::ColumnDecode {
                index: "id".to_string(),
                source: Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UUID")),
            })?,
            user_id: Uuid::parse_str(row.get("user_id")).map_err(|_| sqlx::Error::ColumnDecode {
                index: "user_id".to_string(),
                source: Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UUID")),
            })?,
            device_type: row.get("device_type"),
            token: row.get("token"),
            device_info: row.get("device_info"),
            active: row.get("active"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        };

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
        let log_id = Uuid::new_v4();
        let now = Utc::now();
        let sent_at = if status == "sent" { Some(now) } else { None };
        
        let result = sqlx::query(
            r#"
            INSERT INTO sms_logs (id, user_id, phone, template_code, template_params, 
                                  status, error_message, provider, created_at, sent_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(log_id.to_string())
        .bind(user_id.map(|id| id.to_string()))
        .bind(&dto.phone)
        .bind(&dto.template_code)
        .bind(&dto.template_params)
        .bind(status)
        .bind(&error_message)
        .bind(&provider)
        .bind(&now)
        .bind(&sent_at)
        .execute(pool)
        .await?;

        let query = r#"
            SELECT id, user_id, phone, template_code, template_params, 
                   status, error_message, provider, created_at, sent_at
            FROM sms_logs
            WHERE id = ?
        "#;
        
        let row = sqlx::query(query)
            .bind(log_id.to_string())
            .fetch_one(pool)
            .await?;
        
        use sqlx::Row;
        let log = SmsLog {
            id: Uuid::parse_str(row.get("id")).map_err(|_| sqlx::Error::ColumnDecode {
                index: "id".to_string(),
                source: Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UUID")),
            })?,
            user_id: row.get::<Option<String>, _>("user_id")
                .map(|s| Uuid::parse_str(&s).ok())
                .flatten(),
            phone: row.get("phone"),
            template_code: row.get("template_code"),
            template_params: row.get("template_params"),
            status: row.get("status"),
            error_message: row.get("error_message"),
            provider: row.get("provider"),
            created_at: row.get("created_at"),
            sent_at: row.get("sent_at"),
        };

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
        let log_id = Uuid::new_v4();
        let now = Utc::now();
        let sent_at = if status == "sent" { Some(now) } else { None };
        
        let result = sqlx::query(
            r#"
            INSERT INTO email_logs (id, user_id, email, subject, body, 
                                    status, error_message, provider, created_at, sent_at)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#
        )
        .bind(log_id.to_string())
        .bind(user_id.map(|id| id.to_string()))
        .bind(&dto.email)
        .bind(&dto.subject)
        .bind(&dto.body)
        .bind(status)
        .bind(&error_message)
        .bind(&provider)
        .bind(&now)
        .bind(&sent_at)
        .execute(pool)
        .await?;

        let query = r#"
            SELECT id, user_id, email, subject, body, 
                   status, error_message, provider, created_at, sent_at
            FROM email_logs
            WHERE id = ?
        "#;
        
        let row = sqlx::query(query)
            .bind(log_id.to_string())
            .fetch_one(pool)
            .await?;
        
        use sqlx::Row;
        let log = EmailLog {
            id: Uuid::parse_str(row.get("id")).map_err(|_| sqlx::Error::ColumnDecode {
                index: "id".to_string(),
                source: Box::new(std::io::Error::new(std::io::ErrorKind::InvalidData, "Invalid UUID")),
            })?,
            user_id: row.get::<Option<String>, _>("user_id")
                .map(|s| Uuid::parse_str(&s).ok())
                .flatten(),
            email: row.get("email"),
            subject: row.get("subject"),
            body: row.get("body"),
            status: row.get("status"),
            error_message: row.get("error_message"),
            provider: row.get("provider"),
            created_at: row.get("created_at"),
            sent_at: row.get("sent_at"),
        };

        Ok(log)
    }

    /// 检查用户是否应该接收某类型通知
    pub async fn should_send_notification(
        pool: &DbPool,
        user_id: Uuid,
        notification_type: &NotificationType,
    ) -> Result<(bool, bool, bool, bool), sqlx::Error> {
        let query = r#"
            SELECT enabled, email_enabled, sms_enabled, push_enabled
            FROM notification_settings
            WHERE user_id = ? AND notification_type = ?
        "#;
        
        let settings = sqlx::query(query)
            .bind(user_id.to_string())
            .bind(notification_type.to_string())
            .fetch_optional(pool)
            .await?;

        match settings {
            Some(row) => {
                use sqlx::Row;
                Ok((
                    row.get("enabled"),
                    row.get("email_enabled"),
                    row.get("sms_enabled"),
                    row.get("push_enabled")
                ))
            },
            None => Ok((true, false, false, true)), // 默认启用通知和推送，禁用邮件和短信
        }
    }
}