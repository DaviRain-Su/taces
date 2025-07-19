use crate::config::database::DbPool;
use crate::middleware::auth::AuthUser;
use crate::models::video_consultation::*;
use crate::models::{ApiResponse, UserRole};
use crate::services::video_consultation_service::VideoConsultationService;
use crate::utils::errors::AppError;
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use serde_json::json;
use uuid::Uuid;

pub async fn create_consultation(
    State(db): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(dto): Json<CreateVideoConsultationDto>,
) -> Result<impl IntoResponse, AppError> {
    // Verify user is a doctor or admin
    if auth_user.role != "doctor" && auth_user.role != "admin" {
        return Err(AppError::Forbidden);
    }

    let consultation = VideoConsultationService::create_consultation(&db, dto).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success("视频问诊创建成功", consultation)),
    ))
}

pub async fn get_consultation(
    State(db): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(consultation_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let consultation = VideoConsultationService::get_consultation(&db, consultation_id).await?;

    // Check authorization
    if auth_user.role != "admin" 
        && auth_user.user_id != consultation.doctor_id 
        && auth_user.user_id != consultation.patient_id {
        return Err(AppError::Forbidden);
    }

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success("获取视频问诊成功", consultation)),
    ))
}

pub async fn list_consultations(
    State(db): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<ConsultationListQuery>,
) -> Result<impl IntoResponse, AppError> {
    // For patients, only show their own consultations
    let mut query_params = query;
    if auth_user.role == "patient" {
        query_params.patient_id = Some(auth_user.user_id);
    }

    let consultations = VideoConsultationService::list_consultations(&db, query_params).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success("获取视频问诊列表成功", consultations)),
    ))
}

pub async fn join_room(
    State(db): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(room_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let response = VideoConsultationService::join_room(&db, &room_id, auth_user.user_id).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success("加入房间成功", response)),
    ))
}

pub async fn start_consultation(
    State(db): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(consultation_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    // Only doctors can start consultations
    if auth_user.role != "doctor" {
        return Err(AppError::Forbidden);
    }

    VideoConsultationService::start_consultation(&db, consultation_id, auth_user.user_id).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success("问诊已开始", json!({}))),
    ))
}

pub async fn end_consultation(
    State(db): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(consultation_id): Path<Uuid>,
    Json(dto): Json<CompleteConsultationDto>,
) -> Result<impl IntoResponse, AppError> {
    // Only doctors can end consultations
    if auth_user.role != "doctor" {
        return Err(AppError::Forbidden);
    }

    VideoConsultationService::end_consultation(&db, consultation_id, auth_user.user_id, dto).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success("问诊已结束", json!({}))),
    ))
}

pub async fn update_consultation(
    State(db): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(consultation_id): Path<Uuid>,
    Json(dto): Json<UpdateConsultationDto>,
) -> Result<impl IntoResponse, AppError> {
    // Only doctors can update consultations
    if auth_user.role != "doctor" {
        return Err(AppError::Forbidden);
    }

    VideoConsultationService::update_consultation(&db, consultation_id, auth_user.user_id, dto).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success("问诊信息已更新", json!({}))),
    ))
}

pub async fn rate_consultation(
    State(db): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(consultation_id): Path<Uuid>,
    Json(dto): Json<RateConsultationDto>,
) -> Result<impl IntoResponse, AppError> {
    // Only patients can rate consultations
    if auth_user.role != "patient" {
        return Err(AppError::Forbidden);
    }

    VideoConsultationService::rate_consultation(&db, consultation_id, auth_user.user_id, dto).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success("评价已提交", json!({}))),
    ))
}

// WebRTC Signaling
pub async fn send_signal(
    State(db): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(dto): Json<SendSignalDto>,
) -> Result<impl IntoResponse, AppError> {
    VideoConsultationService::send_signal(&db, auth_user.user_id, dto).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success("信令已发送", json!({}))),
    ))
}

pub async fn receive_signals(
    State(db): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(room_id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let signals = VideoConsultationService::receive_signals(&db, &room_id, auth_user.user_id).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success("获取信令成功", signals)),
    ))
}

