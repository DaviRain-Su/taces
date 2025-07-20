use crate::middleware::auth::AuthUser;
use crate::models::{
    ApiResponse, CreateCommonPhraseDto, CreatePrescriptionTemplateDto, TemplateQuery,
    UpdateCommonPhraseDto, UpdatePrescriptionTemplateDto,
};
use crate::services::template_service::TemplateService;
use crate::AppState;
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::Json,
};
use uuid::Uuid;
use validator::Validate;

// ========== 常用语相关端点 ==========

pub async fn create_common_phrase(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Json(dto): Json<CreateCommonPhraseDto>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    // 只有医生可以创建常用语
    if auth_user.role != "doctor" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Only doctors can create common phrases")),
        ));
    }

    // 验证输入
    if let Err(e) = dto.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(&format!("Validation error: {}", e))),
        ));
    }

    // 获取医生ID
    let doctor_id = TemplateService::verify_doctor_access(&state.pool, auth_user.user_id)
        .await
        .map_err(|_| {
            (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("Doctor profile not found")),
            )
        })?;

    let phrase = TemplateService::create_common_phrase(&state.pool, doctor_id, dto)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&format!(
                    "Failed to create phrase: {}",
                    e
                ))),
            )
        })?;

    Ok(Json(ApiResponse::success(
        "Common phrase created successfully",
        serde_json::to_value(&phrase).unwrap(),
    )))
}

pub async fn get_common_phrases(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Query(query): Query<TemplateQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    // 只有医生可以查看常用语
    if auth_user.role != "doctor" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Only doctors can view common phrases")),
        ));
    }

    let doctor_id = TemplateService::verify_doctor_access(&state.pool, auth_user.user_id)
        .await
        .map_err(|_| {
            (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("Doctor profile not found")),
            )
        })?;

    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(10).min(100);

    let (phrases, total) = TemplateService::get_common_phrases(
        &state.pool,
        doctor_id,
        query.category,
        query.search,
        query.is_active,
        page,
        page_size,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Failed to get phrases: {}", e))),
        )
    })?;

    Ok(Json(ApiResponse::success(
        "Common phrases retrieved successfully",
        serde_json::json!({
            "phrases": phrases,
            "pagination": {
                "page": page,
                "page_size": page_size,
                "total": total,
                "total_pages": (total as f64 / page_size as f64).ceil() as i64,
            }
        }),
    )))
}

pub async fn get_common_phrase_by_id(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    // 只有医生可以查看常用语
    if auth_user.role != "doctor" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Only doctors can view common phrases")),
        ));
    }

    let phrase = TemplateService::get_common_phrase_by_id(&state.pool, id)
        .await
        .map_err(|e| {
            if e.to_string().contains("not found") {
                (
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::error("Common phrase not found")),
                )
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!("Failed to get phrase: {}", e))),
                )
            }
        })?;

    Ok(Json(ApiResponse::success(
        "Common phrase retrieved successfully",
        serde_json::to_value(&phrase).unwrap(),
    )))
}

pub async fn update_common_phrase(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateCommonPhraseDto>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    // 只有医生可以更新常用语
    if auth_user.role != "doctor" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Only doctors can update common phrases")),
        ));
    }

    // 验证输入
    if let Err(e) = dto.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(&format!("Validation error: {}", e))),
        ));
    }

    let doctor_id = TemplateService::verify_doctor_access(&state.pool, auth_user.user_id)
        .await
        .map_err(|_| {
            (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("Doctor profile not found")),
            )
        })?;

    let phrase = TemplateService::update_common_phrase(&state.pool, id, doctor_id, dto)
        .await
        .map_err(|e| {
            if e.to_string().contains("No permission") {
                (
                    StatusCode::FORBIDDEN,
                    Json(ApiResponse::error("No permission to update this phrase")),
                )
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!(
                        "Failed to update phrase: {}",
                        e
                    ))),
                )
            }
        })?;

    Ok(Json(ApiResponse::success(
        "Common phrase updated successfully",
        serde_json::to_value(&phrase).unwrap(),
    )))
}

pub async fn delete_common_phrase(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    // 只有医生可以删除常用语
    if auth_user.role != "doctor" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Only doctors can delete common phrases")),
        ));
    }

    let doctor_id = TemplateService::verify_doctor_access(&state.pool, auth_user.user_id)
        .await
        .map_err(|_| {
            (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("Doctor profile not found")),
            )
        })?;

    TemplateService::delete_common_phrase(&state.pool, id, doctor_id)
        .await
        .map_err(|e| {
            if e.to_string().contains("No permission") {
                (
                    StatusCode::FORBIDDEN,
                    Json(ApiResponse::error("No permission to delete this phrase")),
                )
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!(
                        "Failed to delete phrase: {}",
                        e
                    ))),
                )
            }
        })?;

    Ok(Json(ApiResponse::success(
        "Common phrase deleted successfully",
        (),
    )))
}

pub async fn use_common_phrase(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    // 只有医生可以使用常用语
    if auth_user.role != "doctor" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Only doctors can use common phrases")),
        ));
    }

    TemplateService::increment_phrase_usage(&state.pool, id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&format!(
                    "Failed to update usage: {}",
                    e
                ))),
            )
        })?;

    Ok(Json(ApiResponse::success(
        "Usage count updated successfully",
        (),
    )))
}

