use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use validator::Validate;
use crate::{
    AppState,
    models::{user::*, ApiResponse},
    services::auth_service,
};

pub async fn register(
    State(app_state): State<AppState>,
    Json(dto): Json<CreateUserDto>,
) -> Result<Json<ApiResponse<User>>, (StatusCode, Json<ApiResponse<()>>)> {
    dto.validate()
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error(&format!("Validation error: {}", e))),
            )
        })?;

    match auth_service::register_user(&app_state.pool, dto).await {
        Ok(user) => Ok(Json(ApiResponse::success("User registered successfully", user))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Registration failed: {}", e))),
        )),
    }
}

pub async fn login(
    State(app_state): State<AppState>,
    Json(dto): Json<LoginDto>,
) -> Result<Json<ApiResponse<LoginResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    dto.validate()
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error(&format!("Validation error: {}", e))),
            )
        })?;

    match auth_service::login(&app_state.pool, &app_state.config, dto).await {
        Ok(response) => Ok(Json(ApiResponse::success("Login successful", response))),
        Err(e) => Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::error(&format!("Login failed: {}", e))),
        )),
    }
}