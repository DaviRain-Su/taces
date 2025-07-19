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
#[serde(rename_all = "lowercase")]
pub enum PostStatus {
    Active,
    Deleted,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateCirclePostDto {
    pub circle_id: Uuid,
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    #[validate(length(min = 1, max = 1000))]
    pub content: String,
    #[validate(length(max = 9))]
    pub images: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateCirclePostDto {
    #[validate(length(min = 1, max = 200))]
    pub title: Option<String>,
    #[validate(length(min = 1, max = 1000))]
    pub content: Option<String>,
    #[validate(length(max = 9))]
    pub images: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CirclePostWithAuthor {
    pub id: Uuid,
    pub author_id: Uuid,
    pub author_name: String,
    pub circle_id: Uuid,
    pub circle_name: String,
    pub title: String,
    pub content: String,
    pub images: Vec<String>,
    pub likes: i64,
    pub comments: i64,
    pub is_liked: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PostLike {
    pub id: Uuid,
    pub post_id: Uuid,
    pub user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PostComment {
    pub id: Uuid,
    pub post_id: Uuid,
    pub user_id: Uuid,
    pub content: String,
    pub is_deleted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PostCommentWithAuthor {
    pub id: Uuid,
    pub post_id: Uuid,
    pub user_id: Uuid,
    pub user_name: String,
    pub content: String,
    pub is_deleted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateCommentDto {
    #[validate(length(min = 1, max = 500))]
    pub content: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SensitiveWord {
    pub id: Uuid,
    pub word: String,
    pub category: Option<String>,
    pub severity: String,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}
