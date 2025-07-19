use crate::{
    config::database::DbPool,
    models::live_stream::*,
};
use anyhow::{anyhow, Result};
use chrono::Utc;
use uuid::Uuid;

pub async fn list_live_streams(
    pool: &DbPool,
    page: u32,
    per_page: u32,
    status: Option<String>,
    host_id: Option<Uuid>,
) -> Result<Vec<LiveStreamListItem>> {
    let offset = (page - 1) * per_page;

    let mut query = String::from(
        r#"
        SELECT id, title, host_name, scheduled_time, status, created_at
        FROM live_streams
        WHERE 1=1
    "#,
    );

    if let Some(s) = &status {
        query.push_str(&format!(" AND status = '{}'", s));
    }

    if let Some(h_id) = host_id {
        query.push_str(&format!(" AND host_id = '{}'", h_id));
    }

    query.push_str(&format!(
        " ORDER BY scheduled_time DESC LIMIT {} OFFSET {}",
        per_page, offset
    ));

    let rows = sqlx::query(&query)
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch live streams: {}", e))?;

    let mut streams = Vec::new();
    for row in rows {
        streams.push(parse_live_stream_list_item_from_row(&row)?);
    }

    Ok(streams)
}

pub async fn get_live_stream_by_id(pool: &DbPool, id: Uuid) -> Result<LiveStream> {
    let query = r#"
        SELECT id, title, host_id, host_name, scheduled_time, stream_url, 
               qr_code, status, created_at, updated_at
        FROM live_streams
        WHERE id = ?
    "#;

    let row = sqlx::query(query)
        .bind(id.to_string())
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow!("Live stream not found: {}", e))?;

    parse_live_stream_from_row(&row)
}

pub async fn create_live_stream(
    pool: &DbPool,
    host_id: Uuid,
    host_name: String,
    dto: CreateLiveStreamDto,
) -> Result<LiveStream> {
    let stream_id = Uuid::new_v4();
    let now = Utc::now();

    // Validate scheduled time is in the future
    if dto.scheduled_time <= now {
        return Err(anyhow!("Scheduled time must be in the future"));
    }

    let query = r#"
        INSERT INTO live_streams (id, title, host_id, host_name, scheduled_time, 
                                status, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, 'scheduled', ?, ?)
    "#;

    sqlx::query(query)
        .bind(stream_id.to_string())
        .bind(&dto.title)
        .bind(host_id.to_string())
        .bind(&host_name)
        .bind(dto.scheduled_time)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to create live stream: {}", e))?;

    get_live_stream_by_id(pool, stream_id).await
}

pub async fn update_live_stream(
    pool: &DbPool,
    id: Uuid,
    host_id: Uuid,
    is_admin: bool,
    dto: UpdateLiveStreamDto,
) -> Result<LiveStream> {
    // Check permissions
    let existing = get_live_stream_by_id(pool, id).await?;
    if existing.host_id != host_id && !is_admin {
        return Err(anyhow!("Insufficient permissions"));
    }

    // Cannot update if already ended
    if matches!(existing.status, LiveStreamStatus::Ended) {
        return Err(anyhow!("Cannot update ended live stream"));
    }

    let mut update_fields = Vec::new();
    let mut bindings = Vec::new();

    if let Some(title) = &dto.title {
        update_fields.push("title = ?");
        bindings.push(title.clone());
    }

    if let Some(scheduled_time) = dto.scheduled_time {
        // Validate scheduled time is in the future if stream hasn't started
        if matches!(existing.status, LiveStreamStatus::Scheduled) && scheduled_time <= Utc::now() {
            return Err(anyhow!("Scheduled time must be in the future"));
        }
        update_fields.push("scheduled_time = ?");
    }

    if let Some(_stream_url) = &dto.stream_url {
        update_fields.push("stream_url = ?");
    }

    if let Some(_qr_code) = &dto.qr_code {
        update_fields.push("qr_code = ?");
    }

    update_fields.push("updated_at = ?");

    if update_fields.is_empty() {
        return get_live_stream_by_id(pool, id).await;
    }

    let query = format!(
        "UPDATE live_streams SET {} WHERE id = ?",
        update_fields.join(", ")
    );

    let mut query_builder = sqlx::query(&query);

    for binding in bindings {
        query_builder = query_builder.bind(binding);
    }

    if let Some(scheduled_time) = dto.scheduled_time {
        query_builder = query_builder.bind(scheduled_time);
    }

    if let Some(stream_url) = dto.stream_url {
        query_builder = query_builder.bind(stream_url);
    }

    if let Some(qr_code) = dto.qr_code {
        query_builder = query_builder.bind(qr_code);
    }

    query_builder = query_builder.bind(Utc::now());
    query_builder = query_builder.bind(id.to_string());

    query_builder
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to update live stream: {}", e))?;

    get_live_stream_by_id(pool, id).await
}

