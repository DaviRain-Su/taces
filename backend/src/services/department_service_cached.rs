use crate::{
    config::{database::DbPool, redis::RedisPool},
    models::department::*,
    services::cache_service::{CacheDurations, CacheKeys, CacheService},
};
use anyhow::{anyhow, Result};
use chrono::Utc;
use uuid::Uuid;

pub async fn list_departments_cached(
    pool: &DbPool,
    redis: &Option<RedisPool>,
    page: u32,
    per_page: u32,
    search: Option<String>,
    status: Option<String>,
) -> Result<Vec<Department>> {
    // For searches and filters, skip cache
    if search.is_some() || status.is_some() {
        return list_departments_uncached(pool, page, per_page, search, status).await;
    }

    // Try to get from cache for default listing
    let cache_key = format!(
        "{}:page{}:size{}",
        CacheKeys::department_list(),
        page,
        per_page
    );

    if let Some(departments) = CacheService::get::<Vec<Department>>(redis, &cache_key).await {
        tracing::debug!("Cache hit for departments list");
        return Ok(departments);
    }

    // Cache miss, fetch from database
    let departments = list_departments_uncached(pool, page, per_page, None, None).await?;

    // Store in cache
    if let Err(e) = CacheService::set(redis, &cache_key, &departments, CacheDurations::MEDIUM).await
    {
        tracing::warn!("Failed to cache departments: {}", e);
    }

    Ok(departments)
}

pub async fn get_department_by_id_cached(
    pool: &DbPool,
    redis: &Option<RedisPool>,
    id: Uuid,
) -> Result<Department> {
    let cache_key = format!("department:{}", id);

    // Try cache first
    if let Some(department) = CacheService::get::<Department>(redis, &cache_key).await {
        tracing::debug!("Cache hit for department {}", id);
        return Ok(department);
    }

    // Cache miss, fetch from database
    let department = get_department_by_id_uncached(pool, id).await?;

    // Store in cache
    if let Err(e) = CacheService::set(redis, &cache_key, &department, CacheDurations::LONG).await {
        tracing::warn!("Failed to cache department: {}", e);
    }

    Ok(department)
}

pub async fn get_department_by_code_cached(
    pool: &DbPool,
    redis: &Option<RedisPool>,
    code: &str,
) -> Result<Department> {
    let cache_key = format!("department:code:{}", code);

    // Try cache first
    if let Some(department) = CacheService::get::<Department>(redis, &cache_key).await {
        tracing::debug!("Cache hit for department code {}", code);
        return Ok(department);
    }

    // Cache miss, fetch from database
    let department = get_department_by_code_uncached(pool, code).await?;

    // Store in cache
    if let Err(e) = CacheService::set(redis, &cache_key, &department, CacheDurations::LONG).await {
        tracing::warn!("Failed to cache department: {}", e);
    }

    Ok(department)
}

pub async fn create_department_cached(
    pool: &DbPool,
    redis: &Option<RedisPool>,
    input: CreateDepartmentDto,
) -> Result<Department> {
    let department = create_department_uncached(pool, input).await?;

    // Invalidate department list cache
    if let Err(e) =
        CacheService::delete_pattern(redis, &format!("{}:*", CacheKeys::department_list())).await
    {
        tracing::warn!("Failed to invalidate department list cache: {}", e);
    }

    Ok(department)
}

pub async fn update_department_cached(
    pool: &DbPool,
    redis: &Option<RedisPool>,
    id: Uuid,
    input: UpdateDepartmentDto,
) -> Result<Department> {
    let department = update_department_uncached(pool, id, input).await?;

    // Invalidate caches
    let cache_key = format!("department:{}", id);
    if let Err(e) = CacheService::delete(redis, &cache_key).await {
        tracing::warn!("Failed to invalidate department cache: {}", e);
    }

    if let Err(e) =
        CacheService::delete_pattern(redis, &format!("{}:*", CacheKeys::department_list())).await
    {
        tracing::warn!("Failed to invalidate department list cache: {}", e);
    }

    Ok(department)
}

pub async fn delete_department_cached(
    pool: &DbPool,
    redis: &Option<RedisPool>,
    id: Uuid,
) -> Result<()> {
    delete_department_uncached(pool, id).await?;

    // Invalidate caches
    let cache_key = format!("department:{}", id);
    if let Err(e) = CacheService::delete(redis, &cache_key).await {
        tracing::warn!("Failed to invalidate department cache: {}", e);
    }

    if let Err(e) =
        CacheService::delete_pattern(redis, &format!("{}:*", CacheKeys::department_list())).await
    {
        tracing::warn!("Failed to invalidate department list cache: {}", e);
    }

    Ok(())
}

// Original functions without caching
async fn list_departments_uncached(
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

async fn get_department_by_id_uncached(pool: &DbPool, id: Uuid) -> Result<Department> {
    let query = r#"
        SELECT id, name, code, contact_person, contact_phone, description, status, created_at, updated_at
        FROM departments
        WHERE id = ?
    "#;

    let row = sqlx::query(query)
        .bind(id.to_string())
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch department: {}", e))?;

    parse_department_from_row(&row)
}

async fn get_department_by_code_uncached(pool: &DbPool, code: &str) -> Result<Department> {
    let query = r#"
        SELECT id, name, code, contact_person, contact_phone, description, status, created_at, updated_at
        FROM departments
        WHERE code = ?
    "#;

    let row = sqlx::query(query)
        .bind(code)
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch department: {}", e))?;

    parse_department_from_row(&row)
}

async fn create_department_uncached(
    pool: &DbPool,
    input: CreateDepartmentDto,
) -> Result<Department> {
    let id = Uuid::new_v4();
    let now = Utc::now();

    let query = r#"
        INSERT INTO departments (id, name, code, contact_person, contact_phone, description, status, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
    "#;

    sqlx::query(query)
        .bind(id.to_string())
        .bind(&input.name)
        .bind(&input.code)
        .bind(&input.contact_person)
        .bind(&input.contact_phone)
        .bind(&input.description)
        .bind("active")
        .bind(now)
        .bind(now)
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to create department: {}", e))?;

    get_department_by_id_uncached(pool, id).await
}

async fn update_department_uncached(
    pool: &DbPool,
    id: Uuid,
    input: UpdateDepartmentDto,
) -> Result<Department> {
    let query = r#"
        UPDATE departments
        SET name = COALESCE(?, name),
            contact_person = COALESCE(?, contact_person),
            contact_phone = COALESCE(?, contact_phone),
            description = COALESCE(?, description),
            status = COALESCE(?, status),
            updated_at = ?
        WHERE id = ?
    "#;

    sqlx::query(query)
        .bind(input.name)
        .bind(input.contact_person)
        .bind(input.contact_phone)
        .bind(input.description)
        .bind(input.status)
        .bind(Utc::now())
        .bind(id.to_string())
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to update department: {}", e))?;

    get_department_by_id_uncached(pool, id).await
}

async fn delete_department_uncached(pool: &DbPool, id: Uuid) -> Result<()> {
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
        id: Uuid::parse_str(row.get("id")).map_err(|e| anyhow!("Invalid UUID: {}", e))?,
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
