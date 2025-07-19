use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize)]
pub struct PatientGroup {
    pub id: Uuid,
    pub doctor_id: Uuid,
    pub group_name: String,
    pub member_count: u32, // Will be computed from members
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatientGroupMember {
    pub id: Uuid,
    pub group_id: Uuid,
    pub patient_id: Uuid,
    pub patient_name: String,  // Will be joined from users table
    pub patient_phone: String, // Will be joined from users table
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatientGroupWithMembers {
    pub id: Uuid,
    pub doctor_id: Uuid,
    pub group_name: String,
    pub members: Vec<PatientGroupMember>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreatePatientGroupDto {
    #[validate(length(min = 1, max = 50))]
    pub group_name: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdatePatientGroupDto {
    #[validate(length(min = 1, max = 50))]
    pub group_name: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddMembersDto {
    pub patient_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RemoveMembersDto {
    pub patient_ids: Vec<Uuid>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct GroupMessageDto {
    #[validate(length(min = 1, max = 500))]
    pub message: String,
}