pub async fn start_live_stream(
    pool: &DbPool,
    id: Uuid,
    host_id: Uuid,
    is_admin: bool,
    dto: StartLiveStreamDto,
) -> Result<LiveStream> {
    // Check permissions
    let existing = get_live_stream_by_id(pool, id).await?;
    if existing.host_id != host_id && !is_admin {
        return Err(anyhow!("Insufficient permissions"));
    }

    // Can only start scheduled streams
    if !matches!(existing.status, LiveStreamStatus::Scheduled) {
        return Err(anyhow!("Live stream is not in scheduled status"));
    }

    let query = r#"
        UPDATE live_streams 
        SET status = 'live', stream_url = ?, qr_code = ?, updated_at = ?
        WHERE id = ?
    "#;

    sqlx::query(query)
        .bind(&dto.stream_url)
        .bind(dto.qr_code)
        .bind(Utc::now())
        .bind(id.to_string())
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to start live stream: {}", e))?;

    get_live_stream_by_id(pool, id).await
}

pub async fn end_live_stream(
    pool: &DbPool,
    id: Uuid,
    host_id: Uuid,
    is_admin: bool,
) -> Result<LiveStream> {
    // Check permissions
    let existing = get_live_stream_by_id(pool, id).await?;
    if existing.host_id != host_id && !is_admin {
        return Err(anyhow!("Insufficient permissions"));
    }

    // Can only end live streams
    if !matches!(existing.status, LiveStreamStatus::Live) {
        return Err(anyhow!("Live stream is not currently live"));
    }

    let query = r#"
        UPDATE live_streams 
        SET status = 'ended', updated_at = ?
        WHERE id = ?
    "#;

    sqlx::query(query)
        .bind(Utc::now())
        .bind(id.to_string())
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to end live stream: {}", e))?;

    get_live_stream_by_id(pool, id).await
}

pub async fn delete_live_stream(
    pool: &DbPool,
    id: Uuid,
    host_id: Uuid,
    is_admin: bool,
) -> Result<()> {
    // Check permissions
    let existing = get_live_stream_by_id(pool, id).await?;
    if existing.host_id != host_id && !is_admin {
        return Err(anyhow!("Insufficient permissions"));
    }

    // Can only delete scheduled streams
    if !matches!(existing.status, LiveStreamStatus::Scheduled) {
        return Err(anyhow!("Can only delete scheduled live streams"));
    }

    let query = "DELETE FROM live_streams WHERE id = ?";

    sqlx::query(query)
        .bind(id.to_string())
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to delete live stream: {}", e))?;

    Ok(())
}

pub async fn get_upcoming_live_streams(
    pool: &DbPool,
    limit: u32,
) -> Result<Vec<LiveStreamListItem>> {
    let query = r#"
        SELECT id, title, host_name, scheduled_time, status, created_at
        FROM live_streams
        WHERE status = 'scheduled' AND scheduled_time > ?
        ORDER BY scheduled_time ASC
        LIMIT ?
    "#;

    let rows = sqlx::query(query)
        .bind(Utc::now())
        .bind(limit)
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch upcoming live streams: {}", e))?;

    let mut streams = Vec::new();
    for row in rows {
        streams.push(parse_live_stream_list_item_from_row(&row)?);
    }

    Ok(streams)
}

// Helper functions for parsing
fn parse_live_stream_from_row(row: &sqlx::mysql::MySqlRow) -> Result<LiveStream> {
    use sqlx::Row;

    Ok(LiveStream {
        id: Uuid::parse_str(row.get("id")).map_err(|e| anyhow!("Failed to parse UUID: {}", e))?,
        title: row.get("title"),
        host_id: Uuid::parse_str(row.get("host_id"))
            .map_err(|e| anyhow!("Failed to parse host UUID: {}", e))?,
        host_name: row.get("host_name"),
        scheduled_time: row.get("scheduled_time"),
        stream_url: row.get("stream_url"),
        qr_code: row.get("qr_code"),
        status: match row.get::<&str, _>("status") {
            "scheduled" => LiveStreamStatus::Scheduled,
            "live" => LiveStreamStatus::Live,
            "ended" => LiveStreamStatus::Ended,
            _ => return Err(anyhow!("Invalid status")),
        },
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

fn parse_live_stream_list_item_from_row(row: &sqlx::mysql::MySqlRow) -> Result<LiveStreamListItem> {
    use sqlx::Row;

    Ok(LiveStreamListItem {
        id: Uuid::parse_str(row.get("id")).map_err(|e| anyhow!("Failed to parse UUID: {}", e))?,
        title: row.get("title"),
        host_name: row.get("host_name"),
        scheduled_time: row.get("scheduled_time"),
        status: match row.get::<&str, _>("status") {
            "scheduled" => LiveStreamStatus::Scheduled,
            "live" => LiveStreamStatus::Live,
            "ended" => LiveStreamStatus::Ended,
            _ => return Err(anyhow!("Invalid status")),
        },
        created_at: row.get("created_at"),
    })
}