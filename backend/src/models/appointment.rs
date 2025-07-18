use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use validator::Validate;
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Appointment {
    pub id: Uuid,
    pub patient_id: Uuid,
    pub doctor_id: Uuid,
    pub appointment_date: DateTime<Utc>,
    pub time_slot: String,
    pub visit_type: VisitType,
    pub symptoms: String,
    pub has_visited_before: bool,
    pub status: AppointmentStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type)]
#[sqlx(type_name = "visit_type", rename_all = "snake_case")]
pub enum VisitType {
    OnlineVideo,
    Offline,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type)]
#[sqlx(type_name = "appointment_status", rename_all = "lowercase")]
pub enum AppointmentStatus {
    Pending,
    Confirmed,
    Completed,
    Cancelled,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateAppointmentDto {
    pub patient_id: Uuid,
    pub doctor_id: Uuid,
    pub appointment_date: DateTime<Utc>,
    pub time_slot: String,
    pub visit_type: VisitType,
    #[validate(length(max = 100))]
    pub symptoms: String,
    pub has_visited_before: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateAppointmentDto {
    pub appointment_date: Option<DateTime<Utc>>,
    pub time_slot: Option<String>,
    pub status: Option<AppointmentStatus>,
}