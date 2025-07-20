use crate::{config::database::DbPool, models::appointment::*};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use uuid::Uuid;

pub async fn list_appointments(
    pool: &DbPool,
    page: u32,
    per_page: u32,
    status: Option<String>,
    date_from: Option<DateTime<Utc>>,
    date_to: Option<DateTime<Utc>>,
) -> Result<Vec<Appointment>> {
    let offset = (page - 1) * per_page;

    let mut query = String::from(
        r#"
        SELECT id, patient_id, doctor_id, appointment_date, time_slot, visit_type, 
               symptoms, has_visited_before, status, created_at, updated_at
        FROM appointments
        WHERE 1=1
    "#,
    );

    if let Some(status_filter) = &status {
        query.push_str(&format!(" AND status = '{}'", status_filter));
    }

    if let Some(from) = date_from {
        query.push_str(&format!(
            " AND appointment_date >= '{}'",
            from.format("%Y-%m-%d %H:%M:%S")
        ));
    }

    if let Some(to) = date_to {
        query.push_str(&format!(
            " AND appointment_date <= '{}'",
            to.format("%Y-%m-%d %H:%M:%S")
        ));
    }

    query.push_str(&format!(
        " ORDER BY appointment_date ASC LIMIT {} OFFSET {}",
        per_page, offset
    ));

    let rows = sqlx::query(&query)
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch appointments: {}", e))?;

    let mut appointments = Vec::new();
    for row in rows {
        appointments.push(parse_appointment_row(row)?);
    }

    Ok(appointments)
}

pub async fn get_appointment_by_id(pool: &DbPool, id: Uuid) -> Result<Appointment> {
    let query = r#"
        SELECT id, patient_id, doctor_id, appointment_date, time_slot, visit_type, 
               symptoms, has_visited_before, status, created_at, updated_at
        FROM appointments
        WHERE id = ?
    "#;

    let row = sqlx::query(query)
        .bind(id.to_string())
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow!("Appointment not found: {}", e))?;

    parse_appointment_row(row)
}

pub async fn create_appointment(pool: &DbPool, dto: CreateAppointmentDto) -> Result<Appointment> {
    // Check if the time slot is available
    if !is_slot_available(pool, dto.doctor_id, dto.appointment_date, &dto.time_slot).await? {
        return Err(anyhow!("Time slot is not available"));
    }

    let appointment_id = Uuid::new_v4();
    let now = Utc::now();

    let query = r#"
        INSERT INTO appointments (id, patient_id, doctor_id, appointment_date, time_slot, 
                                visit_type, symptoms, has_visited_before, status, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'pending', ?, ?)
    "#;

    sqlx::query(query)
        .bind(appointment_id.to_string())
        .bind(dto.patient_id.to_string())
        .bind(dto.doctor_id.to_string())
        .bind(dto.appointment_date)
        .bind(&dto.time_slot)
        .bind(match dto.visit_type {
            VisitType::OnlineVideo => "online_video",
            VisitType::Offline => "offline",
        })
        .bind(&dto.symptoms)
        .bind(dto.has_visited_before)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to create appointment: {}", e))?;

    get_appointment_by_id(pool, appointment_id).await
}

pub async fn update_appointment(
    pool: &DbPool,
    id: Uuid,
    dto: UpdateAppointmentDto,
) -> Result<Appointment> {
    let mut query = "UPDATE appointments SET ".to_string();
    let mut first = true;

    if dto.appointment_date.is_some() {
        query.push_str("appointment_date = ?");
        first = false;
    }

    if dto.time_slot.is_some() {
        if !first {
            query.push_str(", ");
        }
        query.push_str("time_slot = ?");
        first = false;
    }

    if dto.status.is_some() {
        if !first {
            query.push_str(", ");
        }
        query.push_str("status = ?");
        first = false;
    }

    if !first {
        query.push_str(", ");
    }
    query.push_str("updated_at = ? WHERE id = ?");

    if first && dto.appointment_date.is_none() && dto.time_slot.is_none() && dto.status.is_none() {
        return get_appointment_by_id(pool, id).await;
    }

    let mut query_builder = sqlx::query(&query);

    if let Some(date) = dto.appointment_date {
        query_builder = query_builder.bind(date);
    }

    if let Some(slot) = dto.time_slot {
        query_builder = query_builder.bind(slot);
    }

    if let Some(status) = dto.status {
        let status_str = match status {
            AppointmentStatus::Pending => "pending",
            AppointmentStatus::Confirmed => "confirmed",
            AppointmentStatus::Completed => "completed",
            AppointmentStatus::Cancelled => "cancelled",
        };
        query_builder = query_builder.bind(status_str);
    }

    query_builder = query_builder.bind(Utc::now());
    query_builder = query_builder.bind(id.to_string());

    query_builder
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to update appointment: {}", e))?;

    get_appointment_by_id(pool, id).await
}

pub async fn cancel_appointment(pool: &DbPool, id: Uuid) -> Result<Appointment> {
    let query = "UPDATE appointments SET status = 'cancelled', updated_at = ? WHERE id = ?";

    sqlx::query(query)
        .bind(Utc::now())
        .bind(id.to_string())
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to cancel appointment: {}", e))?;

    get_appointment_by_id(pool, id).await
}

