use crate::{
    config::database::DbPool,
    models::content::*,
};
use anyhow::{anyhow, Result};
use chrono::Utc;
use serde_json::to_string;
use uuid::Uuid;

// Article services
pub async fn list_articles(
    pool: &DbPool,
    page: u32,
    per_page: u32,
    category: Option<String>,
    status: Option<String>,
    search: Option<String>,
) -> Result<Vec<ArticleListItem>> {
    let offset = (page - 1) * per_page;

    let mut query = String::from(
        r#"
        SELECT id, title, cover_image, summary, author_name, category, 
               view_count, status, published_at, created_at
        FROM articles
        WHERE 1=1
    "#,
    );

    if let Some(cat) = &category {
        query.push_str(&format!(" AND category = '{}'", cat));
    }

    if let Some(s) = &status {
        query.push_str(&format!(" AND status = '{}'", s));
    }

    if let Some(search_term) = &search {
        query.push_str(&format!(
            " AND (title LIKE '%{}%' OR summary LIKE '%{}%')",
            search_term, search_term
        ));
    }

    query.push_str(&format!(
        " ORDER BY published_at DESC, created_at DESC LIMIT {} OFFSET {}",
        per_page, offset
    ));

    let rows = sqlx::query(&query)
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch articles: {}", e))?;

    let mut articles = Vec::new();
    for row in rows {
        articles.push(parse_article_list_item_from_row(&row)?);
    }

    Ok(articles)
}

pub async fn get_article_by_id(pool: &DbPool, id: Uuid) -> Result<Article> {
    let query = r#"
        SELECT id, title, cover_image, summary, content, author_id, author_name, 
               author_type, category, tags, view_count, like_count, status, 
               publish_channels, published_at, created_at, updated_at
        FROM articles
        WHERE id = ?
    "#;

    let row = sqlx::query(query)
        .bind(id.to_string())
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow!("Article not found: {}", e))?;

    // Increment view count
    sqlx::query("UPDATE articles SET view_count = view_count + 1 WHERE id = ?")
        .bind(id.to_string())
        .execute(pool)
        .await?;

    parse_article_from_row(&row)
}

pub async fn create_article(
    pool: &DbPool,
    author_id: Uuid,
    author_name: String,
    author_role: &str,
    dto: CreateArticleDto,
) -> Result<Article> {
    let article_id = Uuid::new_v4();
    let now = Utc::now();
    let author_type = if author_role == "admin" {
        "admin"
    } else {
        "doctor"
    };

    let tags_json = dto.tags.map(|t| to_string(&t).unwrap_or_else(|_| "[]".to_string()));
    let channels_json = dto
        .publish_channels
        .map(|c| to_string(&c).unwrap_or_else(|_| "[]".to_string()));

    let query = r#"
        INSERT INTO articles (id, title, cover_image, summary, content, author_id, 
                            author_name, author_type, category, tags, status, 
                            publish_channels, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'draft', ?, ?, ?)
    "#;

    sqlx::query(query)
        .bind(article_id.to_string())
        .bind(&dto.title)
        .bind(&dto.cover_image)
        .bind(&dto.summary)
        .bind(&dto.content)
        .bind(author_id.to_string())
        .bind(&author_name)
        .bind(author_type)
        .bind(&dto.category)
        .bind(tags_json)
        .bind(channels_json)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to create article: {}", e))?;

    get_article_by_id(pool, article_id).await
}

