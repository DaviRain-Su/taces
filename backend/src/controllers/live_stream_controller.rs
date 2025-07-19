use crate::{
    middleware::auth::AuthUser,
    models::{live_stream::*, ApiResponse},
    services::{live_stream_service, user_service},
    AppState,
};
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    page: Option<u32>,
    per_page: Option<u32>,
    status: Option<String>,
}

pub async fn list_live_streams(
    Query(params): Query<ListQuery>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<LiveStreamListItem>>>, (StatusCode, Json<ApiResponse<()>>)> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(10);

    match live_stream_service::list_live_streams(&state.pool, page, per_page, params.status, None)
        .await
    {
        Ok(streams) => Ok(Json(ApiResponse::success(
            "Live streams fetched successfully",
            streams,
        ))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!(
                "Failed to fetch live streams: {}",
                e
            ))),
        )),
    }
}

pub async fn get_live_stream(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<LiveStream>>, (StatusCode, Json<ApiResponse<()>>)> {
    match live_stream_service::get_live_stream_by_id(&state.pool, id).await {
        Ok(stream) => Ok(Json(ApiResponse::success(
            "Live stream fetched successfully",
            stream,
        ))),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(&format!("Live stream not found: {}", e))),
        )),
    }
}

pub async fn create_live_stream(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Json(dto): Json<CreateLiveStreamDto>,
) -> Result<Json<ApiResponse<LiveStream>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Only admins and doctors can create live streams
    if auth_user.role != "admin" && auth_user.role != "doctor" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error(
                "Only admins and doctors can create live streams",
            )),
        ));
    }

    dto.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(&format!("Validation error: {}", e))),
        )
    })?;

    // Get user name
    let user_name = match user_service::get_user_by_id(&state.pool, auth_user.user_id).await {
        Ok(user) => user.name,
        Err(_) => "Unknown".to_string(),
    };

    match live_stream_service::create_live_stream(&state.pool, auth_user.user_id, user_name, dto)
        .await
    {
        Ok(stream) => Ok(Json(ApiResponse::success(
            "Live stream created successfully",
            stream,
        ))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!(
                "Failed to create live stream: {}",
                e
            ))),
        )),
    }
}

pub async fn update_live_stream(
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(dto): Json<UpdateLiveStreamDto>,
) -> Result<Json<ApiResponse<LiveStream>>, (StatusCode, Json<ApiResponse<()>>)> {
    dto.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(&format!("Validation error: {}", e))),
        )
    })?;

    let is_admin = auth_user.role == "admin";

    match live_stream_service::update_live_stream(&state.pool, id, auth_user.user_id, is_admin, dto)
        .await
    {
        Ok(stream) => Ok(Json(ApiResponse::success(
            "Live stream updated successfully",
            stream,
        ))),
        Err(e) => {
            let status = if e.to_string().contains("permissions") {
                StatusCode::FORBIDDEN
            } else if e.to_string().contains("Cannot update") {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            Err((status, Json(ApiResponse::error(&e.to_string()))))
        }
    }
}

pub async fn start_live_stream(
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(dto): Json<StartLiveStreamDto>,
) -> Result<Json<ApiResponse<LiveStream>>, (StatusCode, Json<ApiResponse<()>>)> {
    dto.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(&format!("Validation error: {}", e))),
        )
    })?;

    let is_admin = auth_user.role == "admin";

    match live_stream_service::start_live_stream(&state.pool, id, auth_user.user_id, is_admin, dto)
        .await
    {
        Ok(stream) => Ok(Json(ApiResponse::success(
            "Live stream started successfully",
            stream,
        ))),
        Err(e) => {
            let status = if e.to_string().contains("permissions") {
                StatusCode::FORBIDDEN
            } else if e.to_string().contains("not in scheduled") {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            Err((status, Json(ApiResponse::error(&e.to_string()))))
        }
    }
}

pub async fn end_live_stream(
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<LiveStream>>, (StatusCode, Json<ApiResponse<()>>)> {
    let is_admin = auth_user.role == "admin";

    match live_stream_service::end_live_stream(&state.pool, id, auth_user.user_id, is_admin).await
    {
        Ok(stream) => Ok(Json(ApiResponse::success(
            "Live stream ended successfully",
            stream,
        ))),
        Err(e) => {
            let status = if e.to_string().contains("permissions") {
                StatusCode::FORBIDDEN
            } else if e.to_string().contains("not currently live") {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            Err((status, Json(ApiResponse::error(&e.to_string()))))
        }
    }
}

pub async fn delete_live_stream(
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    let is_admin = auth_user.role == "admin";

    match live_stream_service::delete_live_stream(&state.pool, id, auth_user.user_id, is_admin)
        .await
    {
        Ok(_) => Ok(Json(ApiResponse::success(
            "Live stream deleted successfully",
            (),
        ))),
        Err(e) => {
            let status = if e.to_string().contains("permissions") {
                StatusCode::FORBIDDEN
            } else if e.to_string().contains("only delete scheduled") {
                StatusCode::BAD_REQUEST
            } else {
                StatusCode::INTERNAL_SERVER_ERROR
            };
            Err((status, Json(ApiResponse::error(&e.to_string()))))
        }
    }
}

pub async fn get_upcoming_live_streams(
    State(state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<LiveStreamListItem>>>, (StatusCode, Json<ApiResponse<()>>)> {
    match live_stream_service::get_upcoming_live_streams(&state.pool, 10).await {
        Ok(streams) => Ok(Json(ApiResponse::success(
            "Upcoming live streams fetched successfully",
            streams,
        ))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!(
                "Failed to fetch upcoming live streams: {}",
                e
            ))),
        )),
    }
}

pub async fn get_my_live_streams(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Query(params): Query<ListQuery>,
) -> Result<Json<ApiResponse<Vec<LiveStreamListItem>>>, (StatusCode, Json<ApiResponse<()>>)> {
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(10);

    match live_stream_service::list_live_streams(
        &state.pool,
        page,
        per_page,
        params.status,
        Some(auth_user.user_id),
    )
    .await
    {
        Ok(streams) => Ok(Json(ApiResponse::success(
            "My live streams fetched successfully",
            streams,
        ))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!(
                "Failed to fetch live streams: {}",
                e
            ))),
        )),
    }
}