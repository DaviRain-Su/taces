use anyhow::{anyhow, Result};
use uuid::Uuid;
use chrono::Utc;
use crate::{
    config::database::DbPool,
    models::patient_profile::*,
};

pub async fn list_user_profiles(pool: &DbPool, user_id: Uuid) -> Result<Vec<PatientProfile>> {
    let query = r#"
        SELECT id, user_id, name, id_number, phone, gender, birthday, 
               relationship, is_default, created_at, updated_at
        FROM patient_profiles
        WHERE user_id = ?
        ORDER BY is_default DESC, created_at DESC
    "#;
    
    let rows = sqlx::query(query)
        .bind(user_id.to_string())
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch patient profiles: {}", e))?;
    
    let mut profiles = Vec::new();
    for row in rows {
        profiles.push(parse_patient_profile_from_row(&row)?);
    }
    
    Ok(profiles)
}

pub async fn get_profile_by_id(pool: &DbPool, id: Uuid, user_id: Uuid) -> Result<PatientProfile> {
    let query = r#"
        SELECT id, user_id, name, id_number, phone, gender, birthday, 
               relationship, is_default, created_at, updated_at
        FROM patient_profiles
        WHERE id = ? AND user_id = ?
    "#;
    
    let row = sqlx::query(query)
        .bind(id.to_string())
        .bind(user_id.to_string())
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow!("Patient profile not found: {}", e))?;
    
    parse_patient_profile_from_row(&row)
}