pub async fn update_article(
    pool: &DbPool,
    id: Uuid,
    author_id: Uuid,
    author_role: &str,
    dto: UpdateArticleDto,
) -> Result<Article> {
    // Check permissions
    let existing = get_article_by_id(pool, id).await?;
    if existing.author_id != author_id && author_role != "admin" {
        return Err(anyhow!("Insufficient permissions"));
    }

    let mut update_fields = Vec::new();
    let mut bindings = Vec::new();

    if let Some(title) = &dto.title {
        update_fields.push("title = ?");
        bindings.push(title.clone());
    }

    if dto.cover_image.is_some() {
        update_fields.push("cover_image = ?");
    }

    if dto.summary.is_some() {
        update_fields.push("summary = ?");
    }

    if let Some(content) = &dto.content {
        update_fields.push("content = ?");
        bindings.push(content.clone());
    }

    if let Some(category) = &dto.category {
        update_fields.push("category = ?");
        bindings.push(category.clone());
    }

    if dto.tags.is_some() {
        update_fields.push("tags = ?");
    }

    if dto.publish_channels.is_some() {
        update_fields.push("publish_channels = ?");
    }

    update_fields.push("updated_at = ?");

    if update_fields.is_empty() {
        return get_article_by_id(pool, id).await;
    }

    let query = format!(
        "UPDATE articles SET {} WHERE id = ?",
        update_fields.join(", ")
    );

    let mut query_builder = sqlx::query(&query);

    for binding in bindings {
        query_builder = query_builder.bind(binding);
    }

    if let Some(cover_image) = dto.cover_image {
        query_builder = query_builder.bind(cover_image);
    }

    if let Some(summary) = dto.summary {
        query_builder = query_builder.bind(summary);
    }

    if let Some(tags) = dto.tags {
        let tags_json = to_string(&tags).unwrap_or_else(|_| "[]".to_string());
        query_builder = query_builder.bind(tags_json);
    }

    if let Some(channels) = dto.publish_channels {
        let channels_json = to_string(&channels).unwrap_or_else(|_| "[]".to_string());
        query_builder = query_builder.bind(channels_json);
    }

    query_builder = query_builder.bind(Utc::now());
    query_builder = query_builder.bind(id.to_string());

    query_builder
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to update article: {}", e))?;

    get_article_by_id(pool, id).await
}

pub async fn publish_article(
    pool: &DbPool,
    id: Uuid,
    author_id: Uuid,
    author_role: &str,
    dto: PublishArticleDto,
) -> Result<Article> {
    // Check permissions
    let existing = get_article_by_id(pool, id).await?;
    if existing.author_id != author_id && author_role != "admin" {
        return Err(anyhow!("Insufficient permissions"));
    }

    let channels_json = to_string(&dto.publish_channels).unwrap_or_else(|_| "[]".to_string());
    let now = Utc::now();

    let query = r#"
        UPDATE articles 
        SET status = 'published', publish_channels = ?, published_at = ?, updated_at = ?
        WHERE id = ?
    "#;

    sqlx::query(query)
        .bind(channels_json)
        .bind(now)
        .bind(now)
        .bind(id.to_string())
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to publish article: {}", e))?;

    get_article_by_id(pool, id).await
}

pub async fn unpublish_article(
    pool: &DbPool,
    id: Uuid,
    author_id: Uuid,
    author_role: &str,
) -> Result<Article> {
    // Check permissions
    let existing = get_article_by_id(pool, id).await?;
    if existing.author_id != author_id && author_role != "admin" {
        return Err(anyhow!("Insufficient permissions"));
    }

    let query = r#"
        UPDATE articles 
        SET status = 'offline', updated_at = ?
        WHERE id = ?
    "#;

    sqlx::query(query)
        .bind(Utc::now())
        .bind(id.to_string())
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to unpublish article: {}", e))?;

    get_article_by_id(pool, id).await
}

pub async fn delete_article(
    pool: &DbPool,
    id: Uuid,
    author_id: Uuid,
    author_role: &str,
) -> Result<()> {
    // Check permissions
    let existing = get_article_by_id(pool, id).await?;
    if existing.author_id != author_id && author_role != "admin" {
        return Err(anyhow!("Insufficient permissions"));
    }

    let query = "DELETE FROM articles WHERE id = ?";

    sqlx::query(query)
        .bind(id.to_string())
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to delete article: {}", e))?;

    Ok(())
}

