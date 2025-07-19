use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// 患者评价
#[derive(Debug, Serialize, Deserialize)]
pub struct PatientReview {
    pub id: Uuid,
    pub appointment_id: Uuid,
    pub doctor_id: Uuid,
    pub patient_id: Uuid,
    pub rating: i32,
    pub attitude_rating: i32,
    pub professionalism_rating: i32,
    pub efficiency_rating: i32,
    pub comment: Option<String>,
    pub reply: Option<String>,
    pub reply_at: Option<DateTime<Utc>>,
    pub is_anonymous: bool,
    pub is_visible: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// 评价详情（包含关联信息）
#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewDetail {
    pub id: Uuid,
    pub appointment_id: Uuid,
    pub doctor_id: Uuid,
    pub doctor_name: String,
    pub patient_id: Uuid,
    pub patient_name: String,
    pub rating: i32,
    pub attitude_rating: i32,
    pub professionalism_rating: i32,
    pub efficiency_rating: i32,
    pub comment: Option<String>,
    pub reply: Option<String>,
    pub reply_at: Option<DateTime<Utc>>,
    pub is_anonymous: bool,
    pub tags: Vec<ReviewTag>,
    pub appointment_date: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

// 评价标签
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ReviewTag {
    pub id: Uuid,
    pub name: String,
    pub category: TagCategory,
    pub usage_count: i32,
    pub is_active: bool,
}

// 标签分类
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type)]
#[sqlx(type_name = "tag_category", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum TagCategory {
    Positive,
    Negative,
}

// 医生评价统计
#[derive(Debug, Serialize, Deserialize)]
pub struct DoctorReviewStatistics {
    pub doctor_id: Uuid,
    pub total_reviews: i64,
    pub average_rating: f64,
    pub average_attitude: f64,
    pub average_professionalism: f64,
    pub average_efficiency: f64,
    pub rating_distribution: RatingDistribution,
}

// 评分分布
#[derive(Debug, Serialize, Deserialize)]
pub struct RatingDistribution {
    pub five_star: i64,
    pub four_star: i64,
    pub three_star: i64,
    pub two_star: i64,
    pub one_star: i64,
}

// 创建评价 DTO
#[derive(Debug, Deserialize, Validate)]
pub struct CreateReviewDto {
    pub appointment_id: Uuid,
    #[validate(range(min = 1, max = 5))]
    pub rating: i32,
    #[validate(range(min = 1, max = 5))]
    pub attitude_rating: i32,
    #[validate(range(min = 1, max = 5))]
    pub professionalism_rating: i32,
    #[validate(range(min = 1, max = 5))]
    pub efficiency_rating: i32,
    #[validate(length(max = 1000))]
    pub comment: Option<String>,
    pub tag_ids: Option<Vec<Uuid>>,
    pub is_anonymous: Option<bool>,
}

// 更新评价 DTO
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateReviewDto {
    #[validate(range(min = 1, max = 5))]
    pub rating: Option<i32>,
    #[validate(range(min = 1, max = 5))]
    pub attitude_rating: Option<i32>,
    #[validate(range(min = 1, max = 5))]
    pub professionalism_rating: Option<i32>,
    #[validate(range(min = 1, max = 5))]
    pub efficiency_rating: Option<i32>,
    #[validate(length(max = 1000))]
    pub comment: Option<String>,
    pub tag_ids: Option<Vec<Uuid>>,
    pub is_anonymous: Option<bool>,
}

// 医生回复 DTO
#[derive(Debug, Deserialize, Validate)]
pub struct ReplyReviewDto {
    #[validate(length(min = 1, max = 500))]
    pub reply: String,
}

// 评价查询参数
#[derive(Debug, Deserialize)]
pub struct ReviewQuery {
    pub doctor_id: Option<Uuid>,
    pub patient_id: Option<Uuid>,
    pub rating: Option<i32>,
    pub has_comment: Option<bool>,
    pub has_reply: Option<bool>,
    pub is_anonymous: Option<bool>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

// 创建标签 DTO
#[derive(Debug, Deserialize, Validate)]
pub struct CreateTagDto {
    #[validate(length(min = 1, max = 50))]
    pub name: String,
    pub category: String,
}

// 管理评价可见性 DTO
#[derive(Debug, Deserialize)]
pub struct UpdateReviewVisibilityDto {
    pub is_visible: bool,
}