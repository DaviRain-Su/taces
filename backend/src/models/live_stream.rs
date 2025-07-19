use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct LiveStream {
    pub id: Uuid,
    pub title: String,
    pub host_id: Uuid,
    pub host_name: String,
    pub scheduled_time: DateTime<Utc>,
    pub stream_url: Option<String>,
    pub qr_code: Option<String>,
    pub status: LiveStreamStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LiveStreamListItem {
    pub id: Uuid,
    pub title: String,
    pub host_name: String,
    pub scheduled_time: DateTime<Utc>,
    pub status: LiveStreamStatus,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type)]
#[sqlx(type_name = "live_stream_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum LiveStreamStatus {
    Scheduled,
    Live,
    Ended,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateLiveStreamDto {
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    pub scheduled_time: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateLiveStreamDto {
    #[validate(length(min = 1, max = 200))]
    pub title: Option<String>,
    pub scheduled_time: Option<DateTime<Utc>>,
    pub stream_url: Option<Option<String>>,
    pub qr_code: Option<Option<String>>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct StartLiveStreamDto {
    #[validate(url)]
    pub stream_url: String,
    pub qr_code: Option<String>,
}

impl Default for LiveStreamStatus {
    fn default() -> Self {
        LiveStreamStatus::Scheduled
    }
}
