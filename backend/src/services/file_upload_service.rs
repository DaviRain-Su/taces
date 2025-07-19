use crate::config::database::DbPool;
use crate::models::file_upload::*;
use crate::utils::errors::AppError;
use chrono::{Duration, Utc};
use sqlx::{MySql, Transaction, Row};
use uuid::Uuid;
use std::collections::HashMap;

pub struct FileUploadService;

impl FileUploadService {
    fn parse_file_upload_from_row(row: &sqlx::mysql::MySqlRow) -> Result<FileUpload, AppError> {
        Ok(FileUpload {
            id: Uuid::parse_str(row.get("id")).map_err(|e| AppError::DatabaseError(format!("Invalid UUID: {}", e)))?,
            user_id: Uuid::parse_str(row.get("user_id")).map_err(|e| AppError::DatabaseError(format!("Invalid UUID: {}", e)))?,
            file_type: row.get("file_type"),
            file_name: row.get("file_name"),
            file_path: row.get("file_path"),
            file_url: row.get("file_url"),
            file_size: row.get("file_size"),
            mime_type: row.get("mime_type"),
            related_type: row.get("related_type"),
            related_id: row.get::<Option<String>, _>("related_id")
                .map(|s| Uuid::parse_str(&s).ok())
                .flatten(),
            width: row.get("width"),
            height: row.get("height"),
            thumbnail_url: row.get("thumbnail_url"),
            is_public: row.get("is_public"),
            status: row.get("status"),
            error_message: row.get("error_message"),
            bucket_name: row.get("bucket_name"),
            object_key: row.get("object_key"),
            etag: row.get("etag"),
            uploaded_at: row.get("uploaded_at"),
            expires_at: row.get("expires_at"),
            deleted_at: row.get("deleted_at"),
        })
    }
    // File Upload Management
    pub async fn create_upload(
        db: &DbPool,
        user_id: Uuid,
        dto: CreateFileUploadDto,
    ) -> Result<UploadUrlResponse, AppError> {
        // Validate file type and size
        Self::validate_upload(&dto).await?;

        let upload_id = Uuid::new_v4();
        let now = Utc::now();
        let expires_at = now + Duration::minutes(30);

        // Generate upload URL and path
        let (file_path, upload_url, upload_method, upload_headers) = 
            Self::generate_upload_url(&upload_id, &dto).await?;

        let query = r#"
            INSERT INTO file_uploads (
                id, user_id, file_type, file_name, file_path,
                file_url, file_size, mime_type, related_type, related_id,
                status, uploaded_at
            ) VALUES (?, ?, ?, ?, ?, '', ?, ?, ?, ?, 'uploading', ?)
        "#;

        sqlx::query(query)
            .bind(upload_id.to_string())
            .bind(user_id.to_string())
            .bind(&dto.file_type)
            .bind(&dto.file_name)
            .bind(&file_path)
            .bind(&dto.file_size)
            .bind(&dto.mime_type)
            .bind(&dto.related_type)
            .bind(dto.related_id.map(|id| id.to_string()))
            .bind(&now)
            .execute(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(UploadUrlResponse {
            upload_id,
            upload_url,
            upload_method,
            upload_headers,
            expires_at,
        })
    }

    pub async fn complete_upload(
        db: &DbPool,
        upload_id: Uuid,
        user_id: Uuid,
        dto: CompleteUploadDto,
    ) -> Result<FileUpload, AppError> {
        let mut tx = db.begin().await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Verify ownership
        let file = Self::get_file_tx(&mut tx, upload_id).await?;
        if file.user_id != user_id {
            return Err(AppError::Forbidden);
        }

        if file.status != UploadStatus::Uploading {
            return Err(AppError::BadRequest("文件已完成上传".to_string()));
        }

        let query = r#"
            UPDATE file_uploads
            SET file_url = ?, bucket_name = ?, object_key = ?,
                etag = ?, width = ?, height = ?, thumbnail_url = ?,
                status = 'completed', updated_at = ?
            WHERE id = ?
        "#;

        sqlx::query(query)
            .bind(&dto.file_url)
            .bind(&dto.bucket_name)
            .bind(&dto.object_key)
            .bind(&dto.etag)
            .bind(&dto.width)
            .bind(&dto.height)
            .bind(&dto.thumbnail_url)
            .bind(&Utc::now())
            .bind(upload_id.to_string())
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        tx.commit().await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Self::get_file(db, upload_id).await
    }

    pub async fn get_file(
        db: &DbPool,
        file_id: Uuid,
    ) -> Result<FileUpload, AppError> {
        let query = r#"
            SELECT * FROM file_uploads WHERE id = ?
        "#;

        let row = sqlx::query(query)
            .bind(file_id.to_string())
            .fetch_one(db)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => AppError::NotFound("文件不存在".to_string()),
                _ => AppError::DatabaseError(e.to_string()),
            })?;
        
