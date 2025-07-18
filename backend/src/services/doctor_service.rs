use anyhow::{anyhow, Result};
use uuid::Uuid;
use chrono::Utc;
use serde_json;
use sqlx::types::Json;
use crate::{
    config::database::DbPool,
    models::doctor::*,
};

pub async fn list_doctors(
    pool: &DbPool,
    page: u32,
    per_page: u32,
    department: Option<String>,
    search: Option<String>,
) -> Result<Vec<Doctor>> {
    let offset = (page - 1) * per_page;
    
    let mut query = String::from(r#"
        SELECT id, user_id, certificate_type, id_number, hospital, department, title, 
               introduction, specialties, experience, avatar, license_photo, 
               id_card_front, id_card_back, title_cert, created_at, updated_at
        FROM doctors
        WHERE 1=1
    "#);
    
    if let Some(dept) = &department {
        query.push_str(&format!(" AND department = '{}'", dept));
    }
    
    if let Some(search_term) = &search {
        query.push_str(&format!(" AND (hospital LIKE '%{}%' OR title LIKE '%{}%')", 
            search_term, search_term));
    }
    
    query.push_str(&format!(" ORDER BY created_at DESC LIMIT {} OFFSET {}", per_page, offset));
    
    let rows = sqlx::query(&query)
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch doctors: {}", e))?;
    
    let mut doctors = Vec::new();
    for row in rows {
        let doctor = Doctor {
            id: Uuid::parse_str(sqlx::Row::get(&row, "id")).unwrap(),
            user_id: Uuid::parse_str(sqlx::Row::get(&row, "user_id")).unwrap(),
            certificate_type: sqlx::Row::get(&row, "certificate_type"),
            id_number: sqlx::Row::get(&row, "id_number"),
            hospital: sqlx::Row::get(&row, "hospital"),
            department: sqlx::Row::get(&row, "department"),
            title: sqlx::Row::get(&row, "title"),
            introduction: sqlx::Row::get(&row, "introduction"),
            specialties: {
            let json_value: Json<Vec<String>> = sqlx::Row::get(&row, "specialties");
            json_value.0
        },
            experience: sqlx::Row::get(&row, "experience"),
            avatar: sqlx::Row::get(&row, "avatar"),
            license_photo: sqlx::Row::get(&row, "license_photo"),
            id_card_front: sqlx::Row::get(&row, "id_card_front"),
            id_card_back: sqlx::Row::get(&row, "id_card_back"),
            title_cert: sqlx::Row::get(&row, "title_cert"),
            created_at: sqlx::Row::get(&row, "created_at"),
            updated_at: sqlx::Row::get(&row, "updated_at"),
        };
        doctors.push(doctor);
    }
    
    Ok(doctors)
}

pub async fn get_doctor_by_id(pool: &DbPool, id: Uuid) -> Result<Doctor> {
    let query = r#"
        SELECT id, user_id, certificate_type, id_number, hospital, department, title, 
               introduction, specialties, experience, avatar, license_photo, 
               id_card_front, id_card_back, title_cert, created_at, updated_at
        FROM doctors
        WHERE id = ?
    "#;
    
    let row = sqlx::query(query)
        .bind(id.to_string())
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow!("Doctor not found: {}", e))?;
    
    Ok(Doctor {
        id: Uuid::parse_str(sqlx::Row::get(&row, "id")).unwrap(),
        user_id: Uuid::parse_str(sqlx::Row::get(&row, "user_id")).unwrap(),
        certificate_type: sqlx::Row::get(&row, "certificate_type"),
        id_number: sqlx::Row::get(&row, "id_number"),
        hospital: sqlx::Row::get(&row, "hospital"),
        department: sqlx::Row::get(&row, "department"),
        title: sqlx::Row::get(&row, "title"),
        introduction: sqlx::Row::get(&row, "introduction"),
        specialties: {
            let json_value: Json<Vec<String>> = sqlx::Row::get(&row, "specialties");
            json_value.0
        },
        experience: sqlx::Row::get(&row, "experience"),
        avatar: sqlx::Row::get(&row, "avatar"),
        license_photo: sqlx::Row::get(&row, "license_photo"),
        id_card_front: sqlx::Row::get(&row, "id_card_front"),
        id_card_back: sqlx::Row::get(&row, "id_card_back"),
        title_cert: sqlx::Row::get(&row, "title_cert"),
        created_at: sqlx::Row::get(&row, "created_at"),
        updated_at: sqlx::Row::get(&row, "updated_at"),
    })
}