pub async fn get_doctor_appointments(
    pool: &DbPool,
    doctor_id: Uuid,
    page: u32,
    per_page: u32,
    status: Option<String>,
) -> Result<Vec<Appointment>> {
    let offset = (page - 1) * per_page;

    let mut query = format!(
        r#"
        SELECT id, patient_id, doctor_id, appointment_date, time_slot, visit_type, 
               symptoms, has_visited_before, status, created_at, updated_at
        FROM appointments
        WHERE doctor_id = '{}'
    "#,
        doctor_id
    );

    if let Some(status_filter) = &status {
        query.push_str(&format!(" AND status = '{}'", status_filter));
    }

    query.push_str(&format!(
        " ORDER BY appointment_date ASC LIMIT {} OFFSET {}",
        per_page, offset
    ));

    let rows = sqlx::query(&query)
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch doctor appointments: {}", e))?;

    let mut appointments = Vec::new();
    for row in rows {
        appointments.push(parse_appointment_row(row)?);
    }

    Ok(appointments)
}

pub async fn get_patient_appointments(
    pool: &DbPool,
    patient_id: Uuid,
    page: u32,
    per_page: u32,
    status: Option<String>,
) -> Result<Vec<Appointment>> {
    let offset = (page - 1) * per_page;

    let mut query = format!(
        r#"
        SELECT id, patient_id, doctor_id, appointment_date, time_slot, visit_type, 
               symptoms, has_visited_before, status, created_at, updated_at
        FROM appointments
        WHERE patient_id = '{}'
    "#,
        patient_id
    );

    if let Some(status_filter) = &status {
        query.push_str(&format!(" AND status = '{}'", status_filter));
    }

    query.push_str(&format!(
        " ORDER BY appointment_date DESC LIMIT {} OFFSET {}",
        per_page, offset
    ));

    let rows = sqlx::query(&query)
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch patient appointments: {}", e))?;

    let mut appointments = Vec::new();
    for row in rows {
        appointments.push(parse_appointment_row(row)?);
    }

    Ok(appointments)
}

pub async fn get_available_slots(
    pool: &DbPool,
    doctor_id: Uuid,
    date: DateTime<Utc>,
) -> Result<Vec<String>> {
    // Define working hours (9 AM to 5 PM)
    let slots = vec![
        "09:00", "09:30", "10:00", "10:30", "11:00", "11:30", "14:00", "14:30", "15:00", "15:30",
        "16:00", "16:30",
    ];

    // Get booked slots for the given date
    let query = r#"
        SELECT time_slot
        FROM appointments
        WHERE doctor_id = ? 
        AND DATE(appointment_date) = DATE(?)
        AND status IN ('pending', 'confirmed')
    "#;

    let booked_rows = sqlx::query(query)
        .bind(doctor_id.to_string())
        .bind(date)
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch booked slots: {}", e))?;

    let booked_slots: Vec<String> = booked_rows
        .iter()
        .map(|row| sqlx::Row::get(row, "time_slot"))
        .collect();

    // Filter out booked slots
    let available_slots: Vec<String> = slots
        .into_iter()
        .filter(|slot| !booked_slots.contains(&slot.to_string()))
        .map(|s| s.to_string())
        .collect();

    Ok(available_slots)
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

async fn is_slot_available(
    pool: &DbPool,
    doctor_id: Uuid,
    date: DateTime<Utc>,
    time_slot: &str,
) -> Result<bool> {
    let query = r#"
        SELECT COUNT(*) as count
        FROM appointments
        WHERE doctor_id = ?
        AND DATE(appointment_date) = DATE(?)
        AND time_slot = ?
        AND status IN ('pending', 'confirmed')
    "#;

    let row = sqlx::query(query)
        .bind(doctor_id.to_string())
        .bind(date)
        .bind(time_slot)
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow!("Failed to check slot availability: {}", e))?;

    let count: i64 = sqlx::Row::get(&row, "count");
    Ok(count == 0)
}

fn parse_appointment_row(row: sqlx::mysql::MySqlRow) -> Result<Appointment> {
    use sqlx::Row;

    let visit_type_str: String = row.get("visit_type");
    let visit_type = match visit_type_str.as_str() {
        "online_video" => VisitType::OnlineVideo,
        "offline" => VisitType::Offline,
        _ => return Err(anyhow!("Invalid visit type")),
    };

    let status_str: String = row.get("status");
    let status = match status_str.as_str() {
        "pending" => AppointmentStatus::Pending,
        "confirmed" => AppointmentStatus::Confirmed,
        "completed" => AppointmentStatus::Completed,
        "cancelled" => AppointmentStatus::Cancelled,
        _ => return Err(anyhow!("Invalid appointment status")),
    };

    Ok(Appointment {
        id: Uuid::parse_str(row.get("id")).unwrap(),
        patient_id: Uuid::parse_str(row.get("patient_id")).unwrap(),
        doctor_id: Uuid::parse_str(row.get("doctor_id")).unwrap(),
        appointment_date: row.get("appointment_date"),
        time_slot: row.get("time_slot"),
        visit_type,
        symptoms: row.get("symptoms"),
        has_visited_before: row.get("has_visited_before"),
        status,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}