// Video services
pub async fn list_videos(
    pool: &DbPool,
    page: u32,
    per_page: u32,
    category: Option<String>,
    status: Option<String>,
    search: Option<String>,
) -> Result<Vec<VideoListItem>> {
    let offset = (page - 1) * per_page;

    let mut query = String::from(
        r#"
        SELECT id, title, cover_image, video_url, duration, author_name, 
               category, view_count, status, published_at, created_at
        FROM videos
        WHERE 1=1
    "#,
    );

    if let Some(cat) = &category {
        query.push_str(&format!(" AND category = '{}'", cat));
    }

    if let Some(s) = &status {
        query.push_str(&format!(" AND status = '{}'", s));
    }

    if let Some(search_term) = &search {
        query.push_str(&format!(
            " AND (title LIKE '%{}%' OR description LIKE '%{}%')",
            search_term, search_term
        ));
    }

    query.push_str(&format!(
        " ORDER BY published_at DESC, created_at DESC LIMIT {} OFFSET {}",
        per_page, offset
    ));

    let rows = sqlx::query(&query)
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch videos: {}", e))?;

    let mut videos = Vec::new();
    for row in rows {
        videos.push(parse_video_list_item_from_row(&row)?);
    }

    Ok(videos)
}

pub async fn get_video_by_id(pool: &DbPool, id: Uuid) -> Result<Video> {
    let query = r#"
        SELECT id, title, cover_image, video_url, duration, file_size, description,
               author_id, author_name, author_type, category, tags, view_count, 
               like_count, status, publish_channels, published_at, created_at, updated_at
        FROM videos
        WHERE id = ?
    "#;

    let row = sqlx::query(query)
        .bind(id.to_string())
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow!("Video not found: {}", e))?;

    // Increment view count
    sqlx::query("UPDATE videos SET view_count = view_count + 1 WHERE id = ?")
        .bind(id.to_string())
        .execute(pool)
        .await?;

    parse_video_from_row(&row)
}

pub async fn create_video(
    pool: &DbPool,
    author_id: Uuid,
    author_name: String,
    author_role: &str,
    dto: CreateVideoDto,
) -> Result<Video> {
    let video_id = Uuid::new_v4();
    let now = Utc::now();
    let author_type = if author_role == "admin" {
        "admin"
    } else {
        "doctor"
    };

    let tags_json = dto.tags.map(|t| to_string(&t).unwrap_or_else(|_| "[]".to_string()));
    let channels_json = dto
        .publish_channels
        .map(|c| to_string(&c).unwrap_or_else(|_| "[]".to_string()));

    let query = r#"
        INSERT INTO videos (id, title, cover_image, video_url, duration, file_size,
                          description, author_id, author_name, author_type, category, 
                          tags, status, publish_channels, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 'draft', ?, ?, ?)
    "#;

    sqlx::query(query)
        .bind(video_id.to_string())
        .bind(&dto.title)
        .bind(&dto.cover_image)
        .bind(&dto.video_url)
        .bind(dto.duration)
        .bind(dto.file_size.map(|s| s as i64))
        .bind(&dto.description)
        .bind(author_id.to_string())
        .bind(&author_name)
        .bind(author_type)
        .bind(&dto.category)
        .bind(tags_json)
        .bind(channels_json)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to create video: {}", e))?;

    get_video_by_id(pool, video_id).await
}