        Self::parse_file_upload_from_row(&row)
    }

    async fn get_file_tx(
        tx: &mut Transaction<'_, MySql>,
        file_id: Uuid,
    ) -> Result<FileUpload, AppError> {
        let query = r#"
            SELECT * FROM file_uploads WHERE id = ?
        "#;

        let row = sqlx::query(query)
            .bind(file_id.to_string())
            .fetch_one(&mut **tx)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => AppError::NotFound("文件不存在".to_string()),
                _ => AppError::DatabaseError(e.to_string()),
            })?;
        
        Self::parse_file_upload_from_row(&row)
    }

    pub async fn list_files(
        db: &DbPool,
        query_params: FileListQuery,
    ) -> Result<FileListResponse, AppError> {
        let page = query_params.page.unwrap_or(1).max(1);
        let page_size = query_params.page_size.unwrap_or(20).min(100);
        let offset = (page - 1) * page_size;

        let mut query = String::from(
            "SELECT * FROM file_uploads WHERE deleted_at IS NULL"
        );
        let mut count_query = String::from(
            "SELECT COUNT(*) as count, SUM(file_size) as total_size FROM file_uploads WHERE deleted_at IS NULL"
        );
        let mut bindings: Vec<Box<dyn sqlx::Encode<'_, MySql> + Send + Sync>> = vec![];

        // Build WHERE clauses
        if let Some(user_id) = &query_params.user_id {
            query.push_str(" AND user_id = ?");
            count_query.push_str(" AND user_id = ?");
            bindings.push(Box::new(user_id.to_string()));
        }

        if let Some(file_type) = &query_params.file_type {
            query.push_str(" AND file_type = ?");
            count_query.push_str(" AND file_type = ?");
            bindings.push(Box::new(file_type.clone()));
        }

        if let Some(related_type) = &query_params.related_type {
            query.push_str(" AND related_type = ?");
            count_query.push_str(" AND related_type = ?");
            bindings.push(Box::new(related_type.clone()));
        }

        if let Some(related_id) = &query_params.related_id {
            query.push_str(" AND related_id = ?");
            count_query.push_str(" AND related_id = ?");
            bindings.push(Box::new(related_id.to_string()));
        }

        if let Some(status) = &query_params.status {
            query.push_str(" AND status = ?");
            count_query.push_str(" AND status = ?");
            bindings.push(Box::new(status.clone()));
        }

        if let Some(is_public) = &query_params.is_public {
            query.push_str(" AND is_public = ?");
            count_query.push_str(" AND is_public = ?");
            bindings.push(Box::new(is_public.clone()));
        }

        // Get total count and size
        let mut count_builder = sqlx::query(&count_query);
        for binding in &bindings {
            count_builder = count_builder.bind(binding.as_ref());
        }

        let count_row = count_builder
            .fetch_one(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let total: i64 = count_row.try_get("count").unwrap_or(0);
        let total_size: i64 = count_row.try_get("total_size").unwrap_or(0);

        // Get files
        query.push_str(" ORDER BY uploaded_at DESC LIMIT ? OFFSET ?");

        let mut query_builder = sqlx::query(&query);
        for binding in &bindings {
            query_builder = query_builder.bind(binding.as_ref());
        }

        let rows = query_builder
            .bind(page_size)
            .bind(offset)
            .fetch_all(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        
        let mut files = Vec::new();
        for row in rows {
            files.push(Self::parse_file_upload_from_row(&row)?);
        }

        Ok(FileListResponse {
            files,
            total,
            page,
            page_size,
            total_size,
        })
    }

    pub async fn delete_file(
        db: &DbPool,
        file_id: Uuid,
        user_id: Uuid,
        is_admin: bool,
    ) -> Result<(), AppError> {
        let file = Self::get_file(db, file_id).await?;

        // Check permission
        if !is_admin && file.user_id != user_id {
            return Err(AppError::Forbidden);
        }

        let query = r#"
            UPDATE file_uploads
            SET status = 'deleted', deleted_at = ?
            WHERE id = ?
        "#;

        sqlx::query(query)
            .bind(&Utc::now())
            .bind(file_id.to_string())
            .execute(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // TODO: Schedule actual file deletion from OSS

        Ok(())
    }

    pub async fn get_file_stats(
        db: &DbPool,
        user_id: Option<Uuid>,
    ) -> Result<FileStorageStats, AppError> {
        let mut base_query = String::from("FROM file_uploads WHERE deleted_at IS NULL");
        let mut bindings: Vec<Box<dyn sqlx::Encode<'_, MySql> + Send + Sync>> = vec![];

        if let Some(uid) = user_id {
            base_query.push_str(" AND user_id = ?");
            bindings.push(Box::new(uid.to_string()));
        }

        // Get total stats
        let total_query = format!(
            "SELECT COUNT(*) as count, SUM(file_size) as size {}",
            base_query
        );

        let mut query_builder = sqlx::query(&total_query);
        for binding in &bindings {
            query_builder = query_builder.bind(binding.as_ref());
        }

        let total_row = query_builder
            .fetch_one(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let total_files: i64 = total_row.try_get("count").unwrap_or(0);
        let total_size: i64 = total_row.try_get("size").unwrap_or(0);

        // Get stats by type
        let type_query = format!(
            "SELECT file_type, COUNT(*) as count, SUM(file_size) as size {} GROUP BY file_type",
            base_query
        );

        let mut query_builder = sqlx::query(&type_query);
        for binding in &bindings {
            query_builder = query_builder.bind(binding.as_ref());
        }

        let type_rows = query_builder
            .fetch_all(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut by_type = Vec::new();
        for row in type_rows {
            by_type.push(TypeStats {
                file_type: row.try_get("file_type")?,
                count: row.try_get("count").unwrap_or(0),
                total_size: row.try_get("size").unwrap_or(0),
            });
        }

        Ok(FileStorageStats {
            total_files,
            total_size,
            by_type,
        })
    }

    // System Configuration
    pub async fn get_upload_config(
        db: &DbPool,
    ) -> Result<UploadConfig, AppError> {
        let configs = Self::get_system_configs(db, "file_upload").await?;

        Ok(UploadConfig {
            max_file_size: configs.get("max_file_size")
                .and_then(|v| v.parse().ok())
                .unwrap_or(104857600), // 100MB default
            allowed_mime_types: configs.get("allowed_mime_types")
                .and_then(|v| serde_json::from_str(v).ok())
                .unwrap_or_else(|| vec![]),
            storage_backend: configs.get("storage_backend")
                .unwrap_or(&"oss".to_string())
                .clone(),
            cdn_base_url: configs.get("cdn_base_url").cloned(),
            enable_compression: configs.get("enable_compression")
                .and_then(|v| v.parse().ok())
                .unwrap_or(true),
            enable_thumbnail: configs.get("enable_thumbnail")
                .and_then(|v| v.parse().ok())
                .unwrap_or(true),
        })
    }

    pub async fn get_image_config(
        db: &DbPool,
    ) -> Result<ImageConfig, AppError> {
        let configs = Self::get_system_configs(db, "file_upload").await?;

        Ok(ImageConfig {
            max_width: configs.get("max_image_width")
                .and_then(|v| v.parse().ok())
                .unwrap_or(4096),
            max_height: configs.get("max_image_height")
                .and_then(|v| v.parse().ok())
                .unwrap_or(4096),
            thumbnail_width: configs.get("thumbnail_width")
                .and_then(|v| v.parse().ok())
                .unwrap_or(200),
            thumbnail_height: configs.get("thumbnail_height")
                .and_then(|v| v.parse().ok())
                .unwrap_or(200),
            compression_quality: configs.get("image_compression_quality")
                .and_then(|v| v.parse().ok())
                .unwrap_or(85),
            allowed_formats: configs.get("allowed_image_types")
                .and_then(|v| serde_json::from_str(v).ok())
                .unwrap_or_else(|| vec!["jpg".to_string(), "jpeg".to_string(), 
                    "png".to_string(), "gif".to_string(), "webp".to_string()]),
        })
    }

    pub async fn get_video_config(
        db: &DbPool,
    ) -> Result<VideoConfig, AppError> {
        let configs = Self::get_system_configs(db, "file_upload").await?;

        Ok(VideoConfig {
            max_duration: configs.get("max_video_duration")
                .and_then(|v| v.parse().ok())
                .unwrap_or(3600), // 1 hour default
            max_file_size: configs.get("max_video_size")
                .and_then(|v| v.parse().ok())
                .unwrap_or(104857600), // 100MB default
            allowed_formats: configs.get("allowed_video_types")
                .and_then(|v| serde_json::from_str(v).ok())
                .unwrap_or_else(|| vec!["mp4".to_string(), "webm".to_string(), 
                    "mov".to_string()]),
            enable_transcoding: configs.get("enable_video_transcoding")
                .and_then(|v| v.parse().ok())
                .unwrap_or(false),
        })
    }

    pub async fn update_system_config(
        db: &DbPool,
        category: &str,
        key: &str,
        dto: UpdateSystemConfigDto,
    ) -> Result<SystemConfig, AppError> {
        let query = r#"
            UPDATE system_configs
            SET config_value = ?, description = ?, updated_at = ?
            WHERE category = ? AND config_key = ?
        "#;

        let result = sqlx::query(query)
            .bind(&dto.config_value)
            .bind(&dto.description)
            .bind(&Utc::now())
            .bind(category)
            .bind(key)
            .execute(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AppError::NotFound("配置项不存在".to_string()));
        }

        Self::get_system_config(db, category, key).await
    }

    // Helper methods
    async fn validate_upload(dto: &CreateFileUploadDto) -> Result<(), AppError> {
        // Validate file size based on type
        let max_size = match &dto.file_type {
            FileType::Image => 10 * 1024 * 1024,      // 10MB
            FileType::Video => 100 * 1024 * 1024,     // 100MB
            FileType::Document => 20 * 1024 * 1024,   // 20MB
            FileType::Audio => 50 * 1024 * 1024,      // 50MB
            FileType::Other => 10 * 1024 * 1024,      // 10MB
        };

        if dto.file_size > max_size {
            return Err(AppError::BadRequest(format!(
                "文件大小超过限制: 最大 {} MB",
                max_size / 1024 / 1024
            )));
        }

        // Validate MIME type
        if let Some(mime_type) = &dto.mime_type {
            let allowed_types = match &dto.file_type {
                FileType::Image => vec!["image/jpeg", "image/png", "image/gif", "image/webp"],
                FileType::Video => vec!["video/mp4", "video/webm", "video/quicktime"],
                FileType::Document => vec!["application/pdf", "application/msword", 
                    "application/vnd.openxmlformats-officedocument.wordprocessingml.document"],
                FileType::Audio => vec!["audio/mpeg", "audio/wav", "audio/ogg"],
                FileType::Other => vec![], // Allow any for other
            };

            if !allowed_types.is_empty() && !allowed_types.contains(&mime_type.as_str()) {
                return Err(AppError::BadRequest("不支持的文件类型".to_string()));
            }
        }

        Ok(())
    }

    async fn generate_upload_url(
        upload_id: &Uuid,
        dto: &CreateFileUploadDto,
    ) -> Result<(String, String, String, Option<serde_json::Value>), AppError> {
        // Generate file path
        let date = Utc::now();
        let extension = dto.file_name.split('.').last().unwrap_or("bin");
        let file_path = format!(
            "{}/{}/{}/{}_{}.{}",
            dto.file_type.to_string().to_lowercase(),
            date.format("%Y"),
            date.format("%m"),
            upload_id,
            date.timestamp(),
            extension
        );

        // TODO: Integrate with actual OSS/S3 service to generate presigned URL
        // For now, return a mock URL
        let upload_url = format!("https://oss.example.com/upload/{}", file_path);
        let upload_method = "PUT".to_string();
        let upload_headers = Some(serde_json::json!({
            "Content-Type": dto.mime_type.as_ref().unwrap_or(&"application/octet-stream".to_string()),
            "x-oss-object-acl": if dto.related_type.is_some() { "private" } else { "public-read" }
        }));

        Ok((file_path, upload_url, upload_method, upload_headers))
    }

    async fn get_system_configs(
        db: &DbPool,
        category: &str,
    ) -> Result<HashMap<String, String>, AppError> {
        let query = r#"
            SELECT config_key, config_value
            FROM system_configs
            WHERE category = ?
        "#;

        let rows = sqlx::query(query)
            .bind(category)
            .fetch_all(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut configs = HashMap::new();
        for row in rows {
            let key: String = row.try_get("config_key")?;
            let value: String = row.try_get("config_value")?;
            configs.insert(key, value);
        }

        Ok(configs)
    }

    async fn get_system_config(
        db: &DbPool,
        category: &str,
        key: &str,
    ) -> Result<SystemConfig, AppError> {
        let query = r#"
            SELECT * FROM system_configs
            WHERE category = ? AND config_key = ?
        "#;

        sqlx::query_as::<_, SystemConfig>(query)
            .bind(category)
            .bind(key)
            .fetch_one(db)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => AppError::NotFound("配置项不存在".to_string()),
                _ => AppError::DatabaseError(e.to_string()),
            })
    }

    pub async fn clean_expired_uploads(db: &DbPool) -> Result<u64, AppError> {
        let query = r#"
            UPDATE file_uploads
            SET status = 'failed', error_message = '上传超时'
            WHERE status = 'uploading' 
            AND uploaded_at < DATE_SUB(NOW(), INTERVAL 1 HOUR)
        "#;

        let result = sqlx::query(query)
            .execute(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected())
    }

    pub async fn clean_deleted_files(db: &DbPool) -> Result<u64, AppError> {
        // Get files deleted more than 30 days ago
        let query = r#"
            SELECT id, file_path, bucket_name, object_key
            FROM file_uploads
            WHERE status = 'deleted' 
            AND deleted_at < DATE_SUB(NOW(), INTERVAL 30 DAY)
        "#;

        let files = sqlx::query(query)
            .fetch_all(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let mut deleted_count = 0;
        for file in files {
            let file_id: Uuid = file.try_get("id")?;
            
            // TODO: Delete from OSS/S3
            // let file_path: String = file.try_get("file_path")?;
            // oss_client.delete_object(&file_path).await?;

            // Delete record from database
            let delete_query = "DELETE FROM file_uploads WHERE id = ?";
            sqlx::query(delete_query)
                .bind(&file_id)
                .execute(db)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            deleted_count += 1;
        }

        Ok(deleted_count)
    }
}

impl std::fmt::Display for FileType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FileType::Image => write!(f, "image"),
            FileType::Video => write!(f, "video"),
            FileType::Document => write!(f, "document"),
            FileType::Audio => write!(f, "audio"),
            FileType::Other => write!(f, "other"),
        }
    }
}