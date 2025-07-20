use crate::{
    config::storage::{StorageConfig, StorageType},
    models::file_upload::*,
    utils::errors::AppError,
};
use aws_sdk_s3::{
    primitives::ByteStream,
    types::{Delete, ObjectIdentifier},
    Client as S3Client,
};
use chrono::Utc;
use std::path::Path;
use tokio::fs;
use uuid::Uuid;

pub struct FileStorageService;

impl FileStorageService {
    /// Upload file to S3 or OSS
    pub async fn upload_to_cloud(
        s3_client: &S3Client,
        file_path: &str,
        file_data: Vec<u8>,
        content_type: &str,
    ) -> Result<String, AppError> {
        let config = StorageConfig::from_env();
        let byte_stream = ByteStream::from(file_data);

        s3_client
            .put_object()
            .bucket(&config.bucket_name)
            .key(file_path)
            .body(byte_stream)
            .content_type(content_type)
            .send()
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to upload to S3: {}", e)))?;

        // Generate URL based on storage type
        let url = match config.storage_type {
            StorageType::S3 => {
                if let Some(endpoint) = config.endpoint {
                    format!("{}/{}/{}", endpoint, config.bucket_name, file_path)
                } else {
                    format!(
                        "https://{}.s3.{}.amazonaws.com/{}",
                        config.bucket_name, config.region, file_path
                    )
                }
            }
            StorageType::OSS => {
                format!(
                    "https://{}.{}/{}",
                    config.bucket_name,
                    config
                        .endpoint
                        .unwrap_or_else(|| "oss-cn-hangzhou.aliyuncs.com".to_string()),
                    file_path
                )
            }
        };

        Ok(url)
    }

