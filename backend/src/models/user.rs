use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use validator::Validate;
use sqlx::FromRow;

#[derive(Debug, Serialize, Deserialize, FromRow, Clone)]
pub struct User {
    pub id: Uuid,
    pub account: String,
    pub name: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub gender: String,
    pub phone: String,
    pub email: Option<String>,
    pub birthday: Option<DateTime<Utc>>,
    pub role: UserRole,
    pub status: UserStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type)]
#[sqlx(type_name = "user_role", rename_all = "lowercase")]
pub enum UserRole {
    Admin,
    Doctor,
    Patient,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type)]
#[sqlx(type_name = "user_status", rename_all = "lowercase")]
pub enum UserStatus {
    Active,
    Inactive,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateUserDto {
    #[validate(length(min = 3, max = 50))]
    pub account: String,
    #[validate(length(min = 2, max = 50))]
    pub name: String,
    #[validate(length(min = 6))]
    pub password: String,
    pub gender: String,
    #[validate(length(min = 11, max = 11))]
    pub phone: String,
    #[validate(email)]
    pub email: Option<String>,
    pub birthday: Option<DateTime<Utc>>,
    pub role: UserRole,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateUserDto {
    #[validate(length(min = 2, max = 50))]
    pub name: Option<String>,
    pub gender: Option<String>,
    #[validate(phone)]
    pub phone: Option<String>,
    #[validate(email)]
    pub email: Option<String>,
    pub birthday: Option<DateTime<Utc>>,
    pub status: Option<UserStatus>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct LoginDto {
    pub account: String,
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LoginResponse {
    pub token: String,
    pub user: User,
}