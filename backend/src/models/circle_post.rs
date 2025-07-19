use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CirclePost {
    pub id: Uuid,
    pub author_id: Uuid,
    pub circle_id: Uuid,
    pub title: String,
    pub content: String,
    pub images: Vec<String>,
    pub likes: i64,
    pub comments: i64,
    pub status: PostStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type)]
#[sqlx(type_name = "post_status", rename_all = "lowercase")]
pub enum PostStatus {
    Active,
    Deleted,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateCirclePostDto {
    pub author_id: Uuid,
    pub circle_id: Uuid,
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    #[validate(length(min = 1, max = 1000))]
    pub content: String,
    #[validate(length(max = 9))]
    pub images: Vec<String>,
}
