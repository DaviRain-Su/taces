use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use validator::Validate;

// 常用语分类
#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type)]
#[sqlx(type_name = "phrase_category", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum PhraseCategory {
    Diagnosis, // 诊断
    Advice,    // 医嘱
    Symptom,   // 症状描述
}

// 常用语
#[derive(Debug, Serialize, Deserialize)]
pub struct CommonPhrase {
    pub id: Uuid,
    pub doctor_id: Uuid,
    pub category: PhraseCategory,
    pub content: String,
    pub usage_count: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// 处方模板
#[derive(Debug, Serialize, Deserialize)]
pub struct PrescriptionTemplate {
    pub id: Uuid,
    pub doctor_id: Uuid,
    pub name: String,
    pub description: Option<String>,
    pub diagnosis: String,
    pub medicines: Vec<Medicine>,
    pub instructions: String,
    pub usage_count: i32,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// 使用处方模块中的 Medicine 结构体
use crate::models::prescription::Medicine;

// 创建常用语 DTO
#[derive(Debug, Deserialize, Validate)]
pub struct CreateCommonPhraseDto {
    #[validate(length(min = 1, max = 50))]
    pub category: String,
    #[validate(length(min = 1, max = 1000))]
    pub content: String,
}

// 更新常用语 DTO
#[derive(Debug, Deserialize, Validate)]
pub struct UpdateCommonPhraseDto {
    #[validate(length(min = 1, max = 1000))]
    pub content: Option<String>,
    pub is_active: Option<bool>,
}

// 创建处方模板 DTO
#[derive(Debug, Deserialize, Validate)]
pub struct CreatePrescriptionTemplateDto {
    #[validate(length(min = 1, max = 100))]
    pub name: String,
    #[validate(length(max = 500))]
    pub description: Option<String>,
    #[validate(length(min = 1, max = 500))]
    pub diagnosis: String,
    #[validate(length(min = 1))]
    pub medicines: Vec<Medicine>,
    #[validate(length(min = 1, max = 1000))]
    pub instructions: String,
}

// 更新处方模板 DTO
#[derive(Debug, Deserialize, Validate)]
pub struct UpdatePrescriptionTemplateDto {
    #[validate(length(min = 1, max = 100))]
    pub name: Option<String>,
    #[validate(length(max = 500))]
    pub description: Option<String>,
    #[validate(length(min = 1, max = 500))]
    pub diagnosis: Option<String>,
    pub medicines: Option<Vec<Medicine>>,
    #[validate(length(min = 1, max = 1000))]
    pub instructions: Option<String>,
    pub is_active: Option<bool>,
}

// 查询参数
#[derive(Debug, Deserialize)]
pub struct TemplateQuery {
    pub category: Option<String>,
    pub search: Option<String>,
    pub is_active: Option<bool>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}
