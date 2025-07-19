use crate::{config::database::DbPool, models::department::*};
use anyhow::{anyhow, Result};
use chrono::Utc;
use uuid::Uuid;

pub async fn list_departments(
    pool: &DbPool,
    page: u32,
    per_page: u32,
    search: Option<String>,
    status: Option<String>,
) -> Result<Vec<Department>> {
    let offset = (page - 1) * per_page;

    let mut query = String::from(
        r#"
        SELECT id, name, code, contact_person, contact_phone, description, status, created_at, updated_at
        FROM departments
        WHERE 1=1
    "#,
    );

    if let Some(search_term) = &search {
        query.push_str(&format!(
            " AND (name LIKE '%{}%' OR code LIKE '%{}%')",
            search_term, search_term
        ));
    }

    if let Some(status_filter) = &status {
        query.push_str(&format!(" AND status = '{}'", status_filter));
    }

    query.push_str(&format!(
        " ORDER BY created_at DESC LIMIT {} OFFSET {}",
        per_page, offset
    ));

    let rows = sqlx::query(&query)
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch departments: {}", e))?;

    let mut departments = Vec::new();
    for row in rows {
        departments.push(parse_department_from_row(&row)?);
    }

    Ok(departments)
}

pub async fn get_department_by_id(pool: &DbPool, id: Uuid) -> Result<Department> {
    let query = r#"
        SELECT id, name, code, contact_person, contact_phone, description, status, created_at, updated_at
        FROM departments
        WHERE id = ?
    "#;

    let row = sqlx::query(query)
        .bind(id.to_string())
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow!("Department not found: {}", e))?;

    parse_department_from_row(&row)
}

pub async fn get_department_by_code(pool: &DbPool, code: &str) -> Result<Department> {
    let query = r#"
        SELECT id, name, code, contact_person, contact_phone, description, status, created_at, updated_at
        FROM departments
        WHERE code = ?
    "#;

    let row = sqlx::query(query)
        .bind(code)
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow!("Department not found: {}", e))?;

    parse_department_from_row(&row)
}

pub async fn create_department(pool: &DbPool, dto: CreateDepartmentDto) -> Result<Department> {
    let department_id = Uuid::new_v4();
    let now = Utc::now();

    let query = r#"
        INSERT INTO departments (id, name, code, contact_person, contact_phone, description, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?)
    "#;

    sqlx::query(query)
        .bind(department_id.to_string())
        .bind(&dto.name)
        .bind(&dto.code)
        .bind(&dto.contact_person)
        .bind(&dto.contact_phone)
        .bind(&dto.description)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to create department: {}", e))?;

    get_department_by_id(pool, department_id).await
}

pub async fn update_department(
    pool: &DbPool,
    id: Uuid,
    dto: UpdateDepartmentDto,
) -> Result<Department> {
    let mut update_fields = Vec::new();
    let mut bindings = Vec::new();

    if let Some(name) = &dto.name {
        update_fields.push("name = ?");
        bindings.push(name.clone());
    }

    if let Some(contact_person) = &dto.contact_person {
        update_fields.push("contact_person = ?");
        bindings.push(contact_person.clone());
    }

    if let Some(contact_phone) = &dto.contact_phone {
        update_fields.push("contact_phone = ?");
        bindings.push(contact_phone.clone());
    }

    if dto.description.is_some() {
        update_fields.push("description = ?");
    }

    if let Some(status) = &dto.status {
        update_fields.push("status = ?");
        let status_str = match status {
            DepartmentStatus::Active => "active",
            DepartmentStatus::Inactive => "inactive",
        };
        bindings.push(status_str.to_string());
    }

    update_fields.push("updated_at = ?");

    if update_fields.is_empty() {
        return get_department_by_id(pool, id).await;
    }

    let query = format!(
        "UPDATE departments SET {} WHERE id = ?",
        update_fields.join(", ")
    );

    let mut query_builder = sqlx::query(&query);

    for binding in bindings {
        query_builder = query_builder.bind(binding);
    }

    if dto.description.is_some() {
        query_builder = query_builder.bind(dto.description);
    }

    query_builder = query_builder.bind(Utc::now());
    query_builder = query_builder.bind(id.to_string());

    query_builder
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to update department: {}", e))?;

    get_department_by_id(pool, id).await
}

pub async fn delete_department(pool: &DbPool, id: Uuid) -> Result<()> {
    let query = "DELETE FROM departments WHERE id = ?";

    sqlx::query(query)
        .bind(id.to_string())
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to delete department: {}", e))?;

    Ok(())
}

fn parse_department_from_row(row: &sqlx::mysql::MySqlRow) -> Result<Department> {
    use sqlx::Row;

    Ok(Department {
        id: Uuid::parse_str(row.get("id")).map_err(|e| anyhow!("Failed to parse UUID: {}", e))?,
        name: row.get("name"),
        code: row.get("code"),
        contact_person: row.get("contact_person"),
        contact_phone: row.get("contact_phone"),
        description: row.get("description"),
        status: match row.get::<&str, _>("status") {
            "active" => DepartmentStatus::Active,
            "inactive" => DepartmentStatus::Inactive,
            _ => return Err(anyhow!("Invalid status")),
        },
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}
