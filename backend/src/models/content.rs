use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// Article models
#[derive(Debug, Serialize, Deserialize)]
pub struct Article {
    pub id: Uuid,
    pub title: String,
    pub cover_image: Option<String>,
    pub summary: Option<String>,
    pub content: String,
    pub author_id: Uuid,
    pub author_name: String,
    pub author_type: AuthorType,
    pub category: String,
    pub tags: Option<Vec<String>>,
    pub view_count: u32,
    pub like_count: u32,
    pub status: ContentStatus,
    pub publish_channels: Option<Vec<String>>,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArticleListItem {
    pub id: Uuid,
    pub title: String,
    pub cover_image: Option<String>,
    pub summary: Option<String>,
    pub author_name: String,
    pub category: String,
    pub view_count: u32,
    pub status: ContentStatus,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// Video models
#[derive(Debug, Serialize, Deserialize)]
pub struct Video {
    pub id: Uuid,
    pub title: String,
    pub cover_image: Option<String>,
    pub video_url: String,
    pub duration: Option<u32>,
    pub file_size: Option<u64>,
    pub description: Option<String>,
    pub author_id: Uuid,
    pub author_name: String,
    pub author_type: AuthorType,
    pub category: String,
    pub tags: Option<Vec<String>>,
    pub view_count: u32,
    pub like_count: u32,
    pub status: VideoStatus,
    pub publish_channels: Option<Vec<String>>,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoListItem {
    pub id: Uuid,
    pub title: String,
    pub cover_image: Option<String>,
    pub video_url: String,
    pub duration: Option<u32>,
    pub author_name: String,
    pub category: String,
    pub view_count: u32,
    pub status: VideoStatus,
    pub published_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

// Category model
#[derive(Debug, Serialize, Deserialize)]
pub struct ContentCategory {
    pub id: Uuid,
    pub name: String,
    pub r#type: CategoryType,
    pub sort_order: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// Enums
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type)]
#[sqlx(type_name = "author_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum AuthorType {
    Admin,
    Doctor,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type)]
#[sqlx(type_name = "content_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ContentStatus {
    Draft,
    Published,
    Offline,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type)]
#[sqlx(type_name = "video_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum VideoStatus {
    Draft,
    Processing,
    Published,
    Offline,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type)]
#[sqlx(type_name = "category_type", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum CategoryType {
    Article,
    Video,
    Both,
}

// DTOs for Article
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateArticleDto {
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    #[validate(url)]
    pub cover_image: Option<String>,
    #[validate(length(max = 500))]
    pub summary: Option<String>,
    #[validate(length(min = 1))]
    pub content: String,
    #[validate(length(min = 1, max = 50))]
    pub category: String,
    pub tags: Option<Vec<String>>,
    pub publish_channels: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateArticleDto {
    #[validate(length(min = 1, max = 200))]
    pub title: Option<String>,
    #[validate(url)]
    pub cover_image: Option<String>,
    #[validate(length(max = 500))]
    pub summary: Option<String>,
    #[validate(length(min = 1))]
    pub content: Option<String>,
    #[validate(length(min = 1, max = 50))]
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub publish_channels: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublishArticleDto {
    pub publish_channels: Vec<String>,
}

// DTOs for Video
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateVideoDto {
    #[validate(length(min = 1, max = 200))]
    pub title: String,
    #[validate(url)]
    pub cover_image: Option<String>,
    #[validate(url)]
    pub video_url: String,
    pub duration: Option<u32>,
    pub file_size: Option<u64>,
    pub description: Option<String>,
    #[validate(length(min = 1, max = 50))]
    pub category: String,
    pub tags: Option<Vec<String>>,
    pub publish_channels: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateVideoDto {
    #[validate(length(min = 1, max = 200))]
    pub title: Option<String>,
    #[validate(url)]
    pub cover_image: Option<String>,
    #[validate(url)]
    pub video_url: Option<String>,
    pub duration: Option<u32>,
    pub file_size: Option<u64>,
    pub description: Option<String>,
    #[validate(length(min = 1, max = 50))]
    pub category: Option<String>,
    pub tags: Option<Vec<String>>,
    pub publish_channels: Option<Vec<String>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PublishVideoDto {
    pub publish_channels: Vec<String>,
}

// DTO for Category
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateCategoryDto {
    #[validate(length(min = 1, max = 50))]
    pub name: String,
    pub r#type: CategoryType,
    pub sort_order: Option<i32>,
}