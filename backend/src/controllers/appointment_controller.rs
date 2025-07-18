use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Json,
    Extension,
};
use uuid::Uuid;
use validator::Validate;
use serde::Deserialize;
use chrono::{DateTime, Utc};
use crate::{
    config::database::DbPool,
    models::{appointment::*, ApiResponse},
    services::appointment_service,
    middleware::auth::AuthUser,
};

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    page: Option<u32>,
    per_page: Option<u32>,
    status: Option<String>,
    date_from: Option<DateTime<Utc>>,
    date_to: Option<DateTime<Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct AvailableSlotsQuery {
    doctor_id: Uuid,
    date: DateTime<Utc>,
}

pub async fn list_appointments(
    Extension(auth_user): Extension<AuthUser>,
    State(pool): State<DbPool>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ApiResponse<Vec<Appointment>>>, (StatusCode, Json<ApiResponse<()>>)> {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);
    
    match appointment_service::list_appointments(&pool, page, per_page, query.status, query.date_from, query.date_to).await {
        Ok(appointments) => Ok(Json(ApiResponse::success("Appointments retrieved successfully", appointments))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Failed to retrieve appointments: {}", e))),
        )),
    }
}

pub async fn get_appointment(
    Extension(auth_user): Extension<AuthUser>,
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Appointment>>, (StatusCode, Json<ApiResponse<()>>)> {
    let appointment = match appointment_service::get_appointment_by_id(&pool, id).await {
        Ok(apt) => apt,
        Err(e) => return Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(&format!("Appointment not found: {}", e))),
        )),
    };
    
    // Check if user has permission to view this appointment
    if auth_user.user_id != appointment.patient_id && auth_user.role != "admin" {
        // Check if user is the doctor
        let doctor = appointment_service::get_doctor_user_id(&pool, appointment.doctor_id).await.ok();
        if doctor != Some(auth_user.user_id) {
            return Err((
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("Insufficient permissions")),
            ));
        }
    }
    
    Ok(Json(ApiResponse::success("Appointment retrieved successfully", appointment)))
}

pub async fn create_appointment(
    Extension(auth_user): Extension<AuthUser>,
    State(pool): State<DbPool>,
    Json(mut dto): Json<CreateAppointmentDto>,
) -> Result<Json<ApiResponse<Appointment>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Patients create their own appointments
    if auth_user.role == "patient" {
        dto.patient_id = auth_user.user_id;
    } else if auth_user.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Only patients can create appointments")),
        ));
    }
    
    dto.validate()
        .map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(ApiResponse::error(&format!("Validation error: {}", e))),
            )
        })?;
    
    match appointment_service::create_appointment(&pool, dto).await {
        Ok(appointment) => Ok(Json(ApiResponse::success("Appointment created successfully", appointment))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Failed to create appointment: {}", e))),
        )),
    }
}

pub async fn update_appointment(
    Extension(auth_user): Extension<AuthUser>,
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateAppointmentDto>,
) -> Result<Json<ApiResponse<Appointment>>, (StatusCode, Json<ApiResponse<()>>)> {
    let appointment = match appointment_service::get_appointment_by_id(&pool, id).await {
        Ok(apt) => apt,
        Err(_) => return Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Appointment not found")),
        )),
    };
    
    // Check permissions
    if auth_user.role != "admin" {
        let doctor_user_id = appointment_service::get_doctor_user_id(&pool, appointment.doctor_id).await.ok();
        if auth_user.user_id != appointment.patient_id && doctor_user_id != Some(auth_user.user_id) {
            return Err((
                StatusCode::FORBIDDEN,
                Json(ApiResponse::error("Insufficient permissions")),
            ));
        }
    }
    
    match appointment_service::update_appointment(&pool, id, dto).await {
        Ok(appointment) => Ok(Json(ApiResponse::success("Appointment updated successfully", appointment))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Failed to update appointment: {}", e))),
        )),
    }
}

pub async fn cancel_appointment(
    Extension(auth_user): Extension<AuthUser>,
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Appointment>>, (StatusCode, Json<ApiResponse<()>>)> {
    let appointment = match appointment_service::get_appointment_by_id(&pool, id).await {
        Ok(apt) => apt,
        Err(_) => return Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error("Appointment not found")),
        )),
    };
    
    // Check permissions
    if auth_user.user_id != appointment.patient_id && auth_user.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Only the patient or admin can cancel appointments")),
        ));
    }
    
    match appointment_service::cancel_appointment(&pool, id).await {
        Ok(appointment) => Ok(Json(ApiResponse::success("Appointment cancelled successfully", appointment))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Failed to cancel appointment: {}", e))),
        )),
    }
}

pub async fn get_doctor_appointments(
    Extension(auth_user): Extension<AuthUser>,
    State(pool): State<DbPool>,
    Path(doctor_id): Path<Uuid>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ApiResponse<Vec<Appointment>>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Check if user is the doctor or admin
    let doctor_user_id = appointment_service::get_doctor_user_id(&pool, doctor_id).await.ok();
    if auth_user.role != "admin" && doctor_user_id != Some(auth_user.user_id) {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Insufficient permissions")),
        ));
    }
    
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);
    
    match appointment_service::get_doctor_appointments(&pool, doctor_id, page, per_page, query.status).await {
        Ok(appointments) => Ok(Json(ApiResponse::success("Doctor appointments retrieved successfully", appointments))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Failed to retrieve doctor appointments: {}", e))),
        )),
    }
}

pub async fn get_patient_appointments(
    Extension(auth_user): Extension<AuthUser>,
    State(pool): State<DbPool>,
    Path(patient_id): Path<Uuid>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ApiResponse<Vec<Appointment>>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Users can view their own appointments, admins can view any
    if auth_user.user_id != patient_id && auth_user.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Insufficient permissions")),
        ));
    }
    
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);
    
    match appointment_service::get_patient_appointments(&pool, patient_id, page, per_page, query.status).await {
        Ok(appointments) => Ok(Json(ApiResponse::success("Patient appointments retrieved successfully", appointments))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Failed to retrieve patient appointments: {}", e))),
        )),
    }
}

pub async fn get_available_slots(
    State(pool): State<DbPool>,
    Query(query): Query<AvailableSlotsQuery>,
) -> Result<Json<ApiResponse<Vec<String>>>, (StatusCode, Json<ApiResponse<()>>)> {
    match appointment_service::get_available_slots(&pool, query.doctor_id, query.date).await {
        Ok(slots) => Ok(Json(ApiResponse::success("Available slots retrieved successfully", slots))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Failed to retrieve available slots: {}", e))),
        )),
    }
}