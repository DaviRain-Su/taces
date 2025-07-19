use crate::{
    middleware::auth::AuthUser,
    models::{patient_group::*, ApiResponse},
    services::patient_group_service,
    AppState,
};
use axum::{
    extract::{Path, State},
    http::StatusCode,
    Extension, Json,
};
use uuid::Uuid;
use validator::Validate;

pub async fn list_groups(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
) -> Result<Json<ApiResponse<Vec<PatientGroup>>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Only doctors can manage patient groups
    if auth_user.role != "doctor" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Only doctors can manage patient groups")),
        ));
    }

    // Get doctor_id from doctors table
    let doctor_id = match get_doctor_id(&app_state.pool, auth_user.user_id).await {
        Ok(id) => id,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&format!(
                    "Failed to get doctor profile: {}",
                    e
                ))),
            ))
        }
    };

    match patient_group_service::list_doctor_groups(&app_state.pool, doctor_id).await {
        Ok(groups) => Ok(Json(ApiResponse::success(
            "Patient groups retrieved successfully",
            groups,
        ))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!(
                "Failed to retrieve patient groups: {}",
                e
            ))),
        )),
    }
}

pub async fn get_group(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<PatientGroupWithMembers>>, (StatusCode, Json<ApiResponse<()>>)> {
    if auth_user.role != "doctor" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Only doctors can manage patient groups")),
        ));
    }

    let doctor_id = match get_doctor_id(&app_state.pool, auth_user.user_id).await {
        Ok(id) => id,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&format!(
                    "Failed to get doctor profile: {}",
                    e
                ))),
            ))
        }
    };

    match patient_group_service::get_group_by_id(&app_state.pool, id, doctor_id).await {
        Ok(group) => Ok(Json(ApiResponse::success(
            "Patient group retrieved successfully",
            group,
        ))),
        Err(e) => {
            if e.to_string().contains("not found") {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::error("Patient group not found")),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!(
                        "Failed to retrieve patient group: {}",
                        e
                    ))),
                ))
            }
        }
    }
}

pub async fn create_group(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Json(dto): Json<CreatePatientGroupDto>,
) -> Result<Json<ApiResponse<PatientGroup>>, (StatusCode, Json<ApiResponse<()>>)> {
    if auth_user.role != "doctor" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Only doctors can manage patient groups")),
        ));
    }

    dto.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(&format!("Validation error: {}", e))),
        )
    })?;

    let doctor_id = match get_doctor_id(&app_state.pool, auth_user.user_id).await {
        Ok(id) => id,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&format!(
                    "Failed to get doctor profile: {}",
                    e
                ))),
            ))
        }
    };

    match patient_group_service::create_group(&app_state.pool, doctor_id, dto).await {
        Ok(group) => Ok(Json(ApiResponse::success(
            "Patient group created successfully",
            group,
        ))),
        Err(e) => {
            if e.to_string().contains("maximum 5") {
                Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::error("Maximum 5 patient groups allowed")),
                ))
            } else if e.to_string().contains("already exists") {
                Err((
                    StatusCode::CONFLICT,
                    Json(ApiResponse::error("Group name already exists")),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!(
                        "Failed to create patient group: {}",
                        e
                    ))),
                ))
            }
        }
    }
}

pub async fn update_group(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdatePatientGroupDto>,
) -> Result<Json<ApiResponse<PatientGroup>>, (StatusCode, Json<ApiResponse<()>>)> {
    if auth_user.role != "doctor" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Only doctors can manage patient groups")),
        ));
    }

    dto.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(&format!("Validation error: {}", e))),
        )
    })?;

    let doctor_id = match get_doctor_id(&app_state.pool, auth_user.user_id).await {
        Ok(id) => id,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&format!(
                    "Failed to get doctor profile: {}",
                    e
                ))),
            ))
        }
    };

    match patient_group_service::update_group(&app_state.pool, id, doctor_id, dto).await {
        Ok(group) => Ok(Json(ApiResponse::success(
            "Patient group updated successfully",
            group,
        ))),
        Err(e) => {
            if e.to_string().contains("not found") {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::error("Patient group not found")),
                ))
            } else if e.to_string().contains("already exists") {
                Err((
                    StatusCode::CONFLICT,
                    Json(ApiResponse::error("Group name already exists")),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!(
                        "Failed to update patient group: {}",
                        e
                    ))),
                ))
            }
        }
    }
}

