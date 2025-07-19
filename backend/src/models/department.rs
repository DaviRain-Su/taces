use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize)]
pub struct Department {
    pub id: Uuid,
    pub name: String,
    pub code: String,
    pub contact_person: Option<String>,
    pub contact_phone: Option<String>,
    pub description: Option<String>,
    pub status: DepartmentStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type)]
#[sqlx(type_name = "department_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum DepartmentStatus {
    Active,
    Inactive,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateDepartmentDto {
    #[validate(length(min = 2, max = 50))]
    pub name: String,
    #[validate(length(min = 2, max = 20))]
    pub code: String,
    pub contact_person: Option<String>,
    #[validate(length(min = 11, max = 11))]
    pub contact_phone: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateDepartmentDto {
    #[validate(length(min = 2, max = 50))]
    pub name: Option<String>,
    pub contact_person: Option<String>,
    #[validate(length(min = 11, max = 11))]
    pub contact_phone: Option<String>,
    pub description: Option<String>,
    pub status: Option<DepartmentStatus>,
}
