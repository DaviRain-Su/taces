use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Doctor {
    pub id: Uuid,
    pub user_id: Uuid,
    pub certificate_type: String,
    pub id_number: String,
    pub hospital: String,
    pub department: String,
    pub title: String,
    pub introduction: Option<String>,
    pub specialties: Vec<String>,
    pub experience: Option<String>,
    pub avatar: Option<String>,
    pub license_photo: Option<String>,
    pub id_card_front: Option<String>,
    pub id_card_back: Option<String>,
    pub title_cert: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateDoctorDto {
    pub user_id: Uuid,
    pub certificate_type: String,
    #[validate(length(min = 15, max = 18))]
    pub id_number: String,
    pub hospital: String,
    pub department: String,
    pub title: String,
    pub introduction: Option<String>,
    pub specialties: Vec<String>,
    pub experience: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateDoctorDto {
    pub hospital: Option<String>,
    pub department: Option<String>,
    pub title: Option<String>,
    pub introduction: Option<String>,
    pub specialties: Option<Vec<String>>,
    pub experience: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DoctorPhotos {
    pub avatar: Option<String>,
    pub license_photo: Option<String>,
    pub id_card_front: Option<String>,
    pub id_card_back: Option<String>,
    pub title_cert: Option<String>,
}