// Recording Management
pub async fn start_recording(
    State(db): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(consultation_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    // Only doctors can start recording
    if auth_user.role != "doctor" {
        return Err(AppError::Forbidden);
    }

    let recording = VideoConsultationService::start_recording(&db, consultation_id).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success("录制已开始", recording)),
    ))
}

pub async fn complete_recording(
    State(db): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(recording_id): Path<Uuid>,
    Json(dto): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    // Only system or admin can complete recording
    if auth_user.role != "admin" {
        return Err(AppError::Forbidden);
    }

    let recording_url = dto["recording_url"].as_str().unwrap_or("").to_string();
    let file_size = dto["file_size"].as_i64().unwrap_or(0);
    let duration = dto["duration"].as_i64().unwrap_or(0) as i32;

    VideoConsultationService::complete_recording(&db, recording_id, recording_url, file_size, duration).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success("录制已完成", json!({}))),
    ))
}

pub async fn get_recording(
    State(db): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(recording_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let recording = VideoConsultationService::get_recording(&db, recording_id).await?;

    // Get consultation to check authorization
    let consultation = VideoConsultationService::get_consultation(&db, recording.consultation_id).await?;
    
    if auth_user.role != "admin" 
        && auth_user.user_id != consultation.doctor_id 
        && auth_user.user_id != consultation.patient_id {
        return Err(AppError::Forbidden);
    }

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success("获取录制记录成功", recording)),
    ))
}

pub async fn get_consultation_recordings(
    State(db): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(consultation_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    // Check authorization
    let consultation = VideoConsultationService::get_consultation(&db, consultation_id).await?;
    
    if auth_user.role != "admin" 
        && auth_user.user_id != consultation.doctor_id 
        && auth_user.user_id != consultation.patient_id {
        return Err(AppError::Forbidden);
    }

    let recordings = VideoConsultationService::get_consultation_recordings(&db, consultation_id).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success("获取录制列表成功", recordings)),
    ))
}

// Template Management
pub async fn create_template(
    State(db): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Json(dto): Json<CreateConsultationTemplateDto>,
) -> Result<impl IntoResponse, AppError> {
    // Only doctors can create templates
    if auth_user.role != "doctor" {
        return Err(AppError::Forbidden);
    }

    let template = VideoConsultationService::create_template(&db, auth_user.user_id, dto).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success("模板创建成功", template)),
    ))
}

pub async fn get_template(
    State(db): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(template_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let template = VideoConsultationService::get_template(&db, template_id).await?;

    // Only the owner can view the template
    if auth_user.user_id != template.doctor_id {
        return Err(AppError::Forbidden);
    }

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success("获取模板成功", template)),
    ))
}

pub async fn list_doctor_templates(
    State(db): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
) -> Result<impl IntoResponse, AppError> {
    // Only doctors can view their templates
    if auth_user.role != "doctor" {
        return Err(AppError::Forbidden);
    }

    let templates = VideoConsultationService::list_doctor_templates(&db, auth_user.user_id).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success("获取模板列表成功", templates)),
    ))
}

pub async fn use_template(
    State(db): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Path(template_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    // Only doctors can use templates
    if auth_user.role != "doctor" {
        return Err(AppError::Forbidden);
    }

    let template = VideoConsultationService::use_template(&db, template_id, auth_user.user_id).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success("模板使用成功", template)),
    ))
}

// Statistics
pub async fn get_consultation_statistics(
    State(db): State<DbPool>,
    Extension(auth_user): Extension<AuthUser>,
    Query(params): Query<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    let doctor_id = if auth_user.role == "doctor" {
        Some(auth_user.user_id)
    } else if auth_user.role == "admin" {
        params["doctor_id"].as_str().and_then(|s| Uuid::parse_str(s).ok())
    } else {
        return Err(AppError::Forbidden);
    };

    let start_date = params["start_date"].as_str()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));

    let end_date = params["end_date"].as_str()
        .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
        .map(|dt| dt.with_timezone(&chrono::Utc));

    let stats = VideoConsultationService::get_consultation_statistics(&db, doctor_id, start_date, end_date).await?;

    Ok((
        StatusCode::OK,
        Json(ApiResponse::success("获取统计数据成功", stats)),
    ))
}