pub async fn get_default_profile(pool: &DbPool, user_id: Uuid) -> Result<Option<PatientProfile>> {
    let query = r#"
        SELECT id, user_id, name, id_number, phone, gender, birthday, 
               relationship, is_default, created_at, updated_at
        FROM patient_profiles
        WHERE user_id = ? AND is_default = TRUE
        LIMIT 1
    "#;
    
    let row = sqlx::query(query)
        .bind(user_id.to_string())
        .fetch_optional(pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch default profile: {}", e))?;
    
    match row {
        Some(row) => Ok(Some(parse_patient_profile_from_row(&row)?)),
        None => Ok(None),
    }
}

pub async fn create_profile(pool: &DbPool, user_id: Uuid, dto: CreatePatientProfileDto) -> Result<PatientProfile> {
    // Validate ID number
    if !validate_id_number(&dto.id_number) {
        return Err(anyhow!("Invalid ID number format"));
    }
    
    // Check if this ID number is already used by another profile
    let check_query = "SELECT COUNT(*) as count FROM patient_profiles WHERE id_number = ? AND user_id != ?";
    let count_row = sqlx::query(check_query)
        .bind(&dto.id_number)
        .bind(user_id.to_string())
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow!("Failed to check ID number: {}", e))?;
    
    use sqlx::Row;
    let count: i64 = count_row.get("count");
    if count > 0 {
        return Err(anyhow!("This ID number is already registered"));
    }
    
    let profile_id = Uuid::new_v4();
    let now = Utc::now();
    
    // If this is the first profile or is marked as self, make it default
    let is_first_profile = list_user_profiles(pool, user_id).await?.is_empty();
    let should_be_default = is_first_profile || matches!(dto.relationship, Relationship::MySelf);
    
    // If setting as default, unset other defaults
    if should_be_default {
        let unset_query = "UPDATE patient_profiles SET is_default = FALSE WHERE user_id = ?";
        sqlx::query(unset_query)
            .bind(user_id.to_string())
            .execute(pool)
            .await
            .map_err(|e| anyhow!("Failed to unset default profiles: {}", e))?;
    }
    
    let query = r#"
        INSERT INTO patient_profiles (id, user_id, name, id_number, phone, gender, 
                                    birthday, relationship, is_default, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    "#;
    
    let gender_str = match dto.gender {
        Gender::Male => "男",
        Gender::Female => "女",
    };
    
    let relationship_str = match dto.relationship {
        Relationship::MySelf => "self",
        Relationship::Family => "family",
        Relationship::Friend => "friend",
        Relationship::Other => "other",
    };
    
    sqlx::query(query)
        .bind(profile_id.to_string())
        .bind(user_id.to_string())
        .bind(&dto.name)
        .bind(&dto.id_number)
        .bind(&dto.phone)
        .bind(gender_str)
        .bind(dto.birthday)
        .bind(relationship_str)
        .bind(should_be_default)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to create patient profile: {}", e))?;
    
    get_profile_by_id(pool, profile_id, user_id).await
}

pub async fn update_profile(pool: &DbPool, id: Uuid, user_id: Uuid, dto: UpdatePatientProfileDto) -> Result<PatientProfile> {
    // First verify ownership
    let check_query = "SELECT id FROM patient_profiles WHERE id = ? AND user_id = ?";
    sqlx::query(check_query)
        .bind(id.to_string())
        .bind(user_id.to_string())
        .fetch_one(pool)
        .await
        .map_err(|_| anyhow!("Patient profile not found or access denied"))?;
    
    let mut update_fields = Vec::new();
    let mut bindings: Vec<String> = Vec::new();
    
    if let Some(name) = &dto.name {
        update_fields.push("name = ?");
        bindings.push(name.clone());
    }
    
    if let Some(phone) = &dto.phone {
        update_fields.push("phone = ?");
        bindings.push(phone.clone());
    }
    
    if let Some(gender) = &dto.gender {
        update_fields.push("gender = ?");
        let gender_str = match gender {
            Gender::Male => "男",
            Gender::Female => "女",
        };
        bindings.push(gender_str.to_string());
    }
    
    if dto.birthday.is_some() {
        update_fields.push("birthday = ?");
    }
    
    if let Some(relationship) = &dto.relationship {
        update_fields.push("relationship = ?");
        let relationship_str = match relationship {
            Relationship::MySelf => "self",
            Relationship::Family => "family", 
            Relationship::Friend => "friend",
            Relationship::Other => "other",
        };
        bindings.push(relationship_str.to_string());
    }
    
    update_fields.push("updated_at = ?");
    
    if update_fields.len() == 1 { // Only updated_at
        return get_profile_by_id(pool, id, user_id).await;
    }
    
    let query = format!(
        "UPDATE patient_profiles SET {} WHERE id = ?",
        update_fields.join(", ")
    );
    
    let mut query_builder = sqlx::query(&query);
    
    for binding in bindings {
        query_builder = query_builder.bind(binding);
    }
    
    if dto.birthday.is_some() {
        query_builder = query_builder.bind(dto.birthday);
    }
    
    query_builder = query_builder.bind(Utc::now());
    query_builder = query_builder.bind(id.to_string());
    
    query_builder
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to update patient profile: {}", e))?;
    
    get_profile_by_id(pool, id, user_id).await
}

pub async fn delete_profile(pool: &DbPool, id: Uuid, user_id: Uuid) -> Result<()> {
    // Check if this is the default profile
    let profile = get_profile_by_id(pool, id, user_id).await?;
    
    // Cannot delete self relationship profile
    if matches!(profile.relationship, Relationship::MySelf) {
        return Err(anyhow!("Cannot delete self profile"));
    }
    
    let query = "DELETE FROM patient_profiles WHERE id = ? AND user_id = ?";
    
    let result = sqlx::query(query)
        .bind(id.to_string())
        .bind(user_id.to_string())
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to delete patient profile: {}", e))?;
    
    if result.rows_affected() == 0 {
        return Err(anyhow!("Patient profile not found or access denied"));
    }
    
    // If deleted profile was default, set another as default
    if profile.is_default {
        let profiles = list_user_profiles(pool, user_id).await?;
        if let Some(first_profile) = profiles.first() {
            set_default_profile(pool, first_profile.id, user_id).await?;
        }
    }
    
    Ok(())
}

pub async fn set_default_profile(pool: &DbPool, id: Uuid, user_id: Uuid) -> Result<()> {
    // First verify ownership
    let check_query = "SELECT id FROM patient_profiles WHERE id = ? AND user_id = ?";
    sqlx::query(check_query)
        .bind(id.to_string())
        .bind(user_id.to_string())
        .fetch_one(pool)
        .await
        .map_err(|_| anyhow!("Patient profile not found or access denied"))?;
    
    // Unset all defaults for this user
    let unset_query = "UPDATE patient_profiles SET is_default = FALSE WHERE user_id = ?";
    sqlx::query(unset_query)
        .bind(user_id.to_string())
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to unset default profiles: {}", e))?;
    
    // Set the new default
    let set_query = "UPDATE patient_profiles SET is_default = TRUE, updated_at = ? WHERE id = ?";
    sqlx::query(set_query)
        .bind(Utc::now())
        .bind(id.to_string())
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to set default profile: {}", e))?;
    
    Ok(())
}

fn parse_patient_profile_from_row(row: &sqlx::mysql::MySqlRow) -> Result<PatientProfile> {
    use sqlx::Row;
    
    let gender = match row.get::<&str, _>("gender") {
        "男" => Gender::Male,
        "女" => Gender::Female,
        _ => return Err(anyhow!("Invalid gender value")),
    };
    
    let relationship = match row.get::<&str, _>("relationship") {
        "self" => Relationship::MySelf,
        "family" => Relationship::Family,
        "friend" => Relationship::Friend,
        "other" => Relationship::Other,
        _ => return Err(anyhow!("Invalid relationship value")),
    };
    
    Ok(PatientProfile {
        id: Uuid::parse_str(row.get("id")).map_err(|e| anyhow!("Failed to parse UUID: {}", e))?,
        user_id: Uuid::parse_str(row.get("user_id")).map_err(|e| anyhow!("Failed to parse UUID: {}", e))?,
        name: row.get("name"),
        id_number: row.get("id_number"),
        phone: row.get("phone"),
        gender,
        birthday: row.get("birthday"),
        relationship,
        is_default: row.get("is_default"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}