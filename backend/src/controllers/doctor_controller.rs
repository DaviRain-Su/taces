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
    models::{doctor::*, ApiResponse},
    services::doctor_service,
    middleware::auth::AuthUser,
};

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    page: Option<u32>,
    per_page: Option<u32>,
    department: Option<String>,
    search: Option<String>,
}

pub async fn list_doctors(
    State(app_state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ApiResponse<Vec<Doctor>>>, (StatusCode, Json<ApiResponse<()>>)> {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);
    
    match doctor_service::list_doctors(&app_state.pool, page, per_page, query.department, query.search).await {
        Ok(doctors) => Ok(Json(ApiResponse::success("Doctors retrieved successfully", doctors))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Failed to retrieve doctors: {}", e))),
        )),
    }
}

pub async fn get_doctor(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Doctor>>, (StatusCode, Json<ApiResponse<()>>)> {
    match doctor_service::get_doctor_by_id(&app_state.pool, id).await {
        Ok(doctor) => Ok(Json(ApiResponse::success("Doctor retrieved successfully", doctor))),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(&format!("Doctor not found: {}", e))),
        )),
    }
}

pub async fn get_doctor_by_user_id(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<ApiResponse<Doctor>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Users can view their own doctor profile, admins can view any
    if auth_user.user_id != user_id && auth_user.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Insufficient permissions")),
        ));
    }
    
    match doctor_service::get_doctor_by_user_id(&app_state.pool, user_id).await {
        Ok(doctor) => Ok(Json(ApiResponse::success("Doctor retrieved successfully", doctor))),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(&format!("Doctor not found: {}", e))),
        )),
    }
}

pub async fn create_doctor(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Json(dto): Json<CreateDoctorDto>,
) -> Result<Json<ApiResponse<Doctor>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Only admin can create doctor profiles
    if auth_user.role != "admin" {
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
    
    match doctor_service::create_doctor(&app_state.pool, dto).await {
        Ok(doctor) => Ok(Json(ApiResponse::success("Doctor created successfully", doctor))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Failed to create doctor: {}", e))),
        )),
    }
}

pub async fn update_doctor(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateDoctorDto>,
) -> Result<Json<ApiResponse<Doctor>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Check if the doctor belongs to the authenticated user
    let doctor = match doctor_service::get_doctor_by_id(&app_state.pool, id).await {
        Ok(d) => d,
        Err(_) => return Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Doctor not found")),
        )),
    };
    
    // Users can update their own doctor profile, admins can update any
    if doctor.user_id != auth_user.user_id && auth_user.role != "admin" {
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
    
    match doctor_service::update_doctor(&app_state.pool, id, dto).await {
        Ok(doctor) => Ok(Json(ApiResponse::success("Doctor updated successfully", doctor))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Failed to update doctor: {}", e))),
        )),
    }
}

pub async fn update_doctor_photos(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(photos): Json<DoctorPhotos>,
) -> Result<Json<ApiResponse<Doctor>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Check if the doctor belongs to the authenticated user
    let doctor = match doctor_service::get_doctor_by_id(&app_state.pool, id).await {
        Ok(d) => d,
        Err(_) => return Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Doctor not found")),
        )),
    };
    
    // Users can update their own doctor photos, admins can update any
    if doctor.user_id != auth_user.user_id && auth_user.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Insufficient permissions")),
        ));
    }
    
    match doctor_service::update_doctor_photos(&app_state.pool, id, photos).await {
        Ok(doctor) => Ok(Json(ApiResponse::success("Doctor photos updated successfully", doctor))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Failed to update doctor photos: {}", e))),
        )),
    }
}