pub async fn update_video(
    pool: &DbPool,
    id: Uuid,
    author_id: Uuid,
    author_role: &str,
    dto: UpdateVideoDto,
) -> Result<Video> {
    // Check permissions
    let existing = get_video_by_id(pool, id).await?;
    if existing.author_id != author_id && author_role != "admin" {
        return Err(anyhow!("Insufficient permissions"));
    }

    let mut update_fields = Vec::new();
    let mut bindings = Vec::new();

    if let Some(title) = &dto.title {
        update_fields.push("title = ?");
        bindings.push(title.clone());
    }

    if dto.cover_image.is_some() {
        update_fields.push("cover_image = ?");
    }

    if let Some(video_url) = &dto.video_url {
        update_fields.push("video_url = ?");
        bindings.push(video_url.clone());
    }

    if dto.duration.is_some() {
        update_fields.push("duration = ?");
    }

    if dto.file_size.is_some() {
        update_fields.push("file_size = ?");
    }

    if dto.description.is_some() {
        update_fields.push("description = ?");
    }

    if let Some(category) = &dto.category {
        update_fields.push("category = ?");
        bindings.push(category.clone());
    }

    if dto.tags.is_some() {
        update_fields.push("tags = ?");
    }

    if dto.publish_channels.is_some() {
        update_fields.push("publish_channels = ?");
    }

    update_fields.push("updated_at = ?");

    if update_fields.is_empty() {
        return get_video_by_id(pool, id).await;
    }

    let query = format!("UPDATE videos SET {} WHERE id = ?", update_fields.join(", "));

    let mut query_builder = sqlx::query(&query);

    for binding in bindings {
        query_builder = query_builder.bind(binding);
    }

    if let Some(cover_image) = dto.cover_image {
        query_builder = query_builder.bind(cover_image);
    }

    if let Some(duration) = dto.duration {
        query_builder = query_builder.bind(duration);
    }

    if let Some(file_size) = dto.file_size {
        query_builder = query_builder.bind(file_size as i64);
    }

    if let Some(description) = dto.description {
        query_builder = query_builder.bind(description);
    }

    if let Some(tags) = dto.tags {
        let tags_json = to_string(&tags).unwrap_or_else(|_| "[]".to_string());
        query_builder = query_builder.bind(tags_json);
    }

    if let Some(channels) = dto.publish_channels {
        let channels_json = to_string(&channels).unwrap_or_else(|_| "[]".to_string());
        query_builder = query_builder.bind(channels_json);
    }

    query_builder = query_builder.bind(Utc::now());
    query_builder = query_builder.bind(id.to_string());

    query_builder
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to update video: {}", e))?;

    get_video_by_id(pool, id).await
}

pub async fn publish_video(
    pool: &DbPool,
    id: Uuid,
    author_id: Uuid,
    author_role: &str,
    dto: PublishVideoDto,
) -> Result<Video> {
    // Check permissions
    let existing = get_video_by_id(pool, id).await?;
    if existing.author_id != author_id && author_role != "admin" {
        return Err(anyhow!("Insufficient permissions"));
    }

    let channels_json = to_string(&dto.publish_channels).unwrap_or_else(|_| "[]".to_string());
    let now = Utc::now();

    let query = r#"
        UPDATE videos 
        SET status = 'published', publish_channels = ?, published_at = ?, updated_at = ?
        WHERE id = ?
    "#;

    sqlx::query(query)
        .bind(channels_json)
        .bind(now)
        .bind(now)
        .bind(id.to_string())
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to publish video: {}", e))?;

    get_video_by_id(pool, id).await
}

pub async fn delete_video(
    pool: &DbPool,
    id: Uuid,
    author_id: Uuid,
    author_role: &str,
) -> Result<()> {
    // Check permissions
    let existing = get_video_by_id(pool, id).await?;
    if existing.author_id != author_id && author_role != "admin" {
        return Err(anyhow!("Insufficient permissions"));
    }

    let query = "DELETE FROM videos WHERE id = ?";

    sqlx::query(query)
        .bind(id.to_string())
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to delete video: {}", e))?;

    Ok(())
}

// Category services
pub async fn list_categories(
    pool: &DbPool,
    content_type: Option<String>,
) -> Result<Vec<ContentCategory>> {
    let mut query = String::from(
        r#"
        SELECT id, name, type, sort_order, is_active, created_at, updated_at
        FROM content_categories
        WHERE is_active = true
    "#,
    );

    if let Some(ct) = &content_type {
        query.push_str(&format!(" AND (type = '{}' OR type = 'both')", ct));
    }

    query.push_str(" ORDER BY sort_order ASC, name ASC");

    let rows = sqlx::query(&query)
        .fetch_all(pool)
        .await
        .map_err(|e| anyhow!("Failed to fetch categories: {}", e))?;

    let mut categories = Vec::new();
    for row in rows {
        categories.push(parse_category_from_row(&row)?);
    }

    Ok(categories)
}

