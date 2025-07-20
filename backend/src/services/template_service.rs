use crate::config::database::DbPool;
use crate::models::{
    CommonPhrase, CreateCommonPhraseDto, CreatePrescriptionTemplateDto,
    PrescriptionTemplate, UpdateCommonPhraseDto, UpdatePrescriptionTemplateDto,
};
use anyhow::{anyhow, Result};
use serde_json;
use sqlx::Row;
use uuid::Uuid;

pub struct TemplateService;

impl TemplateService {
    // ========== 常用语相关方法 ==========
    
    pub async fn create_common_phrase(
        pool: &DbPool,
        doctor_id: Uuid,
        dto: CreateCommonPhraseDto,
    ) -> Result<CommonPhrase> {
        // 验证分类
        let category = match dto.category.as_str() {
            "diagnosis" | "advice" | "symptom" => &dto.category,
            _ => return Err(anyhow!("Invalid category")),
        };

        let phrase_id = Uuid::new_v4();
        
        sqlx::query(
            r#"
            INSERT INTO common_phrases (id, doctor_id, category, content)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(phrase_id.to_string())
        .bind(doctor_id.to_string())
        .bind(category)
        .bind(&dto.content)
        .execute(pool)
        .await?;

        Self::get_common_phrase_by_id(pool, phrase_id).await
    }

    pub async fn get_common_phrases(
        pool: &DbPool,
        doctor_id: Uuid,
        category: Option<String>,
        search: Option<String>,
        is_active: Option<bool>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<CommonPhrase>, i64)> {
        let offset = (page - 1) * page_size;

        // 构建查询条件
        let mut count_query = String::from(
            "SELECT COUNT(*) FROM common_phrases WHERE doctor_id = ?"
        );
        let mut list_query = String::from(
            r#"
            SELECT id, doctor_id, category, content, usage_count, is_active,
                   created_at, updated_at
            FROM common_phrases
            WHERE doctor_id = ?
            "#,
        );

        let mut params = vec![doctor_id.to_string()];

        if let Some(cat) = category {
            count_query.push_str(" AND category = ?");
            list_query.push_str(" AND category = ?");
            params.push(cat);
        }

        if let Some(keyword) = search {
            count_query.push_str(" AND content LIKE ?");
            list_query.push_str(" AND content LIKE ?");
            params.push(format!("%{}%", keyword));
        }

        if let Some(active) = is_active {
            count_query.push_str(" AND is_active = ?");
            list_query.push_str(" AND is_active = ?");
            params.push(active.to_string());
        }

        list_query.push_str(" ORDER BY usage_count DESC, created_at DESC LIMIT ? OFFSET ?");

        // 获取总数
        let mut count_query_builder = sqlx::query(&count_query);
        for param in &params {
            count_query_builder = count_query_builder.bind(param);
        }
        let total: i64 = count_query_builder
            .fetch_one(pool)
            .await?
            .get::<i64, _>(0);

        // 获取列表
        let mut list_query_builder = sqlx::query(&list_query);
        for param in params {
            list_query_builder = list_query_builder.bind(param);
        }
        list_query_builder = list_query_builder.bind(page_size).bind(offset);

        let rows = list_query_builder.fetch_all(pool).await?;
        let phrases = rows
            .into_iter()
            .map(|row| parse_common_phrase_row(&row))
            .collect::<Result<Vec<_>>>()?;

        Ok((phrases, total))
    }

    pub async fn get_common_phrase_by_id(
        pool: &DbPool,
        id: Uuid,
    ) -> Result<CommonPhrase> {
        let row = sqlx::query(
            r#"
            SELECT id, doctor_id, category, content, usage_count, is_active,
                   created_at, updated_at
            FROM common_phrases
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| anyhow!("Common phrase not found"))?;

        parse_common_phrase_row(&row)
    }

    pub async fn update_common_phrase(
        pool: &DbPool,
        id: Uuid,
        doctor_id: Uuid,
        dto: UpdateCommonPhraseDto,
    ) -> Result<CommonPhrase> {
        // 验证所有权
        let phrase = Self::get_common_phrase_by_id(pool, id).await?;
        if phrase.doctor_id != doctor_id {
            return Err(anyhow!("No permission to update this phrase"));
        }

        // 构建动态更新查询
        let mut query = String::from("UPDATE common_phrases SET ");
        let mut first = true;
        
        if dto.content.is_some() {
            if !first { query.push_str(", "); }
            query.push_str("content = ?");
            first = false;
        }
        
        if dto.is_active.is_some() {
            if !first { query.push_str(", "); }
            query.push_str("is_active = ?");
            first = false;
        }
        
        if first {
            return Err(anyhow!("No fields to update"));
        }
        
        query.push_str(", updated_at = CURRENT_TIMESTAMP WHERE id = ?");
        
        // 绑定参数
        let mut query_builder = sqlx::query(&query);
        
        if let Some(content) = dto.content {
            query_builder = query_builder.bind(content);
        }
        
        if let Some(is_active) = dto.is_active {
            query_builder = query_builder.bind(is_active);
        }
        
        query_builder = query_builder.bind(id.to_string());
        
        query_builder.execute(pool).await?;

        Self::get_common_phrase_by_id(pool, id).await
    }

    pub async fn delete_common_phrase(
        pool: &DbPool,
        id: Uuid,
        doctor_id: Uuid,
    ) -> Result<()> {
        // 验证所有权
        let phrase = Self::get_common_phrase_by_id(pool, id).await?;
        if phrase.doctor_id != doctor_id {
            return Err(anyhow!("No permission to delete this phrase"));
        }

        sqlx::query("DELETE FROM common_phrases WHERE id = ?")
            .bind(id.to_string())
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn increment_phrase_usage(
        pool: &DbPool,
        id: Uuid,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE common_phrases SET usage_count = usage_count + 1 WHERE id = ?"
        )
        .bind(id.to_string())
        .execute(pool)
        .await?;

        Ok(())
    }

    // ========== 处方模板相关方法 ==========
    
    pub async fn create_prescription_template(
        pool: &DbPool,
        doctor_id: Uuid,
        dto: CreatePrescriptionTemplateDto,
    ) -> Result<PrescriptionTemplate> {
        let template_id = Uuid::new_v4();
        let medicines_json = serde_json::to_string(&dto.medicines)?;
        
        sqlx::query(
            r#"
            INSERT INTO prescription_templates 
            (id, doctor_id, name, description, diagnosis, medicines, instructions)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(template_id.to_string())
        .bind(doctor_id.to_string())
        .bind(&dto.name)
        .bind(&dto.description)
        .bind(&dto.diagnosis)
        .bind(&medicines_json)
        .bind(&dto.instructions)
        .execute(pool)
        .await?;

        Self::get_prescription_template_by_id(pool, template_id).await
    }

    pub async fn get_prescription_templates(
        pool: &DbPool,
        doctor_id: Uuid,
        search: Option<String>,
        is_active: Option<bool>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<PrescriptionTemplate>, i64)> {
        let offset = (page - 1) * page_size;

        // 构建查询条件
        let mut count_query = String::from(
            "SELECT COUNT(*) FROM prescription_templates WHERE doctor_id = ?"
        );
        let mut list_query = String::from(
            r#"
            SELECT id, doctor_id, name, description, diagnosis, medicines,
                   instructions, usage_count, is_active, created_at, updated_at
            FROM prescription_templates
            WHERE doctor_id = ?
            "#,
        );

        let mut params = vec![doctor_id.to_string()];

        if let Some(keyword) = search {
            count_query.push_str(" AND (name LIKE ? OR diagnosis LIKE ?)");
            list_query.push_str(" AND (name LIKE ? OR diagnosis LIKE ?)");
            let search_param = format!("%{}%", keyword);
            params.push(search_param.clone());
            params.push(search_param);
        }

        if let Some(active) = is_active {
            count_query.push_str(" AND is_active = ?");
            list_query.push_str(" AND is_active = ?");
            params.push(active.to_string());
        }

        list_query.push_str(" ORDER BY usage_count DESC, created_at DESC LIMIT ? OFFSET ?");

        // 获取总数
        let mut count_query_builder = sqlx::query(&count_query);
        for param in &params {
            count_query_builder = count_query_builder.bind(param);
        }
        let total: i64 = count_query_builder
            .fetch_one(pool)
            .await?
            .get::<i64, _>(0);

        // 获取列表
        let mut list_query_builder = sqlx::query(&list_query);
        for param in params {
            list_query_builder = list_query_builder.bind(param);
        }
        list_query_builder = list_query_builder.bind(page_size).bind(offset);

        let rows = list_query_builder.fetch_all(pool).await?;
        let templates = rows
            .into_iter()
            .map(|row| parse_prescription_template_row(&row))
            .collect::<Result<Vec<_>>>()?;

        Ok((templates, total))
    }

    pub async fn get_prescription_template_by_id(
        pool: &DbPool,
        id: Uuid,
    ) -> Result<PrescriptionTemplate> {
        let row = sqlx::query(
            r#"
            SELECT id, doctor_id, name, description, diagnosis, medicines,
                   instructions, usage_count, is_active, created_at, updated_at
            FROM prescription_templates
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| anyhow!("Prescription template not found"))?;

        parse_prescription_template_row(&row)
    }

    pub async fn update_prescription_template(
        pool: &DbPool,
        id: Uuid,
        doctor_id: Uuid,
        dto: UpdatePrescriptionTemplateDto,
    ) -> Result<PrescriptionTemplate> {
        // 验证所有权
        let template = Self::get_prescription_template_by_id(pool, id).await?;
        if template.doctor_id != doctor_id {
            return Err(anyhow!("No permission to update this template"));
        }

        // 构建动态更新查询
        let mut query = String::from("UPDATE prescription_templates SET ");
        let mut updates = vec![];
        
        if dto.name.is_some() {
            updates.push("name = ?");
        }
        
        if dto.description.is_some() {
            updates.push("description = ?");
        }
        
        if dto.diagnosis.is_some() {
            updates.push("diagnosis = ?");
        }
        
        if dto.medicines.is_some() {
            updates.push("medicines = ?");
        }
        
        if dto.instructions.is_some() {
            updates.push("instructions = ?");
        }
        
        if dto.is_active.is_some() {
            updates.push("is_active = ?");
        }
        
        if updates.is_empty() {
            return Err(anyhow!("No fields to update"));
        }
        
        query.push_str(&updates.join(", "));
        query.push_str(", updated_at = CURRENT_TIMESTAMP WHERE id = ?");
        
        // 绑定参数
        let mut query_builder = sqlx::query(&query);
        
        if let Some(name) = dto.name {
            query_builder = query_builder.bind(name);
        }
        
        if let Some(description) = dto.description {
            query_builder = query_builder.bind(description);
        }
        
        if let Some(diagnosis) = dto.diagnosis {
            query_builder = query_builder.bind(diagnosis);
        }
        
        if let Some(medicines) = dto.medicines {
            let medicines_json = serde_json::to_string(&medicines)?;
            query_builder = query_builder.bind(medicines_json);
        }
        
        if let Some(instructions) = dto.instructions {
            query_builder = query_builder.bind(instructions);
        }
        
        if let Some(is_active) = dto.is_active {
            query_builder = query_builder.bind(is_active);
        }
        
        query_builder = query_builder.bind(id.to_string());
        
        query_builder.execute(pool).await?;

        Self::get_prescription_template_by_id(pool, id).await
    }

    pub async fn delete_prescription_template(
        pool: &DbPool,
        id: Uuid,
        doctor_id: Uuid,
    ) -> Result<()> {
        // 验证所有权
        let template = Self::get_prescription_template_by_id(pool, id).await?;
        if template.doctor_id != doctor_id {
            return Err(anyhow!("No permission to delete this template"));
        }

        sqlx::query("DELETE FROM prescription_templates WHERE id = ?")
            .bind(id.to_string())
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn increment_template_usage(
        pool: &DbPool,
        id: Uuid,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE prescription_templates SET usage_count = usage_count + 1 WHERE id = ?"
        )
        .bind(id.to_string())
        .execute(pool)
        .await?;

        Ok(())
    }

    // 验证医生身份
    pub async fn verify_doctor_access(
        pool: &DbPool,
        user_id: Uuid,
    ) -> Result<Uuid> {
        let row = sqlx::query(
            "SELECT id FROM doctors WHERE user_id = ?"
        )
        .bind(user_id.to_string())
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| anyhow!("Doctor profile not found"))?;

        let doctor_id_str: String = row.get("id");
        Ok(Uuid::parse_str(&doctor_id_str)?)
    }
}

// 解析常用语行
fn parse_common_phrase_row(row: &sqlx::mysql::MySqlRow) -> Result<CommonPhrase> {
    let id_str: String = row.get("id");
    let doctor_id_str: String = row.get("doctor_id");
    let category_str: String = row.get("category");
    
    Ok(CommonPhrase {
        id: Uuid::parse_str(&id_str)?,
        doctor_id: Uuid::parse_str(&doctor_id_str)?,
        category: match category_str.as_str() {
            "diagnosis" => crate::models::template::PhraseCategory::Diagnosis,
            "advice" => crate::models::template::PhraseCategory::Advice,
            "symptom" => crate::models::template::PhraseCategory::Symptom,
            _ => return Err(anyhow!("Invalid category")),
        },
        content: row.get("content"),
        usage_count: row.get("usage_count"),
        is_active: row.get("is_active"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

// 解析处方模板行
fn parse_prescription_template_row(row: &sqlx::mysql::MySqlRow) -> Result<PrescriptionTemplate> {
    let id_str: String = row.get("id");
    let doctor_id_str: String = row.get("doctor_id");
    let medicines_json: serde_json::Value = row.get("medicines");
    
    Ok(PrescriptionTemplate {
        id: Uuid::parse_str(&id_str)?,
        doctor_id: Uuid::parse_str(&doctor_id_str)?,
        name: row.get("name"),
        description: row.get("description"),
        diagnosis: row.get("diagnosis"),
        medicines: serde_json::from_value(medicines_json)?,
        instructions: row.get("instructions"),
        usage_count: row.get("usage_count"),
        is_active: row.get("is_active"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}