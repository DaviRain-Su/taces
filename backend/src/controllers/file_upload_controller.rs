use crate::config::database::DbPool;
use crate::middleware::auth::AuthUser;
use crate::models::file_upload::*;
use crate::models::{ApiResponse, UserRole};
use crate::services::file_upload_service::FileUploadService;
use crate::utils::errors::AppError;
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use uuid::Uuid;

pub async fn create_upload(
    State(db): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(dto): Json<CreateFileUploadDto>,
) -> Result<impl IntoResponse, AppError> {
    let response = FileUploadService::create_upload(&db, auth_user.user_id, dto).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success("获取上传链接成功", response)),
    ))
}

pub async fn complete_upload(
    State(db): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(upload_id): Path<Uuid>,
    Json(dto): Json<CompleteUploadDto>,
) -> Result<impl IntoResponse, AppError> {
    let file = FileUploadService::complete_upload(&db, upload_id, auth_user.user_id, dto).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success("文件上传完成", file)),
    ))
}

pub async fn get_file(
    State(db): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(file_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let file = FileUploadService::get_file(&db, file_id).await?;

    // Check authorization
    if !file.is_public && file.user_id != auth_user.user_id && auth_user.role != UserRole::Admin {
        return Err(AppError::Forbidden);
    }

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success("获取文件信息成功", file)),
    ))
}

pub async fn list_files(
    State(db): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<FileListQuery>,
) -> Result<impl IntoResponse, AppError> {
    // For non-admin users, only show their own files
    let mut query_params = query;
    if auth_user.role != UserRole::Admin {
        query_params.user_id = Some(auth_user.user_id);
    }

    let response = FileUploadService::list_files(&db, query_params).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success("获取文件列表成功", response)),
    ))
}

pub async fn delete_file(
    State(db): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(file_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let is_admin = auth_user.role == UserRole::Admin;
    FileUploadService::delete_file(&db, file_id, auth_user.user_id, is_admin).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success("文件删除成功", json!({}))),
    ))
}

pub async fn get_file_stats(
    State(db): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
    let user_id = if auth_user.role == UserRole::Admin {
        None
    } else {
        Some(auth_user.user_id)
    };

    let stats = FileUploadService::get_file_stats(&db, user_id).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success("获取文件统计成功", stats)),
    ))
}

// Configuration endpoints (admin only)
pub async fn get_upload_config(
    State(db): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
    if auth_user.role != UserRole::Admin {
        return Err(AppError::Forbidden);
    }

    let config = FileUploadService::get_upload_config(&db).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success("获取上传配置成功", config)),
    ))
}

pub async fn get_image_config(
    State(db): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
    if auth_user.role != UserRole::Admin {
        return Err(AppError::Forbidden);
    }

    let config = FileUploadService::get_image_config(&db).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success("获取图片配置成功", config)),
    ))
}

pub async fn get_video_config(
    State(db): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
    if auth_user.role != UserRole::Admin {
        return Err(AppError::Forbidden);
    }

    let config = FileUploadService::get_video_config(&db).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success("获取视频配置成功", config)),
    ))
}

pub async fn update_system_config(
    State(db): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path((category, key)): Path<(String, String)>,
    Json(dto): Json<UpdateSystemConfigDto>,
) -> Result<impl IntoResponse, AppError> {
    if auth_user.role != UserRole::Admin {
        return Err(AppError::Forbidden);
    }

    let config = FileUploadService::update_system_config(&db, &category, &key, dto).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success("配置更新成功", config)),
    ))
}