    /// Delete file from S3 or OSS
    pub async fn delete_from_cloud(s3_client: &S3Client, file_path: &str) -> Result<(), AppError> {
        let config = StorageConfig::from_env();

        s3_client
            .delete_object()
            .bucket(&config.bucket_name)
            .key(file_path)
            .send()
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Failed to delete from S3: {}", e))
            })?;

        Ok(())
    }

    /// Batch delete files from S3 or OSS
    pub async fn batch_delete_from_cloud(
        s3_client: &S3Client,
        file_paths: Vec<String>,
    ) -> Result<(), AppError> {
        let config = StorageConfig::from_env();

        let objects: Vec<ObjectIdentifier> = file_paths
            .into_iter()
            .map(|path| ObjectIdentifier::builder().key(path).build().unwrap())
            .collect();

        let delete = Delete::builder()
            .set_objects(Some(objects))
            .build()
            .map_err(|e| {
                AppError::InternalServerError(format!("Failed to build delete request: {}", e))
            })?;

        s3_client
            .delete_objects()
            .bucket(&config.bucket_name)
            .delete(delete)
            .send()
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Failed to batch delete from S3: {}", e))
            })?;

        Ok(())
    }

    /// Generate pre-signed URL for direct upload
    pub async fn generate_presigned_upload_url(
        s3_client: &S3Client,
        file_path: &str,
        content_type: &str,
        expires_in_seconds: u64,
    ) -> Result<String, AppError> {
        let config = StorageConfig::from_env();

        let presigned_request = s3_client
            .put_object()
            .bucket(&config.bucket_name)
            .key(file_path)
            .content_type(content_type)
            .presigned(
                aws_sdk_s3::presigning::PresigningConfig::expires_in(
                    std::time::Duration::from_secs(expires_in_seconds),
                )
                .map_err(|e| {
                    AppError::InternalServerError(format!(
                        "Failed to create presigning config: {}",
                        e
                    ))
                })?,
            )
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Failed to generate presigned URL: {}", e))
            })?;

        Ok(presigned_request.uri().to_string())
    }

    /// Generate pre-signed URL for download
    pub async fn generate_presigned_download_url(
        s3_client: &S3Client,
        file_path: &str,
        expires_in_seconds: u64,
    ) -> Result<String, AppError> {
        let config = StorageConfig::from_env();

        let presigned_request = s3_client
            .get_object()
            .bucket(&config.bucket_name)
            .key(file_path)
            .presigned(
                aws_sdk_s3::presigning::PresigningConfig::expires_in(
                    std::time::Duration::from_secs(expires_in_seconds),
                )
                .map_err(|e| {
                    AppError::InternalServerError(format!(
                        "Failed to create presigning config: {}",
                        e
                    ))
                })?,
            )
            .await
            .map_err(|e| {
                AppError::InternalServerError(format!("Failed to generate presigned URL: {}", e))
            })?;

        Ok(presigned_request.uri().to_string())
    }

    /// Upload file to local filesystem (fallback)
    pub async fn upload_to_local(file_path: &str, file_data: Vec<u8>) -> Result<String, AppError> {
        let upload_dir = "uploads";
        let full_path = format!("{}/{}", upload_dir, file_path);

        // Create directory if it doesn't exist
        if let Some(parent) = Path::new(&full_path).parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                AppError::InternalServerError(format!("Failed to create directory: {}", e))
            })?;
        }

        // Write file
        fs::write(&full_path, file_data)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to write file: {}", e)))?;

        Ok(format!("/uploads/{}", file_path))
    }

    /// Delete file from local filesystem
    pub async fn delete_from_local(file_path: &str) -> Result<(), AppError> {
        let full_path = format!("uploads/{}", file_path);

        if Path::new(&full_path).exists() {
            fs::remove_file(&full_path).await.map_err(|e| {
                AppError::InternalServerError(format!("Failed to delete file: {}", e))
            })?;
        }

        Ok(())
    }

    /// Generate unique file path
    pub fn generate_file_path(file_type: &FileType, original_filename: &str) -> String {
        let ext = Path::new(original_filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("");

        let date = Utc::now().format("%Y/%m/%d");
        let unique_name = format!("{}.{}", Uuid::new_v4(), ext);

        match file_type {
            FileType::Image => format!("images/{}/{}", date, unique_name),
            FileType::Document => format!("documents/{}/{}", date, unique_name),
            FileType::Video => format!("videos/{}/{}", date, unique_name),
            FileType::Audio => format!("audio/{}/{}", date, unique_name),
            FileType::Other => format!("others/{}/{}", date, unique_name),
        }
    }

    /// Validate file size
    pub fn validate_file_size(size: usize, file_type: &FileType) -> Result<(), AppError> {
        let max_size = match file_type {
            FileType::Image => 10 * 1024 * 1024,    // 10MB
            FileType::Document => 50 * 1024 * 1024, // 50MB
            FileType::Video => 500 * 1024 * 1024,   // 500MB
            FileType::Audio => 100 * 1024 * 1024,   // 100MB
            FileType::Other => 100 * 1024 * 1024,   // 100MB
        };

        if size > max_size {
            return Err(AppError::BadRequest(format!(
                "File size exceeds maximum allowed size of {} MB",
                max_size / 1024 / 1024
            )));
        }

        Ok(())
    }

    /// Validate file extension
    pub fn validate_file_extension(filename: &str, file_type: &FileType) -> Result<(), AppError> {
        let ext = Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        let allowed_extensions = match file_type {
            FileType::Image => vec!["jpg", "jpeg", "png", "gif", "webp", "bmp"],
            FileType::Document => vec!["pdf", "doc", "docx", "xls", "xlsx", "ppt", "pptx", "txt"],
            FileType::Video => vec!["mp4", "avi", "mov", "wmv", "flv", "mkv"],
            FileType::Audio => vec!["mp3", "wav", "aac", "ogg", "m4a"],
            FileType::Other => vec![], // Allow any extension for "Other"
        };

        if file_type != &FileType::Other && !allowed_extensions.contains(&ext.as_str()) {
            return Err(AppError::BadRequest(format!(
                "File extension '{}' is not allowed for file type {:?}",
                ext, file_type
            )));
        }

        Ok(())
    }

    /// Get content type from file extension
    pub fn get_content_type(filename: &str) -> &'static str {
        let ext = Path::new(filename)
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        match ext.as_str() {
            // Images
            "jpg" | "jpeg" => "image/jpeg",
            "png" => "image/png",
            "gif" => "image/gif",
            "webp" => "image/webp",
            "bmp" => "image/bmp",

            // Documents
            "pdf" => "application/pdf",
            "doc" => "application/msword",
            "docx" => "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
            "xls" => "application/vnd.ms-excel",
            "xlsx" => "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
            "ppt" => "application/vnd.ms-powerpoint",
            "pptx" => "application/vnd.openxmlformats-officedocument.presentationml.presentation",
            "txt" => "text/plain",

            // Videos
            "mp4" => "video/mp4",
            "avi" => "video/x-msvideo",
            "mov" => "video/quicktime",
            "wmv" => "video/x-ms-wmv",
            "flv" => "video/x-flv",
            "mkv" => "video/x-matroska",

            // Audio
            "mp3" => "audio/mpeg",
            "wav" => "audio/wav",
            "aac" => "audio/aac",
            "ogg" => "audio/ogg",
            "m4a" => "audio/mp4",

            _ => "application/octet-stream",
        }
    }
}
