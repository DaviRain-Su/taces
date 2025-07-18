use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
    Extension,
};
use uuid::Uuid;
use validator::Validate;
use serde::Deserialize;
use crate::{
    AppState,
    models::{user::*, ApiResponse},
    services::user_service,
    middleware::auth::AuthUser,
};

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    page: Option<u32>,
    per_page: Option<u32>,
    search: Option<String>,
    role: Option<String>,
    status: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct BatchDeleteRequest {
    ids: Vec<Uuid>,
}

pub async fn list_users(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ApiResponse<Vec<User>>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Only admin can list all users
    if auth_user.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Insufficient permissions")),
        ));
    }

    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);
    
    match user_service::list_users(&app_state.pool, page, per_page, query.search, query.role, query.status).await {
        Ok(users) => Ok(Json(ApiResponse::success("Users retrieved successfully", users))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Failed to retrieve users: {}", e))),
        )),
    }
}

pub async fn get_user(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<User>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Users can view their own profile, admins can view any profile
    if auth_user.user_id != id && auth_user.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Insufficient permissions")),
        ));
    }
    
    match user_service::get_user_by_id(&app_state.pool, id).await {
        Ok(user) => Ok(Json(ApiResponse::success("User retrieved successfully", user))),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(&format!("User not found: {}", e))),
        )),
    }
}

pub async fn update_user(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateUserDto>,
) -> Result<Json<ApiResponse<User>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Users can update their own profile, admins can update any profile
    if auth_user.user_id != id && auth_user.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Insufficient permissions")),
        ));
    }
    
    dto.validate()
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error(&format!("Validation error: {}", e))),
            )
        })?;
    
    match user_service::update_user(&app_state.pool, id, dto).await {
        Ok(user) => Ok(Json(ApiResponse::success("User updated successfully", user))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Failed to update user: {}", e))),
        )),
    }
}

pub async fn delete_user(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Only admin can delete users
    if auth_user.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Insufficient permissions")),
        ));
    }
    
    match user_service::delete_user(&app_state.pool, id).await {
        Ok(_) => Ok(Json(ApiResponse::success("User deleted successfully", ()))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Failed to delete user: {}", e))),
        )),
    }
}

pub async fn batch_delete_users(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Json(request): Json<BatchDeleteRequest>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Only admin can batch delete users
    if auth_user.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Insufficient permissions")),
        ));
    }
    
    match user_service::batch_delete_users(&app_state.pool, request.ids).await {
        Ok(count) => Ok(Json(ApiResponse::success(&format!("{} users deleted successfully", count), ()))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Failed to delete users: {}", e))),
        )),
    }
}

pub async fn export_users(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ApiResponse<String>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Only admin can export users
    if auth_user.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Insufficient permissions")),
        ));
    }
    
    match user_service::export_users(&app_state.pool, query.search, query.role, query.status).await {
        Ok(csv_data) => Ok(Json(ApiResponse::success("Users exported successfully", csv_data))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Failed to export users: {}", e))),
        )),
    }
}