// ========== 处方模板相关端点 ==========

pub async fn create_prescription_template(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Json(dto): Json<CreatePrescriptionTemplateDto>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    // 只有医生可以创建处方模板
    if auth_user.role != "doctor" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error(
                "Only doctors can create prescription templates",
            )),
        ));
    }

    // 验证输入
    if let Err(e) = dto.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(&format!("Validation error: {}", e))),
        ));
    }

    let doctor_id = TemplateService::verify_doctor_access(&state.pool, auth_user.user_id)
        .await
        .map_err(|_| {
            (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("Doctor profile not found")),
            )
        })?;

    let template = TemplateService::create_prescription_template(&state.pool, doctor_id, dto)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&format!(
                    "Failed to create template: {}",
                    e
                ))),
            )
        })?;

    Ok(Json(ApiResponse::success(
        "Prescription template created successfully",
        serde_json::to_value(&template).unwrap(),
    )))
}

pub async fn get_prescription_templates(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Query(query): Query<TemplateQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    // 只有医生可以查看处方模板
    if auth_user.role != "doctor" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error(
                "Only doctors can view prescription templates",
            )),
        ));
    }

    let doctor_id = TemplateService::verify_doctor_access(&state.pool, auth_user.user_id)
        .await
        .map_err(|_| {
            (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("Doctor profile not found")),
            )
        })?;

    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(10).min(100);

    let (templates, total) = TemplateService::get_prescription_templates(
        &state.pool,
        doctor_id,
        query.search,
        query.is_active,
        page,
        page_size,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!(
                "Failed to get templates: {}",
                e
            ))),
        )
    })?;

    Ok(Json(ApiResponse::success(
        "Prescription templates retrieved successfully",
        serde_json::json!({
            "templates": templates,
            "pagination": {
                "page": page,
                "page_size": page_size,
                "total": total,
                "total_pages": (total as f64 / page_size as f64).ceil() as i64,
            }
        }),
    )))
}

pub async fn get_prescription_template_by_id(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    // 只有医生可以查看处方模板
    if auth_user.role != "doctor" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error(
                "Only doctors can view prescription templates",
            )),
        ));
    }

    let template = TemplateService::get_prescription_template_by_id(&state.pool, id)
        .await
        .map_err(|e| {
            if e.to_string().contains("not found") {
                (
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::error("Prescription template not found")),
                )
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!(
                        "Failed to get template: {}",
                        e
                    ))),
                )
            }
        })?;

    Ok(Json(ApiResponse::success(
        "Prescription template retrieved successfully",
        serde_json::to_value(&template).unwrap(),
    )))
}

pub async fn update_prescription_template(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdatePrescriptionTemplateDto>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    // 只有医生可以更新处方模板
    if auth_user.role != "doctor" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error(
                "Only doctors can update prescription templates",
            )),
        ));
    }

    // 验证输入
    if let Err(e) = dto.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(&format!("Validation error: {}", e))),
        ));
    }

    let doctor_id = TemplateService::verify_doctor_access(&state.pool, auth_user.user_id)
        .await
        .map_err(|_| {
            (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("Doctor profile not found")),
            )
        })?;

    let template = TemplateService::update_prescription_template(&state.pool, id, doctor_id, dto)
        .await
        .map_err(|e| {
            if e.to_string().contains("No permission") {
                (
                    StatusCode::FORBIDDEN,
                    Json(ApiResponse::error("No permission to update this template")),
                )
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!(
                        "Failed to update template: {}",
                        e
                    ))),
                )
            }
        })?;

    Ok(Json(ApiResponse::success(
        "Prescription template updated successfully",
        serde_json::to_value(&template).unwrap(),
    )))
}

pub async fn delete_prescription_template(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    // 只有医生可以删除处方模板
    if auth_user.role != "doctor" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error(
                "Only doctors can delete prescription templates",
            )),
        ));
    }

    let doctor_id = TemplateService::verify_doctor_access(&state.pool, auth_user.user_id)
        .await
        .map_err(|_| {
            (
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("Doctor profile not found")),
            )
        })?;

    TemplateService::delete_prescription_template(&state.pool, id, doctor_id)
        .await
        .map_err(|e| {
            if e.to_string().contains("No permission") {
                (
                    StatusCode::FORBIDDEN,
                    Json(ApiResponse::error("No permission to delete this template")),
                )
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!(
                        "Failed to delete template: {}",
                        e
                    ))),
                )
            }
        })?;

    Ok(Json(ApiResponse::success(
        "Prescription template deleted successfully",
        (),
    )))
}

pub async fn use_prescription_template(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    // 只有医生可以使用处方模板
    if auth_user.role != "doctor" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error(
                "Only doctors can use prescription templates",
            )),
        ));
    }

    TemplateService::increment_template_usage(&state.pool, id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&format!(
                    "Failed to update usage: {}",
                    e
                ))),
            )
        })?;

    Ok(Json(ApiResponse::success(
        "Usage count updated successfully",
        (),
    )))
}
