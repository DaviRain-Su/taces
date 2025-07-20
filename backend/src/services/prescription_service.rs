use crate::{
    config::database::DbPool,
    models::{doctor::Doctor, prescription::*},
};
use anyhow::{anyhow, Result};
use chrono::Utc;
use serde_json;
use uuid::Uuid;

pub async fn list_prescriptions(
    pool: &DbPool,
    page: u32,
    per_page: u32,
    search: Option<String>,
) -> Result<Vec<Prescription>> {
    let offset = (page - 1) * per_page;

    let mut query = String::from(
        r#"
        SELECT id, code, doctor_id, patient_id, patient_name, diagnosis, 
               medicines, instructions, prescription_date, created_at
        FROM prescriptions
        WHERE 1=1
    "#,
    );

    if let Some(search_term) = &search {
        query.push_str(&format!(
            " AND (code LIKE '%{}%' OR patient_name LIKE '%{}%' OR diagnosis LIKE '%{}%')",
            search_term, search_term, search_term
        ));
    }

    query.push_str(&format!(
        " ORDER BY created_at DESC LIMIT {} OFFSET {}",
        per_page, offset
    ));

    let rows = sqlx::query(&query)
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch prescriptions: {}", e))?;

    let mut prescriptions = Vec::new();
    for row in rows {
        prescriptions.push(parse_prescription_row(row)?);
    }

    Ok(prescriptions)
}

pub async fn get_prescription_by_id(pool: &DbPool, id: Uuid) -> Result<Prescription> {
    let query = r#"
        SELECT id, code, doctor_id, patient_id, patient_name, diagnosis, 
               medicines, instructions, prescription_date, created_at
        FROM prescriptions
        WHERE id = ?
    "#;

    let row = sqlx::query(query)
        .bind(id.to_string())
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow!("Prescription not found: {}", e))?;

    parse_prescription_row(row)
}

pub async fn get_prescription_by_code(pool: &DbPool, code: &str) -> Result<Prescription> {
    let query = r#"
        SELECT id, code, doctor_id, patient_id, patient_name, diagnosis, 
               medicines, instructions, prescription_date, created_at
        FROM prescriptions
        WHERE code = ?
    "#;

    let row = sqlx::query(query)
        .bind(code)
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow!("Prescription not found: {}", e))?;

    parse_prescription_row(row)
}

pub async fn create_prescription(
    pool: &DbPool,
    dto: CreatePrescriptionDto,
) -> Result<Prescription> {
    let prescription_id = Uuid::new_v4();
    let code = generate_prescription_code();
    let now = Utc::now();
    let medicines_json = serde_json::to_string(&dto.medicines)?;

    let query = r#"
        INSERT INTO prescriptions (id, code, doctor_id, patient_id, patient_name, 
                                 diagnosis, medicines, instructions, prescription_date, created_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    "#;

    sqlx::query(query)
        .bind(prescription_id.to_string())
        .bind(&code)
        .bind(dto.doctor_id.to_string())
        .bind(dto.patient_id.to_string())
        .bind(&dto.patient_name)
        .bind(&dto.diagnosis)
        .bind(&medicines_json)
        .bind(&dto.instructions)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to create prescription: {}", e))?;

    get_prescription_by_id(pool, prescription_id).await
}

pub async fn get_doctor_prescriptions(
    pool: &DbPool,
    doctor_id: Uuid,
    page: u32,
    per_page: u32,
) -> Result<Vec<Prescription>> {
    let offset = (page - 1) * per_page;

    let query = format!(
        r#"
        SELECT id, code, doctor_id, patient_id, patient_name, diagnosis, 
               medicines, instructions, prescription_date, created_at
        FROM prescriptions
        WHERE doctor_id = '{}'
        ORDER BY created_at DESC LIMIT {} OFFSET {}
    "#,
        doctor_id, per_page, offset
    );

    let rows = sqlx::query(&query)
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch doctor prescriptions: {}", e))?;

    let mut prescriptions = Vec::new();
    for row in rows {
        prescriptions.push(parse_prescription_row(row)?);
    }

    Ok(prescriptions)
}

pub async fn get_patient_prescriptions(
    pool: &DbPool,
    patient_id: Uuid,
    page: u32,
    per_page: u32,
) -> Result<Vec<Prescription>> {
    let offset = (page - 1) * per_page;

    let query = format!(
        r#"
        SELECT id, code, doctor_id, patient_id, patient_name, diagnosis, 
               medicines, instructions, prescription_date, created_at
        FROM prescriptions
        WHERE patient_id = '{}'
        ORDER BY prescription_date DESC LIMIT {} OFFSET {}
    "#,
        patient_id, per_page, offset
    );

    let rows = sqlx::query(&query)
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch patient prescriptions: {}", e))?;

    let mut prescriptions = Vec::new();
    for row in rows {
        prescriptions.push(parse_prescription_row(row)?);
    }

    Ok(prescriptions)
}

pub async fn get_doctor_user_id(pool: &DbPool, doctor_id: Uuid) -> Result<Uuid> {
    let query = "SELECT user_id FROM doctors WHERE id = ?";

    let row = sqlx::query(query)
        .bind(doctor_id.to_string())
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow!("Doctor not found: {}", e))?;

    let user_id_str: String = sqlx::Row::get(&row, "user_id");
    Uuid::parse_str(&user_id_str).map_err(|e| anyhow!("Invalid UUID: {}", e))
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
        specialties: sqlx::Row::get::<serde_json::Value, _>(&row, "specialties")
            .as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
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

fn generate_prescription_code() -> String {
    let timestamp = Utc::now().format("%Y%m%d");
    let random_suffix = format!("{:04}", rand::random::<u16>() % 10000);
    format!("RX{}{}", timestamp, random_suffix)
}

fn parse_prescription_row(row: sqlx::mysql::MySqlRow) -> Result<Prescription> {
    use sqlx::Row;

    let medicines_json: serde_json::Value = row.get("medicines");
    let medicines: Vec<Medicine> = serde_json::from_value(medicines_json)
        .map_err(|e| anyhow!("Failed to parse medicines: {}", e))?;

    Ok(Prescription {
        id: Uuid::parse_str(row.get("id")).unwrap(),
        code: row.get("code"),
        doctor_id: Uuid::parse_str(row.get("doctor_id")).unwrap(),
        patient_id: Uuid::parse_str(row.get("patient_id")).unwrap(),
        patient_name: row.get("patient_name"),
        diagnosis: row.get("diagnosis"),
        medicines,
        instructions: row.get("instructions"),
        prescription_date: row.get("prescription_date"),
        created_at: row.get("created_at"),
    })
}