pub async fn create_category(pool: &DbPool, dto: CreateCategoryDto) -> Result<ContentCategory> {
    let category_id = Uuid::new_v4();
    let now = Utc::now();
    let sort_order = dto.sort_order.unwrap_or(0);

    let query = r#"
        INSERT INTO content_categories (id, name, type, sort_order, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?)
    "#;

    sqlx::query(query)
        .bind(category_id.to_string())
        .bind(&dto.name)
        .bind(match dto.r#type {
            CategoryType::Article => "article",
            CategoryType::Video => "video",
            CategoryType::Both => "both",
        })
        .bind(sort_order)
        .bind(now)
        .bind(now)
        .execute(pool)
        .await
        .map_err(|e| {
            if e.to_string().contains("Duplicate entry") {
                anyhow!("Category name already exists for this type")
            } else {
                anyhow!("Failed to create category: {}", e)
            }
        })?;

    get_category_by_id(pool, category_id).await
}

async fn get_category_by_id(pool: &DbPool, id: Uuid) -> Result<ContentCategory> {
    let query = r#"
        SELECT id, name, type, sort_order, is_active, created_at, updated_at
        FROM content_categories
        WHERE id = ?
    "#;

    let row = sqlx::query(query)
        .bind(id.to_string())
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow!("Category not found: {}", e))?;

    parse_category_from_row(&row)
}

