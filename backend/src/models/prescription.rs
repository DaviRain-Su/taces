use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Prescription {
    pub id: Uuid,
    pub code: String,
    pub doctor_id: Uuid,
    pub patient_id: Uuid,
    pub patient_name: String,
    pub diagnosis: String,
    pub medicines: Vec<Medicine>,
    pub instructions: String,
    pub prescription_date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Medicine {
    pub name: String,
    pub dosage: String,
    pub frequency: String,
    pub duration: String,
    pub notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreatePrescriptionDto {
    pub doctor_id: Uuid,
    pub patient_id: Uuid,
    pub patient_name: String,
    pub diagnosis: String,
    pub medicines: Vec<Medicine>,
    pub instructions: String,
}
