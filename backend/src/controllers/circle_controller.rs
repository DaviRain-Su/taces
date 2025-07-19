use crate::AppState;
use crate::middleware::auth::AuthUser;
use crate::models::{ApiResponse, CreateCircleDto, UpdateCircleDto, UpdateMemberRoleDto};
use crate::services::circle_service::CircleService;
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize)]
pub struct CircleQuery {
    pub category: Option<String>,
    pub keyword: Option<String>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

pub async fn create_circle(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Json(dto): Json<CreateCircleDto>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Validate input
    if let Err(e) = dto.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(&format!("Validation error: {}", e))),
        ));
    }

    let circle = CircleService::create_circle(&state.pool, auth_user.user_id, dto)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&format!("Failed to create circle: {}", e))),
            )
        })?;

    Ok(Json(ApiResponse::success(
        "Circle created successfully",
        serde_json::to_value(&circle).unwrap(),
    )))
}

pub async fn get_circles(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Query(query): Query<CircleQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(10).min(100);

    let (circles, total) = CircleService::get_circles(
        &state.pool,
        Some(auth_user.user_id),
        query.category,
        query.keyword,
        page,
        page_size,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Failed to get circles: {}", e))),
        )
    })?;

    Ok(Json(ApiResponse::success(
        "Circles retrieved successfully",
        serde_json::json!({
            "circles": circles,
            "pagination": {
                "page": page,
                "page_size": page_size,
                "total": total,
                "total_pages": (total as f64 / page_size as f64).ceil() as i64,
            }
        }),
    )))
}

pub async fn get_circle_by_id(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    let circle_info = CircleService::get_circle_by_id(&state.pool, id, Some(auth_user.user_id))
        .await
        .map_err(|e| {
            if e.to_string().contains("not found") {
                (
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::error("Circle not found")),
                )
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!("Failed to get circle: {}", e))),
                )
            }
        })?;

    Ok(Json(ApiResponse::success(
        "Circle retrieved successfully",
        serde_json::to_value(&circle_info).unwrap(),
    )))
}

pub async fn update_circle(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateCircleDto>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Validate input
    if let Err(e) = dto.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(&format!("Validation error: {}", e))),
        ));
    }

    let is_admin = auth_user.role == "admin";
    let circle = CircleService::update_circle(
        &state.pool,
        id,
        auth_user.user_id,
        is_admin,
        dto,
    )
    .await
    .map_err(|e| {
        let error_str = e.to_string();
        if error_str.contains("permission") || error_str.contains("Only circle owner") || error_str.contains("can update") {
            (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("No permission to update this circle")),
            )
        } else {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&format!("Failed to update circle: {}", e))),
            )
        }
    })?;

    Ok(Json(ApiResponse::success(
        "Circle updated successfully",
        serde_json::to_value(&circle).unwrap(),
    )))
}

pub async fn delete_circle(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    let is_admin = auth_user.role == "admin";
    CircleService::delete_circle(&state.pool, id, auth_user.user_id, is_admin)
        .await
        .map_err(|e| {
            let error_str = e.to_string();
            if error_str.contains("permission") || error_str.contains("Only circle owner") || error_str.contains("can delete") {
                (
                    StatusCode::FORBIDDEN,
                    Json(ApiResponse::error("No permission to delete this circle")),
                )
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!("Failed to delete circle: {}", e))),
                )
            }
        })?;

    Ok(Json(ApiResponse::success(
        "Circle deleted successfully",
        (),
    )))
}

pub async fn join_circle(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    CircleService::join_circle(&state.pool, id, auth_user.user_id)
        .await
        .map_err(|e| {
            if e.to_string().contains("Already joined") {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::error("Already joined this circle")),
                )
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!("Failed to join circle: {}", e))),
                )
            }
        })?;

    Ok(Json(ApiResponse::success(
        "Joined circle successfully",
        (),
    )))
}

pub async fn leave_circle(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    CircleService::leave_circle(&state.pool, id, auth_user.user_id)
        .await
        .map_err(|e| {
            if e.to_string().contains("Owner cannot leave") {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::error("Owner cannot leave the circle")),
                )
            } else if e.to_string().contains("Not a member") {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::error("Not a member of this circle")),
                )
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!("Failed to leave circle: {}", e))),
                )
            }
        })?;

    Ok(Json(ApiResponse::success(
        "Left circle successfully",
        (),
    )))
}

pub async fn get_circle_members(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).min(100);

    let (members, total) = CircleService::get_circle_members(&state.pool, id, page, page_size)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&format!(
                    "Failed to get circle members: {}",
                    e
                ))),
            )
        })?;

    Ok(Json(ApiResponse::success(
        "Circle members retrieved successfully",
        serde_json::json!({
            "members": members,
            "pagination": {
                "page": page,
                "page_size": page_size,
                "total": total,
                "total_pages": (total as f64 / page_size as f64).ceil() as i64,
            }
        }),
    )))
}

pub async fn update_member_role(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path((circle_id, user_id)): Path<(Uuid, Uuid)>,
    Json(dto): Json<UpdateMemberRoleDto>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    let is_admin = auth_user.role == "admin";
    CircleService::update_member_role(
        &state.pool,
        circle_id,
        user_id,
        auth_user.user_id,
        is_admin,
        dto,
    )
    .await
    .map_err(|e| {
        if e.to_string().contains("permission") {
            (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("No permission to update member role")),
            )
        } else {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&format!(
                    "Failed to update member role: {}",
                    e
                ))),
            )
        }
    })?;

    Ok(Json(ApiResponse::success(
        "Member role updated successfully",
        (),
    )))
}

pub async fn remove_member(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path((circle_id, user_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    let is_admin = auth_user.role == "admin";
    CircleService::remove_member(
        &state.pool,
        circle_id,
        user_id,
        auth_user.user_id,
        is_admin,
    )
    .await
    .map_err(|e| {
        if e.to_string().contains("permission") {
            (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("No permission to remove member")),
            )
        } else if e.to_string().contains("Cannot remove owner") {
            (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error("Cannot remove owner")),
            )
        } else {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&format!(
                    "Failed to remove member: {}",
                    e
                ))),
            )
        }
    })?;

    Ok(Json(ApiResponse::success(
        "Member removed successfully",
        (),
    )))
}

pub async fn get_user_circles(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(10).min(100);

    let (circles, total) = CircleService::get_user_circles(
        &state.pool,
        auth_user.user_id,
        page,
        page_size,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!(
                "Failed to get user circles: {}",
                e
            ))),
        )
    })?;

    Ok(Json(ApiResponse::success(
        "User circles retrieved successfully",
        serde_json::json!({
            "circles": circles,
            "pagination": {
                "page": page,
                "page_size": page_size,
                "total": total,
                "total_pages": (total as f64 / page_size as f64).ceil() as i64,
            }
        }),
    )))
}