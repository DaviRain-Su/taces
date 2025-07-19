use crate::{config::database::DbPool, models::patient_group::*};
use anyhow::{anyhow, Result};
use chrono::Utc;
use uuid::Uuid;

pub async fn list_doctor_groups(pool: &DbPool, doctor_id: Uuid) -> Result<Vec<PatientGroup>> {
    let query = r#"
        SELECT 
            pg.id, 
            pg.doctor_id, 
            pg.group_name, 
            pg.created_at, 
            pg.updated_at,
            COUNT(pgm.id) as member_count
        FROM patient_groups pg
        LEFT JOIN patient_group_members pgm ON pg.id = pgm.group_id
        WHERE pg.doctor_id = ?
        GROUP BY pg.id
        ORDER BY pg.created_at DESC
    "#;

    let rows = sqlx::query(query)
        .bind(doctor_id.to_string())
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch patient groups: {}", e))?;

    let mut groups = Vec::new();
    for row in rows {
        groups.push(parse_patient_group_from_row(&row)?);
    }

    Ok(groups)
}

pub async fn get_group_by_id(
    pool: &DbPool,
    id: Uuid,
    doctor_id: Uuid,
) -> Result<PatientGroupWithMembers> {
    // First get the group
    let group_query = r#"
        SELECT 
            pg.id, 
            pg.doctor_id, 
            pg.group_name, 
            pg.created_at, 
            pg.updated_at
        FROM patient_groups pg
        WHERE pg.id = ? AND pg.doctor_id = ?
    "#;

    let group_row = sqlx::query(group_query)
        .bind(id.to_string())
        .bind(doctor_id.to_string())
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow!("Patient group not found: {}", e))?;

    // Then get members
    let members_query = r#"
        SELECT 
            pgm.id,
            pgm.group_id,
            pgm.patient_id,
            u.name as patient_name,
            u.phone as patient_phone,
            pgm.joined_at
        FROM patient_group_members pgm
        JOIN users u ON pgm.patient_id = u.id
        WHERE pgm.group_id = ?
        ORDER BY pgm.joined_at DESC
    "#;

    let member_rows = sqlx::query(members_query)
        .bind(id.to_string())
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch group members: {}", e))?;

    let mut members = Vec::new();
    for row in member_rows {
        members.push(parse_group_member_from_row(&row)?);
    }

    Ok(parse_group_with_members(&group_row, members)?)
}

pub async fn create_group(
    pool: &DbPool,
    doctor_id: Uuid,
    dto: CreatePatientGroupDto,
) -> Result<PatientGroup> {
    // Check if doctor already has 5 groups
    let count_query = "SELECT COUNT(*) as count FROM patient_groups WHERE doctor_id = ?";
    let count_row = sqlx::query(count_query)
        .bind(doctor_id.to_string())
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow!("Failed to count groups: {}", e))?;

    use sqlx::Row;
    let count: i64 = count_row.get("count");
    if count >= 5 {
        return Err(anyhow!("Doctor can have maximum 5 patient groups"));
    }

    let group_id = Uuid::new_v4();
    let now = Utc::now();

    let query = r#"
        INSERT INTO patient_groups (id, doctor_id, group_name, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?)
    "#;

    sqlx::query(query)
        .bind(group_id.to_string())
        .bind(doctor_id.to_string())
        .bind(&dto.group_name)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("Duplicate entry") {
                anyhow!("Group name already exists")
            } else {
                anyhow!("Failed to create patient group: {}", e)
            }
        })?;

    // Return the created group
    let created_group = PatientGroup {
        id: group_id,
        doctor_id,
        group_name: dto.group_name,
        member_count: 0,
        created_at: now,
        updated_at: now,
    };

    Ok(created_group)
}

pub async fn update_group(
    pool: &DbPool,
    id: Uuid,
    doctor_id: Uuid,
    dto: UpdatePatientGroupDto,
) -> Result<PatientGroup> {
    // First verify ownership
    let check_query = "SELECT id FROM patient_groups WHERE id = ? AND doctor_id = ?";
    sqlx::query(check_query)
        .bind(id.to_string())
        .bind(doctor_id.to_string())
        .fetch_one(pool)
        .await
        .map_err(|_| anyhow!("Patient group not found or access denied"))?;

    if let Some(group_name) = &dto.group_name {
        let update_query = "UPDATE patient_groups SET group_name = ?, updated_at = ? WHERE id = ?";

        sqlx::query(update_query)
            .bind(group_name)
            .bind(Utc::now())
            .bind(id.to_string())
            .execute(pool)
            .await
            .map_err(|e| {
                if e.to_string().contains("Duplicate entry") {
                    anyhow!("Group name already exists")
                } else {
                    anyhow!("Failed to update patient group: {}", e)
                }
            })?;
    }

    // Fetch and return updated group
    list_doctor_groups(pool, doctor_id)
        .await?
        .into_iter()
        .find(|g| g.id == id)
        .ok_or_else(|| anyhow!("Failed to fetch updated group"))
}

pub async fn delete_group(pool: &DbPool, id: Uuid, doctor_id: Uuid) -> Result<()> {
    let query = "DELETE FROM patient_groups WHERE id = ? AND doctor_id = ?";

    let result = sqlx::query(query)
        .bind(id.to_string())
        .bind(doctor_id.to_string())
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to delete patient group: {}", e))?;

    if result.rows_affected() == 0 {
        return Err(anyhow!("Patient group not found or access denied"));
    }

    Ok(())
}

