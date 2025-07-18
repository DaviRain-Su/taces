use anyhow::{anyhow, Result};
use uuid::Uuid;
use chrono::Utc;
use crate::{
    config::database::DbPool,
    models::user::*,
};

fn parse_user_from_row(row: &sqlx::mysql::MySqlRow) -> Result<User> {
    use sqlx::Row;
    
    Ok(User {
        id: Uuid::parse_str(row.get("id")).map_err(|e| anyhow!("Failed to parse UUID: {}", e))?,
        account: row.get("account"),
        name: row.get("name"),
        password: row.get("password"),
        gender: row.get("gender"),
        phone: row.get("phone"),
        email: row.get("email"),
        birthday: row.get("birthday"),
        role: match row.get::<&str, _>("role") {
            "admin" => UserRole::Admin,
            "doctor" => UserRole::Doctor,
            "patient" => UserRole::Patient,
            _ => return Err(anyhow!("Invalid role")),
        },
        status: match row.get::<&str, _>("status") {
            "active" => UserStatus::Active,
            "inactive" => UserStatus::Inactive,
            _ => return Err(anyhow!("Invalid status")),
        },
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

pub async fn list_users(
    pool: &DbPool,
    page: u32,
    per_page: u32,
    search: Option<String>,
    role: Option<String>,
    status: Option<String>,
) -> Result<Vec<User>> {
    let offset = (page - 1) * per_page;
    
    let mut query = String::from(r#"
        SELECT id, account, name, password, gender, phone, email, birthday, role, status, created_at, updated_at
        FROM users
        WHERE 1=1
    "#);
    
    if let Some(search_term) = &search {
        query.push_str(&format!(" AND (account LIKE '%{}%' OR name LIKE '%{}%' OR phone LIKE '%{}%')", 
            search_term, search_term, search_term));
    }
    
    if let Some(role_filter) = &role {
        query.push_str(&format!(" AND role = '{}'", role_filter));
    }
    
    if let Some(status_filter) = &status {
        query.push_str(&format!(" AND status = '{}'", status_filter));
    }
    
    query.push_str(&format!(" ORDER BY created_at DESC LIMIT {} OFFSET {}", per_page, offset));
    
    let rows = sqlx::query(&query)
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch users: {}", e))?;
    
    let mut users = Vec::new();
    for row in rows {
        users.push(parse_user_from_row(&row)?);
    }
    
    Ok(users)
}

pub async fn get_user_by_id(pool: &DbPool, id: Uuid) -> Result<User> {
    let query = r#"
        SELECT id, account, name, password, gender, phone, email, birthday, role, status, created_at, updated_at
        FROM users
        WHERE id = ?
    "#;
    
    let row = sqlx::query(query)
        .bind(id.to_string())
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow!("User not found: {}", e))?;
    
    parse_user_from_row(&row)
}

pub async fn update_user(pool: &DbPool, id: Uuid, dto: UpdateUserDto) -> Result<User> {
    let mut update_fields = Vec::new();
    let mut bindings = Vec::new();
    
    if let Some(name) = &dto.name {
        update_fields.push("name = ?");
        bindings.push(name.clone());
    }
    
    if let Some(gender) = &dto.gender {
        update_fields.push("gender = ?");
        bindings.push(gender.clone());
    }
    
    if let Some(phone) = &dto.phone {
        update_fields.push("phone = ?");
        bindings.push(phone.clone());
    }
    
    if let Some(email) = &dto.email {
        update_fields.push("email = ?");
        bindings.push(email.clone());
    }
    
    if dto.birthday.is_some() {
        update_fields.push("birthday = ?");
    }
    
    if let Some(status) = &dto.status {
        update_fields.push("status = ?");
        let status_str = match status {
            UserStatus::Active => "active",
            UserStatus::Inactive => "inactive",
        };
        bindings.push(status_str.to_string());
    }
    
    update_fields.push("updated_at = ?");
    
    if update_fields.is_empty() {
        return get_user_by_id(pool, id).await;
    }
    
    let query = format!(
        "UPDATE users SET {} WHERE id = ?",
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
        .map_err(|e| anyhow!("Failed to update user: {}", e))?;
    
    get_user_by_id(pool, id).await
}

pub async fn delete_user(pool: &DbPool, id: Uuid) -> Result<()> {
    let query = "DELETE FROM users WHERE id = ?";
    
    sqlx::query(query)
        .bind(id.to_string())
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to delete user: {}", e))?;
    
    Ok(())
}

pub async fn batch_delete_users(pool: &DbPool, ids: Vec<Uuid>) -> Result<u64> {
    if ids.is_empty() {
        return Ok(0);
    }
    
    let placeholders = vec!["?"; ids.len()].join(", ");
    let query = format!("DELETE FROM users WHERE id IN ({})", placeholders);
    
    let mut query_builder = sqlx::query(&query);
    
    for id in ids {
        query_builder = query_builder.bind(id.to_string());
    }
    
    let result = query_builder
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to batch delete users: {}", e))?;
    
    Ok(result.rows_affected())
}

pub async fn export_users(
    pool: &DbPool,
    search: Option<String>,
    role: Option<String>,
    status: Option<String>,
) -> Result<String> {
    let users = list_users(pool, 1, 10000, search, role, status).await?;
    
    let mut csv_data = String::from("Account,Name,Gender,Phone,Email,Role,Status,Created At\n");
    
    for user in users {
        csv_data.push_str(&format!(
            "{},{},{},{},{},{},{},{}\n",
            user.account,
            user.name,
            user.gender,
            user.phone,
            user.email.unwrap_or_default(),
            match user.role {
                UserRole::Admin => "Admin",
                UserRole::Doctor => "Doctor",
                UserRole::Patient => "Patient",
            },
            match user.status {
                UserStatus::Active => "Active",
                UserStatus::Inactive => "Inactive",
            },
            user.created_at.format("%Y-%m-%d %H:%M:%S")
        ));
    }
    
    Ok(csv_data)
}