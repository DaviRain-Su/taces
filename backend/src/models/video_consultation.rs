use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type, PartialEq)]
#[sqlx(type_name = "consultation_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ConsultationStatus {
    Waiting,
    InProgress,
    Completed,
    Cancelled,
    NoShow,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type)]
#[sqlx(type_name = "connection_quality", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ConnectionQuality {
    Excellent,
    Good,
    Fair,
    Poor,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type)]
#[sqlx(type_name = "signal_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum SignalType {
    Offer,
    Answer,
    IceCandidate,
    Join,
    Leave,
    Error,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type)]
#[sqlx(type_name = "recording_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum RecordingStatus {
    Recording,
    Processing,
    Completed,
    Failed,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type)]
#[sqlx(type_name = "video_event_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum VideoEventType {
    Joined,
    Left,
    Reconnected,
    Disconnected,
    CameraOn,
    CameraOff,
    MicOn,
    MicOff,
    ScreenShareStart,
    ScreenShareEnd,
    RecordingStart,
    RecordingEnd,
    NetworkPoor,
    NetworkRecovered,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct VideoConsultation {
    pub id: Uuid,
    pub appointment_id: Uuid,
    pub doctor_id: Uuid,
    pub patient_id: Uuid,
    pub room_id: String,
    pub status: ConsultationStatus,
    pub scheduled_start_time: DateTime<Utc>,
    pub actual_start_time: Option<DateTime<Utc>>,
    pub end_time: Option<DateTime<Utc>>,
    pub duration: Option<i32>,
    pub doctor_token: Option<String>,
    pub patient_token: Option<String>,
    pub ice_servers: Option<serde_json::Value>,
    pub chief_complaint: Option<String>,
    pub diagnosis: Option<String>,
    pub treatment_plan: Option<String>,
    pub notes: Option<String>,
    pub connection_quality: Option<ConnectionQuality>,
    pub patient_rating: Option<i32>,
    pub patient_feedback: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateVideoConsultationDto {
    pub appointment_id: Uuid,
    pub doctor_id: Uuid,
    pub patient_id: Uuid,
    pub scheduled_start_time: DateTime<Utc>,
    #[validate(length(max = 500))]
    pub chief_complaint: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateConsultationDto {
    #[validate(length(max = 500))]
    pub chief_complaint: Option<String>,
    #[validate(length(max = 1000))]
    pub diagnosis: Option<String>,
    #[validate(length(max = 1000))]
    pub treatment_plan: Option<String>,
    #[validate(length(max = 2000))]
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CompleteConsultationDto {
    #[validate(length(min = 1, max = 1000))]
    pub diagnosis: String,
    #[validate(length(max = 1000))]
    pub treatment_plan: Option<String>,
    #[validate(length(max = 2000))]
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct RateConsultationDto {
    #[validate(range(min = 1, max = 5))]
    pub rating: i32,
    #[validate(length(max = 500))]
    pub feedback: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct VideoRecording {
    pub id: Uuid,
    pub consultation_id: Uuid,
    pub recording_url: Option<String>,
    pub thumbnail_url: Option<String>,
    pub file_size: Option<i64>,
    pub duration: Option<i32>,
    pub format: Option<String>,
    pub status: RecordingStatus,
    pub error_message: Option<String>,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct WebRTCSignal {
    pub id: Uuid,
    pub room_id: String,
    pub from_user_id: Uuid,
    pub to_user_id: Uuid,
    pub signal_type: SignalType,
    pub payload: serde_json::Value,
    pub delivered: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct SendSignalDto {
    #[validate(length(min = 1, max = 100))]
    pub room_id: String,
    pub to_user_id: Uuid,
    pub signal_type: SignalType,
    pub payload: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct VideoCallEvent {
    pub id: Uuid,
    pub consultation_id: Uuid,
    pub user_id: Uuid,
    pub event_type: VideoEventType,
    pub event_data: Option<serde_json::Value>,
    pub ip_address: Option<String>,
    pub user_agent: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct LogEventDto {
    pub consultation_id: Uuid,
    pub event_type: VideoEventType,
    pub event_data: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct VideoConsultationTemplate {
    pub id: Uuid,
    pub doctor_id: Uuid,
    pub name: String,
    pub chief_complaint: Option<String>,
    pub diagnosis: Option<String>,
    pub treatment_plan: Option<String>,
    pub notes: Option<String>,
    pub usage_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateConsultationTemplateDto {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(max = 500))]
    pub chief_complaint: Option<String>,
    #[validate(length(max = 1000))]
    pub diagnosis: Option<String>,
    #[validate(length(max = 1000))]
    pub treatment_plan: Option<String>,
    #[validate(length(max = 2000))]
    pub notes: Option<String>,
}

// Response DTOs
#[derive(Debug, Serialize, Deserialize)]
pub struct VideoConsultationResponse {
    pub consultation: VideoConsultation,
    pub doctor_name: String,
    pub patient_name: String,
    pub can_start: bool,
    pub can_join: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JoinRoomResponse {
    pub room_id: String,
    pub token: String,
    pub ice_servers: serde_json::Value,
    pub role: String, // "doctor" or "patient"
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsultationListQuery {
    pub doctor_id: Option<Uuid>,
    pub patient_id: Option<Uuid>,
    pub status: Option<ConsultationStatus>,
    pub date_from: Option<DateTime<Utc>>,
    pub date_to: Option<DateTime<Utc>>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ConsultationStatistics {
    pub total_consultations: i64,
    pub completed_consultations: i64,
    pub average_duration: Option<f64>,
    pub average_rating: Option<f64>,
    pub no_show_rate: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IceServer {
    pub urls: Vec<String>,
    pub username: Option<String>,
    pub credential: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RoomConfig {
    pub ice_servers: Vec<IceServer>,
    pub video_codec: String,
    pub audio_codec: String,
    pub max_video_bitrate: i32,
    pub max_audio_bitrate: i32,
    pub recording_enabled: bool,
}

// WebRTC Signaling Messages
#[derive(Debug, Serialize, Deserialize)]
pub struct OfferSignal {
    pub sdp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnswerSignal {
    pub sdp: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct IceCandidateSignal {
    pub candidate: String,
    pub sdp_mid: Option<String>,
    pub sdp_mline_index: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorSignal {
    pub code: String,
    pub message: String,
}
