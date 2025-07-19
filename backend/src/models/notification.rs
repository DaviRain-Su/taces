use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;
use std::fmt;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type)]
#[sqlx(type_name = "notification_type", rename_all = "snake_case")]
pub enum NotificationType {
    AppointmentReminder,
    AppointmentConfirmed,
    AppointmentCancelled,
    PrescriptionReady,
    DoctorReply,
    SystemAnnouncement,
    ReviewReply,
    LiveStreamReminder,
    GroupMessage,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type)]
#[sqlx(type_name = "notification_status", rename_all = "lowercase")]
pub enum NotificationStatus {
    Unread,
    Read,
    Deleted,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Notification {
    pub id: Uuid,
    pub user_id: Uuid,
    #[sqlx(rename = "type")]
    pub notification_type: NotificationType,
    pub title: String,
    pub content: String,
    pub related_id: Option<Uuid>,
    pub status: NotificationStatus,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateNotificationDto {
    pub user_id: Uuid,
    #[serde(rename = "type")]
    pub notification_type: NotificationType,
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    #[validate(length(min = 1))]
    pub content: String,
    pub related_id: Option<Uuid>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationResponse {
    pub id: Uuid,
    pub user_id: Uuid,
    #[serde(rename = "type")]
    pub notification_type: NotificationType,
    pub title: String,
    pub content: String,
    pub related_id: Option<Uuid>,
    pub status: NotificationStatus,
    pub metadata: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub read_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct NotificationSettings {
    pub id: Uuid,
    pub user_id: Uuid,
    pub notification_type: NotificationType,
    pub enabled: bool,
    pub email_enabled: bool,
    pub sms_enabled: bool,
    pub push_enabled: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateNotificationSettingsDto {
    pub notification_type: NotificationType,
    pub enabled: Option<bool>,
    pub email_enabled: Option<bool>,
    pub sms_enabled: Option<bool>,
    pub push_enabled: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SmsLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub phone: String,
    pub template_code: String,
    pub template_params: serde_json::Value,
    pub status: String,
    pub error_message: Option<String>,
    pub provider: Option<String>,
    pub created_at: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct SendSmsDto {
    #[validate(length(min = 10, max = 20))]
    pub phone: String,
    pub template_code: String,
    pub template_params: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct EmailLog {
    pub id: Uuid,
    pub user_id: Option<Uuid>,
    pub email: String,
    pub subject: String,
    pub body: String,
    pub status: String,
    pub error_message: Option<String>,
    pub provider: Option<String>,
    pub created_at: DateTime<Utc>,
    pub sent_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct SendEmailDto {
    #[validate(email)]
    pub email: String,
    #[validate(length(min = 1, max = 255))]
    pub subject: String,
    #[validate(length(min = 1))]
    pub body: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PushToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub device_type: String,
    pub token: String,
    pub device_info: serde_json::Value,
    pub active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct RegisterPushTokenDto {
    #[validate(length(min = 1, max = 20))]
    pub device_type: String,
    #[validate(length(min = 1))]
    pub token: String,
    pub device_info: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct NotificationStats {
    pub total_count: i64,
    pub unread_count: i64,
    pub read_count: i64,
}

impl From<Notification> for NotificationResponse {
    fn from(notification: Notification) -> Self {
        NotificationResponse {
            id: notification.id,
            user_id: notification.user_id,
            notification_type: notification.notification_type,
            title: notification.title,
            content: notification.content,
            related_id: notification.related_id,
            status: notification.status,
            metadata: notification.metadata,
            created_at: notification.created_at,
            read_at: notification.read_at,
        }
    }
}

impl fmt::Display for NotificationType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotificationType::AppointmentReminder => write!(f, "appointment_reminder"),
            NotificationType::AppointmentConfirmed => write!(f, "appointment_confirmed"),
            NotificationType::AppointmentCancelled => write!(f, "appointment_cancelled"),
            NotificationType::PrescriptionReady => write!(f, "prescription_ready"),
            NotificationType::DoctorReply => write!(f, "doctor_reply"),
            NotificationType::SystemAnnouncement => write!(f, "system_announcement"),
            NotificationType::ReviewReply => write!(f, "review_reply"),
            NotificationType::LiveStreamReminder => write!(f, "live_stream_reminder"),
            NotificationType::GroupMessage => write!(f, "group_message"),
        }
    }
}

impl fmt::Display for NotificationStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NotificationStatus::Unread => write!(f, "unread"),
            NotificationStatus::Read => write!(f, "read"),
            NotificationStatus::Deleted => write!(f, "deleted"),
        }
    }
}