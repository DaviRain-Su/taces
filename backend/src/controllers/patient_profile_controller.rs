use crate::{
    middleware::auth::AuthUser,
    models::{patient_profile::*, ApiResponse},
    services::patient_profile_service,
    AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

pub async fn list_profiles(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<PatientProfile>>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Only patients can manage patient profiles
    if auth_user.role != "patient" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error(
                "Only patients can manage patient profiles",
            )),
        ));
    }

    match patient_profile_service::list_user_profiles(&app_state.pool, auth_user.user_id).await {
        Ok(profiles) => Ok(Json(ApiResponse::success(
            "Patient profiles retrieved successfully",
            profiles,
        ))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!(
                "Failed to retrieve patient profiles: {}",
                e
            ))),
        )),
    }
}

pub async fn get_profile(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<PatientProfile>>, (StatusCode, Json<ApiResponse<()>>)> {
    if auth_user.role != "patient" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error(
                "Only patients can manage patient profiles",
            )),
        ));
    }

    match patient_profile_service::get_profile_by_id(&app_state.pool, id, auth_user.user_id).await {
        Ok(profile) => Ok(Json(ApiResponse::success(
            "Patient profile retrieved successfully",
            profile,
        ))),
        Err(e) => {
            if e.to_string().contains("not found") {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::error("Patient profile not found")),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!(
                        "Failed to retrieve patient profile: {}",
                        e
                    ))),
                ))
            }
        }
    }
}

pub async fn get_default_profile(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
) -> Result<Json<ApiResponse<Option<PatientProfile>>>, (StatusCode, Json<ApiResponse<()>>)> {
    if auth_user.role != "patient" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error(
                "Only patients can manage patient profiles",
            )),
        ));
    }

    match patient_profile_service::get_default_profile(&app_state.pool, auth_user.user_id).await {
        Ok(profile) => Ok(Json(ApiResponse::success(
            "Default patient profile retrieved successfully",
            profile,
        ))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!(
                "Failed to retrieve default profile: {}",
                e
            ))),
        )),
    }
}

pub async fn create_profile(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Json(dto): Json<CreatePatientProfileDto>,
) -> Result<Json<ApiResponse<PatientProfile>>, (StatusCode, Json<ApiResponse<()>>)> {
    if auth_user.role != "patient" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error(
                "Only patients can manage patient profiles",
            )),
        ));
    }

    dto.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(&format!("Validation error: {}", e))),
        )
    })?;

    match patient_profile_service::create_profile(&app_state.pool, auth_user.user_id, dto).await {
        Ok(profile) => Ok(Json(ApiResponse::success(
            "Patient profile created successfully",
            profile,
        ))),
        Err(e) => {
            if e.to_string().contains("Invalid ID number") {
                Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::error("Invalid ID number format")),
                ))
            } else if e.to_string().contains("already registered") {
                Err((
                    StatusCode::CONFLICT,
                    Json(ApiResponse::error("This ID number is already registered")),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!(
                        "Failed to create patient profile: {}",
                        e
                    ))),
                ))
            }
        }
    }
}

pub async fn update_profile(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdatePatientProfileDto>,
) -> Result<Json<ApiResponse<PatientProfile>>, (StatusCode, Json<ApiResponse<()>>)> {
    if auth_user.role != "patient" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error(
                "Only patients can manage patient profiles",
            )),
        ));
    }

    dto.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(&format!("Validation error: {}", e))),
        )
    })?;

    match patient_profile_service::update_profile(&app_state.pool, id, auth_user.user_id, dto).await
    {
        Ok(profile) => Ok(Json(ApiResponse::success(
            "Patient profile updated successfully",
            profile,
        ))),
        Err(e) => {
            if e.to_string().contains("not found") {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::error("Patient profile not found")),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!(
                        "Failed to update patient profile: {}",
                        e
                    ))),
                ))
            }
        }
    }
}

pub async fn delete_profile(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    if auth_user.role != "patient" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error(
                "Only patients can manage patient profiles",
            )),
        ));
    }

    match patient_profile_service::delete_profile(&app_state.pool, id, auth_user.user_id).await {
        Ok(_) => Ok(Json(ApiResponse::success(
            "Patient profile deleted successfully",
            (),
        ))),
        Err(e) => {
            if e.to_string().contains("not found") {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::error("Patient profile not found")),
                ))
            } else if e.to_string().contains("Cannot delete self profile") {
                Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::error("Cannot delete self profile")),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!(
                        "Failed to delete patient profile: {}",
                        e
                    ))),
                ))
            }
        }
    }
}

pub async fn set_default(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    if auth_user.role != "patient" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error(
                "Only patients can manage patient profiles",
            )),
        ));
    }

    match patient_profile_service::set_default_profile(&app_state.pool, id, auth_user.user_id).await
    {
        Ok(_) => Ok(Json(ApiResponse::success(
            "Default profile set successfully",
            (),
        ))),
        Err(e) => {
            if e.to_string().contains("not found") {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::error("Patient profile not found")),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!(
                        "Failed to set default profile: {}",
                        e
                    ))),
                ))
            }
        }
    }
}
