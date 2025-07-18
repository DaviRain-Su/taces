use axum::{
    extract::State,
    http::StatusCode,
    Json,
};
use uuid::Uuid;
use validator::Validate;
use crate::{
    config::{Config, database::DbPool},
    models::{user::*, ApiResponse},
    services::auth_service,
};

pub async fn register(
    State(pool): State<DbPool>,
    State(config): State<Config>,
    Json(dto): Json<CreateUserDto>,
) -> Result<Json<ApiResponse<User>>, (StatusCode, Json<ApiResponse<()>>)> {
    dto.validate()
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error(&format!("Validation error: {}", e))),
            )
        })?;

    match auth_service::register_user(&pool, dto).await {
        Ok(user) => Ok(Json(ApiResponse::success("User registered successfully", user))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Registration failed: {}", e))),
        )),
    }
}

pub async fn login(
    State(pool): State<DbPool>,
    State(config): State<Config>,
    Json(dto): Json<LoginDto>,
) -> Result<Json<ApiResponse<LoginResponse>>, (StatusCode, Json<ApiResponse<()>>)> {
    dto.validate()
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error(&format!("Validation error: {}", e))),
            )
        })?;

    match auth_service::login(&pool, &config, dto).await {
        Ok(response) => Ok(Json(ApiResponse::success("Login successful", response))),
        Err(e) => Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::error(&format!("Login failed: {}", e))),
        )),
    }
}