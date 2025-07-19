use crate::{
    middleware::auth::AuthUser,
    models::{file_upload::*, ApiResponse},
    services::{file_storage_service::FileStorageService, file_upload_service},
    AppState,
};
use axum::{
    extract::{Multipart, Path as AxumPath, Query, State},
    http::StatusCode,
    Extension, Json,
};
use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct UploadQuery {
    file_type: Option<String>,
    purpose: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct PresignedUrlRequest {
    filename: String,
    file_type: FileType,
    purpose: FilePurpose,
}

/// Upload file with S3/OSS support
pub async fn upload_file_enhanced(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Query(query): Query<UploadQuery>,
    mut multipart: Multipart,
) -> Result<Json<ApiResponse<FileUpload>>, (StatusCode, Json<ApiResponse<()>>)> {
    let file_type = query.file_type
        .and_then(|t| serde_json::from_str::<FileType>(&format!("\"{}\"", t)).ok())
        .unwrap_or(FileType::Other);
    
    let purpose = query.purpose
        .and_then(|p| serde_json::from_str::<FilePurpose>(&format!("\"{}\"", p)).ok())
        .unwrap_or(FilePurpose::Other);
    
    while let Some(field) = multipart.next_field().await.unwrap() {
        let name = field.name().unwrap_or("file").to_string();
        if name != "file" {
            continue;
        }
        
        let filename = field.file_name().unwrap_or("unnamed").to_string();
        let content_type = field.content_type().unwrap_or("application/octet-stream").to_string();
        let data = field.bytes().await.unwrap();
        
        // Validate file
        FileStorageService::validate_file_extension(&filename, &file_type)
            .map_err(|e| (StatusCode::BAD_REQUEST, Json(ApiResponse::error(&e.to_string()))))?;
        
        FileStorageService::validate_file_size(data.len(), &file_type)
            .map_err(|e| (StatusCode::BAD_REQUEST, Json(ApiResponse::error(&e.to_string()))))?;
        
        // Generate file path
        let file_path = FileStorageService::generate_file_path(&file_type, &filename);
        
        // Upload to S3 or local storage
        let url = if let Some(s3_client) = &app_state.s3_client {
            FileStorageService::upload_to_cloud(
                s3_client,
                &file_path,
                data.to_vec(),
                &content_type,
            )
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!("Failed to upload file: {}", e))),
                )
            })?
        } else {
            FileStorageService::upload_to_local(&file_path, data.to_vec())
                .await
                .map_err(|e| {
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::error(&format!("Failed to upload file: {}", e))),
                    )
                })?
        };
        
        // Create file upload record
        let create_dto = CreateFileUploadDto {
            filename: filename.clone(),
            original_filename: filename,
            file_path: file_path.clone(),
            file_size: data.len() as i64,
            file_type,
            mime_type: content_type,
            purpose,
            url,
            uploaded_by: auth_user.user_id,
        };
        
        let file_upload = file_upload_service::create_file_upload(&app_state.pool, create_dto)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!("Failed to save file record: {}", e))),
                )
            })?;
        
        return Ok(Json(ApiResponse::success(
            "File uploaded successfully",
            file_upload,
        )));
    }
    
    Err((
        StatusCode::BAD_REQUEST,
        Json(ApiResponse::error("No file provided")),
    ))
}

/// Generate pre-signed URL for direct upload
pub async fn generate_presigned_url(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Json(request): Json<PresignedUrlRequest>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Validate file
    FileStorageService::validate_file_extension(&request.filename, &request.file_type)
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(ApiResponse::error(&e.to_string()))))?;
    
    if let Some(s3_client) = &app_state.s3_client {
        let file_path = FileStorageService::generate_file_path(&request.file_type, &request.filename);
        let content_type = FileStorageService::get_content_type(&request.filename);
        
        // Generate pre-signed URL (valid for 1 hour)
        let presigned_url = FileStorageService::generate_presigned_upload_url(
            s3_client,
            &file_path,
            content_type,
            3600,
        )
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&format!("Failed to generate presigned URL: {}", e))),
            )
        })?;
        
        // Create pending file upload record
        let create_dto = CreateFileUploadDto {
            filename: request.filename.clone(),
            original_filename: request.filename,
            file_path: file_path.clone(),
            file_size: 0, // Will be updated after upload
            file_type: request.file_type,
            mime_type: content_type.to_string(),
            purpose: request.purpose,
            url: String::new(), // Will be updated after upload
            uploaded_by: auth_user.user_id,
        };
        
        let file_upload = file_upload_service::create_file_upload(&app_state.pool, create_dto)
            .await
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!("Failed to create file record: {}", e))),
                )
            })?;
        
        let response = serde_json::json!({
            "upload_id": file_upload.id,
            "presigned_url": presigned_url,
            "file_path": file_path,
            "expires_in": 3600,
        });
        
        Ok(Json(ApiResponse::success(
            "Presigned URL generated successfully",
            response,
        )))
    } else {
        Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(ApiResponse::error("Cloud storage not configured")),
        ))
    }
}

/// Delete file with S3/OSS support
pub async fn delete_file_enhanced(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    AxumPath(id): AxumPath<Uuid>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Get file record
    let file = file_upload_service::get_file_by_id(&app_state.pool, id)
        .await
        .map_err(|e| {
            (
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error(&format!("File not found: {}", e))),
            )
        })?;
    
    // Check permission
    if file.uploaded_by != auth_user.user_id && auth_user.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Insufficient permissions")),
        ));
    }
    
    // Delete from storage
    if let Some(s3_client) = &app_state.s3_client {
        FileStorageService::delete_from_cloud(s3_client, &file.file_path)
            .await
            .map_err(|e| {
                tracing::warn!("Failed to delete file from S3: {}", e);
                // Continue even if S3 deletion fails
            })
            .ok();
    } else {
        FileStorageService::delete_from_local(&file.file_path)
            .await
            .map_err(|e| {
                tracing::warn!("Failed to delete file from local storage: {}", e);
                // Continue even if local deletion fails
            })
            .ok();
    }
    
    // Delete database record
    file_upload_service::delete_file(&app_state.pool, id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&format!("Failed to delete file record: {}", e))),
            )
        })?;
    
    Ok(Json(ApiResponse::success("File deleted successfully", ())))
}

/// Batch delete files
pub async fn batch_delete_files(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Json(file_ids): Json<Vec<Uuid>>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    let mut deleted_count = 0;
    let mut file_paths = Vec::new();
    
    for id in &file_ids {
        // Get file record
        if let Ok(file) = file_upload_service::get_file_by_id(&app_state.pool, *id).await {
            // Check permission
            if file.uploaded_by == auth_user.user_id || auth_user.role == "admin" {
                file_paths.push(file.file_path.clone());
                
                // Delete database record
                if file_upload_service::delete_file(&app_state.pool, *id).await.is_ok() {
                    deleted_count += 1;
                }
            }
        }
    }
    
    // Batch delete from storage
    if !file_paths.is_empty() {
        if let Some(s3_client) = &app_state.s3_client {
            FileStorageService::batch_delete_from_cloud(s3_client, file_paths)
                .await
                .map_err(|e| {
                    tracing::warn!("Failed to batch delete files from S3: {}", e);
                })
                .ok();
        }
    }
    
    let response = serde_json::json!({
        "requested": file_ids.len(),
        "deleted": deleted_count,
    });
    
    Ok(Json(ApiResponse::success(
        "Batch delete completed",
        response,
    )))
}