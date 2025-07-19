use crate::{
    middleware::auth::AuthUser,
    models::{user::*, ApiResponse},
    services::auth_service_cached as auth_service,
    AppState,
};
use axum::{extract::State, http::StatusCode, Extension, Json};
use axum_extra::{headers, TypedHeader};
use validator::Validate;

pub async fn register(
    State(app_state): State<AppState>,
    Json(dto): Json<CreateUserDto>,
) -> Result<Json<ApiResponse<User>>, (StatusCode, Json<ApiResponse<()>>)> {
    dto.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(&format!("Validation error: {}", e))),
        )
    })?;

    match auth_service::register_user_cached(&app_state.pool, &app_state.redis, dto).await {
        Ok(user) => Ok(Json(ApiResponse::success(
            "User registered successfully",
            user,
        ))),
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
    dto.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(&format!("Validation error: {}", e))),
        )
    })?;

    match auth_service::login_cached(&app_state.pool, &app_state.redis, &app_state.config, dto).await {
        Ok(response) => Ok(Json(ApiResponse::success("Login successful", response))),
        Err(e) => Err((
            StatusCode::UNAUTHORIZED,
            Json(ApiResponse::error(&format!("Login failed: {}", e))),
        )),
    }
}

pub async fn logout(
    State(app_state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    TypedHeader(auth_header): TypedHeader<headers::Authorization<headers::authorization::Bearer>>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    let token = auth_header.token();
    
    match auth_service::logout_cached(&app_state.redis, token).await {
        Ok(_) => Ok(Json(ApiResponse::success("Logout successful", ()))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Logout failed: {}", e))),
        )),
    }
}