pub async fn get_doctor_by_user_id(pool: &DbPool, user_id: Uuid) -> Result<Doctor> {
    let query = r#"
        SELECT id, user_id, certificate_type, id_number, hospital, department, title, 
               introduction, specialties, experience, avatar, license_photo, 
               id_card_front, id_card_back, title_cert, created_at, updated_at
        FROM doctors
        WHERE user_id = ?
    "#;
    
    let row = sqlx::query(query)
        .bind(user_id.to_string())
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow!("Doctor not found: {}", e))?;
    
    Ok(Doctor {
        id: Uuid::parse_str(sqlx::Row::get(&row, "id")).unwrap(),
        user_id: Uuid::parse_str(sqlx::Row::get(&row, "user_id")).unwrap(),
        certificate_type: sqlx::Row::get(&row, "certificate_type"),
        id_number: sqlx::Row::get(&row, "id_number"),
        hospital: sqlx::Row::get(&row, "hospital"),
        department: sqlx::Row::get(&row, "department"),
        title: sqlx::Row::get(&row, "title"),
        introduction: sqlx::Row::get(&row, "introduction"),
        specialties: {
            let json_value: Json<Vec<String>> = sqlx::Row::get(&row, "specialties");
            json_value.0
        },
        experience: sqlx::Row::get(&row, "experience"),
        avatar: sqlx::Row::get(&row, "avatar"),
        license_photo: sqlx::Row::get(&row, "license_photo"),
        id_card_front: sqlx::Row::get(&row, "id_card_front"),
        id_card_back: sqlx::Row::get(&row, "id_card_back"),
        title_cert: sqlx::Row::get(&row, "title_cert"),
        created_at: sqlx::Row::get(&row, "created_at"),
        updated_at: sqlx::Row::get(&row, "updated_at"),
    })
}

pub async fn create_doctor(pool: &DbPool, dto: CreateDoctorDto) -> Result<Doctor> {
    let doctor_id = Uuid::new_v4();
    let now = Utc::now();
    let specialties_json = serde_json::to_string(&dto.specialties)?;
    
    let query = r#"
        INSERT INTO doctors (id, user_id, certificate_type, id_number, hospital, department, 
                           title, introduction, specialties, experience, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    "#;
    
    sqlx::query(query)
        .bind(doctor_id.to_string())
        .bind(dto.user_id.to_string())
        .bind(&dto.certificate_type)
        .bind(&dto.id_number)
        .bind(&dto.hospital)
        .bind(&dto.department)
        .bind(&dto.title)
        .bind(&dto.introduction)
        .bind(&specialties_json)
        .bind(&dto.experience)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to create doctor: {}", e))?;
    
    get_doctor_by_id(pool, doctor_id).await
}

pub async fn update_doctor(pool: &DbPool, id: Uuid, dto: UpdateDoctorDto) -> Result<Doctor> {
    let mut update_fields = Vec::new();
    let mut bindings = Vec::new();
    
    if let Some(hospital) = &dto.hospital {
        update_fields.push("hospital = ?");
        bindings.push(hospital.clone());
    }
    
    if let Some(department) = &dto.department {
        update_fields.push("department = ?");
        bindings.push(department.clone());
    }
    
    if let Some(title) = &dto.title {
        update_fields.push("title = ?");
        bindings.push(title.clone());
    }
    
    if let Some(introduction) = &dto.introduction {
        update_fields.push("introduction = ?");
        bindings.push(introduction.clone());
    }
    
    if let Some(specialties) = &dto.specialties {
        update_fields.push("specialties = ?");
        let specialties_json = serde_json::to_string(specialties)?;
        bindings.push(specialties_json);
    }
    
    if let Some(experience) = &dto.experience {
        update_fields.push("experience = ?");
        bindings.push(experience.clone());
    }
    
    update_fields.push("updated_at = ?");
    
    if update_fields.is_empty() {
        return get_doctor_by_id(pool, id).await;
    }
    
    let query = format!(
        "UPDATE doctors SET {} WHERE id = ?",
        update_fields.join(", ")
    );
    
    let mut query_builder = sqlx::query(&query);
    
    for binding in bindings {
        query_builder = query_builder.bind(binding);
    }
    
    query_builder = query_builder.bind(Utc::now());
    query_builder = query_builder.bind(id.to_string());
    
    query_builder
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to update doctor: {}", e))?;
    
    get_doctor_by_id(pool, id).await
}

pub async fn update_doctor_photos(pool: &DbPool, id: Uuid, photos: DoctorPhotos) -> Result<Doctor> {
    let mut update_fields = Vec::new();
    let mut bindings = Vec::new();
    
    if let Some(avatar) = &photos.avatar {
        update_fields.push("avatar = ?");
        bindings.push(avatar.clone());
    }
    
    if let Some(license_photo) = &photos.license_photo {
        update_fields.push("license_photo = ?");
        bindings.push(license_photo.clone());
    }
    
    if let Some(id_card_front) = &photos.id_card_front {
        update_fields.push("id_card_front = ?");
        bindings.push(id_card_front.clone());
    }
    
    if let Some(id_card_back) = &photos.id_card_back {
        update_fields.push("id_card_back = ?");
        bindings.push(id_card_back.clone());
    }
    
    if let Some(title_cert) = &photos.title_cert {
        update_fields.push("title_cert = ?");
        bindings.push(title_cert.clone());
    }
    
    update_fields.push("updated_at = ?");
    
    if update_fields.is_empty() {
        return get_doctor_by_id(pool, id).await;
    }
    
    let query = format!(
        "UPDATE doctors SET {} WHERE id = ?",
        update_fields.join(", ")
    );
    
    let mut query_builder = sqlx::query(&query);
    
    for binding in bindings {
        query_builder = query_builder.bind(binding);
    }
    
    query_builder = query_builder.bind(Utc::now());
    query_builder = query_builder.bind(id.to_string());
    
    query_builder
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to update doctor photos: {}", e))?;
    
    get_doctor_by_id(pool, id).await
}