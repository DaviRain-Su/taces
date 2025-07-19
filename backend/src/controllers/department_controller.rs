use crate::{
    middleware::auth::AuthUser,
    models::{department::*, ApiResponse},
    services::department_service,
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    page: Option<u32>,
    per_page: Option<u32>,
    search: Option<String>,
    status: Option<String>,
}

pub async fn list_departments(
    State(app_state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ApiResponse<Vec<Department>>>, (StatusCode, Json<ApiResponse<()>>)> {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);

    match department_service::list_departments(
        &app_state.pool,
        page,
        per_page,
        query.search,
        query.status,
    )
    .await
    {
        Ok(departments) => Ok(Json(ApiResponse::success(
            "Departments retrieved successfully",
            departments,
        ))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!(
                "Failed to retrieve departments: {}",
                e
            ))),
        )),
    }
}

pub async fn get_department(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Department>>, (StatusCode, Json<ApiResponse<()>>)> {
    match department_service::get_department_by_id(&app_state.pool, id).await {
        Ok(department) => Ok(Json(ApiResponse::success(
            "Department retrieved successfully",
            department,
        ))),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(&format!("Department not found: {}", e))),
        )),
    }
}

pub async fn get_department_by_code(
    State(app_state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Json<ApiResponse<Department>>, (StatusCode, Json<ApiResponse<()>>)> {
    match department_service::get_department_by_code(&app_state.pool, &code).await {
        Ok(department) => Ok(Json(ApiResponse::success(
            "Department retrieved successfully",
            department,
        ))),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(&format!("Department not found: {}", e))),
        )),
    }
}

pub async fn create_department(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Json(dto): Json<CreateDepartmentDto>,
) -> Result<Json<ApiResponse<Department>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Only admin can create departments
    if auth_user.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Insufficient permissions")),
        ));
    }

    dto.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(&format!("Validation error: {}", e))),
        )
    })?;

    match department_service::create_department(&app_state.pool, dto).await {
        Ok(department) => Ok(Json(ApiResponse::success(
            "Department created successfully",
            department,
        ))),
        Err(e) => {
            if e.to_string().contains("Duplicate entry") {
                Err((
                    StatusCode::CONFLICT,
                    Json(ApiResponse::error("Department code already exists")),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!(
                        "Failed to create department: {}",
                        e
                    ))),
                ))
            }
        }
    }
}

pub async fn update_department(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateDepartmentDto>,
) -> Result<Json<ApiResponse<Department>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Only admin can update departments
    if auth_user.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Insufficient permissions")),
        ));
    }

    dto.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(&format!("Validation error: {}", e))),
        )
    })?;

    match department_service::update_department(&app_state.pool, id, dto).await {
        Ok(department) => Ok(Json(ApiResponse::success(
            "Department updated successfully",
            department,
        ))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!(
                "Failed to update department: {}",
                e
            ))),
        )),
    }
}

pub async fn delete_department(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Only admin can delete departments
    if auth_user.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Insufficient permissions")),
        ));
    }

    match department_service::delete_department(&app_state.pool, id).await {
        Ok(_) => Ok(Json(ApiResponse::success(
            "Department deleted successfully",
            (),
        ))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!(
                "Failed to delete department: {}",
                e
            ))),
        )),
    }
}