pub async fn delete_group(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    if auth_user.role != "doctor" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Only doctors can manage patient groups")),
        ));
    }

    let doctor_id = match get_doctor_id(&app_state.pool, auth_user.user_id).await {
        Ok(id) => id,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&format!(
                    "Failed to get doctor profile: {}",
                    e
                ))),
            ))
        }
    };

    match patient_group_service::delete_group(&app_state.pool, id, doctor_id).await {
        Ok(_) => Ok(Json(ApiResponse::success(
            "Patient group deleted successfully",
            (),
        ))),
        Err(e) => {
            if e.to_string().contains("not found") {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::error("Patient group not found")),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!(
                        "Failed to delete patient group: {}",
                        e
                    ))),
                ))
            }
        }
    }
}

pub async fn add_members(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<AddMembersDto>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    if auth_user.role != "doctor" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Only doctors can manage patient groups")),
        ));
    }

    let doctor_id = match get_doctor_id(&app_state.pool, auth_user.user_id).await {
        Ok(id) => id,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&format!(
                    "Failed to get doctor profile: {}",
                    e
                ))),
            ))
        }
    };

    match patient_group_service::add_members(&app_state.pool, id, doctor_id, dto.patient_ids).await
    {
        Ok(_) => Ok(Json(ApiResponse::success("Members added successfully", ()))),
        Err(e) => {
            if e.to_string().contains("not found") {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::error(&e.to_string())),
                ))
            } else if e.to_string().contains("not a patient") {
                Err((
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::error(&e.to_string())),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!("Failed to add members: {}", e))),
                ))
            }
        }
    }
}

pub async fn remove_members(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<RemoveMembersDto>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    if auth_user.role != "doctor" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Only doctors can manage patient groups")),
        ));
    }

    let doctor_id = match get_doctor_id(&app_state.pool, auth_user.user_id).await {
        Ok(id) => id,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&format!(
                    "Failed to get doctor profile: {}",
                    e
                ))),
            ))
        }
    };

    match patient_group_service::remove_members(&app_state.pool, id, doctor_id, dto.patient_ids)
        .await
    {
        Ok(_) => Ok(Json(ApiResponse::success(
            "Members removed successfully",
            (),
        ))),
        Err(e) => {
            if e.to_string().contains("not found") {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::error("Patient group not found")),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!(
                        "Failed to remove members: {}",
                        e
                    ))),
                ))
            }
        }
    }
}

pub async fn send_message(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<GroupMessageDto>,
) -> Result<Json<ApiResponse<Vec<String>>>, (StatusCode, Json<ApiResponse<()>>)> {
    if auth_user.role != "doctor" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Only doctors can manage patient groups")),
        ));
    }

    dto.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(&format!("Validation error: {}", e))),
        )
    })?;

    let doctor_id = match get_doctor_id(&app_state.pool, auth_user.user_id).await {
        Ok(id) => id,
        Err(e) => {
            return Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&format!(
                    "Failed to get doctor profile: {}",
                    e
                ))),
            ))
        }
    };

    match patient_group_service::send_group_message(&app_state.pool, id, doctor_id, &dto.message)
        .await
    {
        Ok(phone_numbers) => Ok(Json(ApiResponse::success(
            &format!("Message would be sent to {} members", phone_numbers.len()),
            phone_numbers,
        ))),
        Err(e) => {
            if e.to_string().contains("No members") {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::error("No members in group or group not found")),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!(
                        "Failed to send message: {}",
                        e
                    ))),
                ))
            }
        }
    }
}

// Helper function to get doctor_id from user_id
async fn get_doctor_id(
    pool: &crate::config::database::DbPool,
    user_id: Uuid,
) -> Result<Uuid, anyhow::Error> {
    let query = "SELECT id FROM doctors WHERE user_id = ?";
    let row = sqlx::query(query)
        .bind(user_id.to_string())
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow::anyhow!("Doctor profile not found: {}", e))?;

    use sqlx::Row;
    let id: String = row.get("id");
    Uuid::parse_str(&id).map_err(|e| anyhow::anyhow!("Failed to parse doctor ID: {}", e))
}
