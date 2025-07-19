use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Circle {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub avatar: Option<String>,
    pub category: String,
    pub creator_id: Uuid,
    pub member_count: i32,
    pub post_count: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CircleListItem {
    pub id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub avatar: Option<String>,
    pub category: String,
    pub member_count: i32,
    pub post_count: i32,
    pub is_joined: bool,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct CircleMember {
    pub id: Uuid,
    pub circle_id: Uuid,
    pub user_id: Uuid,
    pub role: MemberRole,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type)]
#[sqlx(type_name = "member_role", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum MemberRole {
    Owner,
    Admin,
    Member,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateCircleDto {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(max = 500))]
    pub description: Option<String>,
    pub avatar: Option<String>,
    #[validate(length(min = 1, max = 50))]
    pub category: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateCircleDto {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    #[validate(length(max = 500))]
    pub description: Option<String>,
    pub avatar: Option<String>,
    pub is_active: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CircleWithMemberInfo {
    pub circle: Circle,
    pub is_joined: bool,
    pub member_role: Option<MemberRole>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CircleMemberInfo {
    pub id: Uuid,
    pub user_id: Uuid,
    pub user_name: String,
    pub user_avatar: Option<String>,
    pub role: MemberRole,
    pub joined_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdateMemberRoleDto {
    pub role: MemberRole,
}