pub async fn add_members(
    pool: &DbPool,
    group_id: Uuid,
    doctor_id: Uuid,
    patient_ids: Vec<Uuid>,
) -> Result<()> {
    // First verify ownership
    let check_query = "SELECT id FROM patient_groups WHERE id = ? AND doctor_id = ?";
    sqlx::query(check_query)
        .bind(group_id.to_string())
        .bind(doctor_id.to_string())
        .fetch_one(pool)
        .await
        .map_err(|_| anyhow!("Patient group not found or access denied"))?;

    // Verify all patients exist and have role 'patient'
    for patient_id in &patient_ids {
        let patient_check = "SELECT role FROM users WHERE id = ?";
        let row = sqlx::query(patient_check)
            .bind(patient_id.to_string())
            .fetch_one(pool)
            .await
            .map_err(|_| anyhow!("Patient {} not found", patient_id))?;

        use sqlx::Row;
        let role: String = row.get("role");
        if role != "patient" {
            return Err(anyhow!("User {} is not a patient", patient_id));
        }
    }

    // Add members
    for patient_id in patient_ids {
        let member_id = Uuid::new_v4();
        let insert_query = r#"
            INSERT INTO patient_group_members (id, group_id, patient_id, joined_at)
            VALUES (?, ?, ?, ?)
            ON DUPLICATE KEY UPDATE joined_at = joined_at
        "#;

        sqlx::query(insert_query)
            .bind(member_id.to_string())
            .bind(group_id.to_string())
            .bind(patient_id.to_string())
            .bind(Utc::now())
            .execute(pool)
            .await
            .map_err(|e| anyhow!("Failed to add member: {}", e))?;
    }

    Ok(())
}

pub async fn remove_members(
    pool: &DbPool,
    group_id: Uuid,
    doctor_id: Uuid,
    patient_ids: Vec<Uuid>,
) -> Result<()> {
    // First verify ownership
    let check_query = "SELECT id FROM patient_groups WHERE id = ? AND doctor_id = ?";
    sqlx::query(check_query)
        .bind(group_id.to_string())
        .bind(doctor_id.to_string())
        .fetch_one(pool)
        .await
        .map_err(|_| anyhow!("Patient group not found or access denied"))?;

    // Remove members
    for patient_id in patient_ids {
        let delete_query =
            "DELETE FROM patient_group_members WHERE group_id = ? AND patient_id = ?";

        sqlx::query(delete_query)
            .bind(group_id.to_string())
            .bind(patient_id.to_string())
            .execute(pool)
            .await
            .map_err(|e| anyhow!("Failed to remove member: {}", e))?;
    }

    Ok(())
}

pub async fn send_group_message(
    pool: &DbPool,
    group_id: Uuid,
    doctor_id: Uuid,
    _message: &str,
) -> Result<Vec<String>> {
    // First verify ownership and get members
    let members_query = r#"
        SELECT u.id, u.phone
        FROM patient_group_members pgm
        JOIN patient_groups pg ON pgm.group_id = pg.id
        JOIN users u ON pgm.patient_id = u.id
        WHERE pg.id = ? AND pg.doctor_id = ?
    "#;

    let rows = sqlx::query(members_query)
        .bind(group_id.to_string())
        .bind(doctor_id.to_string())
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch group members: {}", e))?;

    if rows.is_empty() {
        return Err(anyhow!("No members in group or access denied"));
    }

    use sqlx::Row;
    let mut phone_numbers = Vec::new();
    for row in rows {
        let phone: String = row.get("phone");
        phone_numbers.push(phone);
    }

    // TODO: Integrate with SMS service to actually send messages
    // For now, just return the phone numbers that would receive the message

    Ok(phone_numbers)
}

fn parse_patient_group_from_row(row: &sqlx::mysql::MySqlRow) -> Result<PatientGroup> {
    use sqlx::Row;

    Ok(PatientGroup {
        id: Uuid::parse_str(row.get("id")).map_err(|e| anyhow!("Failed to parse UUID: {}", e))?,
        doctor_id: Uuid::parse_str(row.get("doctor_id"))
            .map_err(|e| anyhow!("Failed to parse UUID: {}", e))?,
        group_name: row.get("group_name"),
        member_count: row.get::<i64, _>("member_count") as u32,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

fn parse_group_member_from_row(row: &sqlx::mysql::MySqlRow) -> Result<PatientGroupMember> {
    use sqlx::Row;

    Ok(PatientGroupMember {
        id: Uuid::parse_str(row.get("id")).map_err(|e| anyhow!("Failed to parse UUID: {}", e))?,
        group_id: Uuid::parse_str(row.get("group_id"))
            .map_err(|e| anyhow!("Failed to parse UUID: {}", e))?,
        patient_id: Uuid::parse_str(row.get("patient_id"))
            .map_err(|e| anyhow!("Failed to parse UUID: {}", e))?,
        patient_name: row.get("patient_name"),
        patient_phone: row.get("patient_phone"),
        joined_at: row.get("joined_at"),
    })
}

fn parse_group_with_members(
    row: &sqlx::mysql::MySqlRow,
    members: Vec<PatientGroupMember>,
) -> Result<PatientGroupWithMembers> {
    use sqlx::Row;

    Ok(PatientGroupWithMembers {
        id: Uuid::parse_str(row.get("id")).map_err(|e| anyhow!("Failed to parse UUID: {}", e))?,
        doctor_id: Uuid::parse_str(row.get("doctor_id"))
            .map_err(|e| anyhow!("Failed to parse UUID: {}", e))?,
        group_name: row.get("group_name"),
        members,
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}