// Helper functions for parsing
fn parse_article_from_row(row: &sqlx::mysql::MySqlRow) -> Result<Article> {
    use sqlx::Row;

    let tags_value: Option<serde_json::Value> = row.get("tags");
    let tags = tags_value
        .and_then(|v| serde_json::from_value::<Vec<String>>(v).ok())
        .filter(|v| !v.is_empty());

    let channels_value: Option<serde_json::Value> = row.get("publish_channels");
    let publish_channels = channels_value
        .and_then(|v| serde_json::from_value::<Vec<String>>(v).ok())
        .filter(|v| !v.is_empty());

    Ok(Article {
        id: Uuid::parse_str(row.get("id")).map_err(|e| anyhow!("Failed to parse UUID: {}", e))?,
        title: row.get("title"),
        cover_image: row.get("cover_image"),
        summary: row.get("summary"),
        content: row.get("content"),
        author_id: Uuid::parse_str(row.get("author_id"))
            .map_err(|e| anyhow!("Failed to parse author UUID: {}", e))?,
        author_name: row.get("author_name"),
        author_type: match row.get::<&str, _>("author_type") {
            "admin" => AuthorType::Admin,
            "doctor" => AuthorType::Doctor,
            _ => return Err(anyhow!("Invalid author type")),
        },
        category: row.get("category"),
        tags,
        view_count: row.get::<i32, _>("view_count") as u32,
        like_count: row.get::<i32, _>("like_count") as u32,
        status: match row.get::<&str, _>("status") {
            "draft" => ContentStatus::Draft,
            "published" => ContentStatus::Published,
            "offline" => ContentStatus::Offline,
            _ => return Err(anyhow!("Invalid status")),
        },
        publish_channels,
        published_at: row.get("published_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

fn parse_article_list_item_from_row(row: &sqlx::mysql::MySqlRow) -> Result<ArticleListItem> {
    use sqlx::Row;

    Ok(ArticleListItem {
        id: Uuid::parse_str(row.get("id")).map_err(|e| anyhow!("Failed to parse UUID: {}", e))?,
        title: row.get("title"),
        cover_image: row.get("cover_image"),
        summary: row.get("summary"),
        author_name: row.get("author_name"),
        category: row.get("category"),
        view_count: row.get::<i32, _>("view_count") as u32,
        status: match row.get::<&str, _>("status") {
            "draft" => ContentStatus::Draft,
            "published" => ContentStatus::Published,
            "offline" => ContentStatus::Offline,
            _ => return Err(anyhow!("Invalid status")),
        },
        published_at: row.get("published_at"),
        created_at: row.get("created_at"),
    })
}

fn parse_video_from_row(row: &sqlx::mysql::MySqlRow) -> Result<Video> {
    use sqlx::Row;

    let tags_value: Option<serde_json::Value> = row.get("tags");
    let tags = tags_value
        .and_then(|v| serde_json::from_value::<Vec<String>>(v).ok())
        .filter(|v| !v.is_empty());

    let channels_value: Option<serde_json::Value> = row.get("publish_channels");
    let publish_channels = channels_value
        .and_then(|v| serde_json::from_value::<Vec<String>>(v).ok())
        .filter(|v| !v.is_empty());

    Ok(Video {
        id: Uuid::parse_str(row.get("id")).map_err(|e| anyhow!("Failed to parse UUID: {}", e))?,
        title: row.get("title"),
        cover_image: row.get("cover_image"),
        video_url: row.get("video_url"),
        duration: row.get::<Option<i32>, _>("duration").map(|d| d as u32),
        file_size: row.get::<Option<i64>, _>("file_size").map(|s| s as u64),
        description: row.get("description"),
        author_id: Uuid::parse_str(row.get("author_id"))
            .map_err(|e| anyhow!("Failed to parse author UUID: {}", e))?,
        author_name: row.get("author_name"),
        author_type: match row.get::<&str, _>("author_type") {
            "admin" => AuthorType::Admin,
            "doctor" => AuthorType::Doctor,
            _ => return Err(anyhow!("Invalid author type")),
        },
        category: row.get("category"),
        tags,
        view_count: row.get::<i32, _>("view_count") as u32,
        like_count: row.get::<i32, _>("like_count") as u32,
        status: match row.get::<&str, _>("status") {
            "draft" => VideoStatus::Draft,
            "processing" => VideoStatus::Processing,
            "published" => VideoStatus::Published,
            "offline" => VideoStatus::Offline,
            _ => return Err(anyhow!("Invalid status")),
        },
        publish_channels,
        published_at: row.get("published_at"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

fn parse_video_list_item_from_row(row: &sqlx::mysql::MySqlRow) -> Result<VideoListItem> {
    use sqlx::Row;

    Ok(VideoListItem {
        id: Uuid::parse_str(row.get("id")).map_err(|e| anyhow!("Failed to parse UUID: {}", e))?,
        title: row.get("title"),
        cover_image: row.get("cover_image"),
        video_url: row.get("video_url"),
        duration: row.get::<Option<i32>, _>("duration").map(|d| d as u32),
        author_name: row.get("author_name"),
        category: row.get("category"),
        view_count: row.get::<i32, _>("view_count") as u32,
        status: match row.get::<&str, _>("status") {
            "draft" => VideoStatus::Draft,
            "processing" => VideoStatus::Processing,
            "published" => VideoStatus::Published,
            "offline" => VideoStatus::Offline,
            _ => return Err(anyhow!("Invalid status")),
        },
        published_at: row.get("published_at"),
        created_at: row.get("created_at"),
    })
}

fn parse_category_from_row(row: &sqlx::mysql::MySqlRow) -> Result<ContentCategory> {
    use sqlx::Row;

    Ok(ContentCategory {
        id: Uuid::parse_str(row.get("id")).map_err(|e| anyhow!("Failed to parse UUID: {}", e))?,
        name: row.get("name"),
        r#type: match row.get::<&str, _>("type") {
            "article" => CategoryType::Article,
            "video" => CategoryType::Video,
            "both" => CategoryType::Both,
            _ => return Err(anyhow!("Invalid category type")),
        },
        sort_order: row.get("sort_order"),
        is_active: row.get("is_active"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}