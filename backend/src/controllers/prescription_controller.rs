use crate::{
    middleware::auth::AuthUser,
    models::{prescription::*, ApiResponse},
    services::prescription_service,
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
}

pub async fn list_prescriptions(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ApiResponse<Vec<Prescription>>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Only admin can list all prescriptions
    if auth_user.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Insufficient permissions")),
        ));
    }

    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);

    match prescription_service::list_prescriptions(&app_state.pool, page, per_page, query.search)
        .await
    {
        Ok(prescriptions) => Ok(Json(ApiResponse::success(
            "Prescriptions retrieved successfully",
            prescriptions,
        ))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!(
                "Failed to retrieve prescriptions: {}",
                e
            ))),
        )),
    }
}

pub async fn get_prescription(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Prescription>>, (StatusCode, Json<ApiResponse<()>>)> {
    let prescription = match prescription_service::get_prescription_by_id(&app_state.pool, id).await
    {
        Ok(p) => p,
        Err(e) => {
            return Err((
                StatusCode::NOT_FOUND,
                Json(ApiResponse::error(&format!(
                    "Prescription not found: {}",
                    e
                ))),
            ))
        }
    };

    // Check permissions
    if auth_user.user_id != prescription.patient_id && auth_user.role != "admin" {
        // Check if user is the doctor
        let doctor_user_id =
            prescription_service::get_doctor_user_id(&app_state.pool, prescription.doctor_id)
                .await
                .ok();
        if doctor_user_id != Some(auth_user.user_id) {
            return Err((
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("Insufficient permissions")),
            ));
        }
    }

    Ok(Json(ApiResponse::success(
        "Prescription retrieved successfully",
        prescription,
    )))
}

pub async fn get_prescription_by_code(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Path(code): Path<String>,
) -> Result<Json<ApiResponse<Prescription>>, (StatusCode, Json<ApiResponse<()>>)> {
    let prescription =
        match prescription_service::get_prescription_by_code(&app_state.pool, &code).await {
            Ok(p) => p,
            Err(e) => {
                return Err((
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::error(&format!(
                        "Prescription not found: {}",
                        e
                    ))),
                ))
            }
        };

    // Check permissions
    if auth_user.user_id != prescription.patient_id && auth_user.role != "admin" {
        // Check if user is the doctor
        let doctor_user_id =
            prescription_service::get_doctor_user_id(&app_state.pool, prescription.doctor_id)
                .await
                .ok();
        if doctor_user_id != Some(auth_user.user_id) {
            return Err((
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("Insufficient permissions")),
            ));
        }
    }

    Ok(Json(ApiResponse::success(
        "Prescription retrieved successfully",
        prescription,
    )))
}

pub async fn create_prescription(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Json(mut dto): Json<CreatePrescriptionDto>,
) -> Result<Json<ApiResponse<Prescription>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Only doctors can create prescriptions
    if auth_user.role != "doctor" && auth_user.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Only doctors can create prescriptions")),
        ));
    }

    // If user is a doctor, verify they are creating prescription with their own doctor_id
    if auth_user.role == "doctor" {
        let doctor =
            match prescription_service::get_doctor_by_user_id(&app_state.pool, auth_user.user_id)
                .await
            {
                Ok(d) => d,
                Err(_) => {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        Json(ApiResponse::error("Doctor profile not found")),
                    ))
                }
            };
        dto.doctor_id = doctor.id;
    }

    dto.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(&format!("Validation error: {}", e))),
        )
    })?;

    match prescription_service::create_prescription(&app_state.pool, dto).await {
        Ok(prescription) => Ok(Json(ApiResponse::success(
            "Prescription created successfully",
            prescription,
        ))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!(
                "Failed to create prescription: {}",
                e
            ))),
        )),
    }
}

pub async fn get_doctor_prescriptions(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Path(doctor_id): Path<Uuid>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ApiResponse<Vec<Prescription>>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Check if user is the doctor or admin
    let doctor_user_id = prescription_service::get_doctor_user_id(&app_state.pool, doctor_id)
        .await
        .ok();
    if auth_user.role != "admin" && doctor_user_id != Some(auth_user.user_id) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Insufficient permissions")),
        ));
    }

    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);

    match prescription_service::get_doctor_prescriptions(&app_state.pool, doctor_id, page, per_page)
        .await
    {
        Ok(prescriptions) => Ok(Json(ApiResponse::success(
            "Doctor prescriptions retrieved successfully",
            prescriptions,
        ))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!(
                "Failed to retrieve doctor prescriptions: {}",
                e
            ))),
        )),
    }
}

pub async fn get_patient_prescriptions(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Path(patient_id): Path<Uuid>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ApiResponse<Vec<Prescription>>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Users can view their own prescriptions, admins can view any
    if auth_user.user_id != patient_id && auth_user.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Insufficient permissions")),
        ));
    }

    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);

    match prescription_service::get_patient_prescriptions(
        &app_state.pool,
        patient_id,
        page,
        per_page,
    )
    .await
    {
        Ok(prescriptions) => Ok(Json(ApiResponse::success(
            "Patient prescriptions retrieved successfully",
            prescriptions,
        ))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!(
                "Failed to retrieve patient prescriptions: {}",
                e
            ))),
        )),
    }
}
