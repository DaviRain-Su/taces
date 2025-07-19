use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type)]
#[sqlx(type_name = "file_type", rename_all = "lowercase")]
pub enum FileType {
    Image,
    Video,
    Document,
    Audio,
    Other,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type)]
#[sqlx(type_name = "upload_status", rename_all = "lowercase")]
pub enum UploadStatus {
    Uploading,
    Completed,
    Failed,
    Deleted,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct FileUpload {
    pub id: Uuid,
    pub user_id: Uuid,
    pub file_type: FileType,
    pub file_name: String,
    pub file_path: String,
    pub file_url: String,
    pub file_size: i64,
    pub mime_type: Option<String>,
    pub related_type: Option<String>,
    pub related_id: Option<Uuid>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub thumbnail_url: Option<String>,
    pub is_public: bool,
    pub status: UploadStatus,
    pub error_message: Option<String>,
    pub bucket_name: Option<String>,
    pub object_key: Option<String>,
    pub etag: Option<String>,
    pub uploaded_at: DateTime<Utc>,
    pub expires_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateFileUploadDto {
    #[validate(length(min = 1, max = 255))]
    pub file_name: String,
    pub file_type: FileType,
    pub file_size: i64,
    #[validate(length(max = 100))]
    pub mime_type: Option<String>,
    #[validate(length(max = 50))]
    pub related_type: Option<String>,
    pub related_id: Option<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CompleteUploadDto {
    #[validate(length(min = 1, max = 500))]
    pub file_path: String,
    #[validate(url)]
    pub file_url: String,
    #[validate(length(max = 100))]
    pub bucket_name: Option<String>,
    #[validate(length(max = 500))]
    pub object_key: Option<String>,
    #[validate(length(max = 100))]
    pub etag: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    #[validate(url)]
    pub thumbnail_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UploadUrlResponse {
    pub upload_id: Uuid,
    pub upload_url: String,
    pub upload_method: String, // PUT or POST
    pub upload_headers: Option<serde_json::Value>,
    pub expires_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileListQuery {
    pub user_id: Option<Uuid>,
    pub file_type: Option<FileType>,
    pub related_type: Option<String>,
    pub related_id: Option<Uuid>,
    pub status: Option<UploadStatus>,
    pub is_public: Option<bool>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileListResponse {
    pub files: Vec<FileUpload>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
    pub total_size: i64, // Total size of all files in bytes
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FileStorageStats {
    pub total_files: i64,
    pub total_size: i64,
    pub by_type: Vec<TypeStats>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TypeStats {
    pub file_type: FileType,
    pub count: i64,
    pub total_size: i64,
}

// Configuration DTOs
#[derive(Debug, Serialize, Deserialize)]
pub struct UploadConfig {
    pub max_file_size: i64,
    pub allowed_mime_types: Vec<String>,
    pub storage_backend: String, // "oss", "s3", "local"
    pub cdn_base_url: Option<String>,
    pub enable_compression: bool,
    pub enable_thumbnail: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageConfig {
    pub max_width: i32,
    pub max_height: i32,
    pub thumbnail_width: i32,
    pub thumbnail_height: i32,
    pub compression_quality: u8,
    pub allowed_formats: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct VideoConfig {
    pub max_duration: i32, // seconds
    pub max_file_size: i64,
    pub allowed_formats: Vec<String>,
    pub enable_transcoding: bool,
}

// System Configuration
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type)]
#[sqlx(type_name = "value_type", rename_all = "lowercase")]
pub enum ValueType {
    String,
    Number,
    Boolean,
    Json,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct SystemConfig {
    pub id: Uuid,
    pub category: String,
    pub config_key: String,
    pub config_value: String,
    pub value_type: ValueType,
    pub description: Option<String>,
    pub is_encrypted: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateSystemConfigDto {
    #[validate(length(min = 1))]
    pub config_value: String,
    #[validate(length(max = 500))]
    pub description: Option<String>,
}