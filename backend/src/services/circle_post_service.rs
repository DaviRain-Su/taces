use crate::config::database::DbPool;
use crate::models::{
    CirclePost, CirclePostWithAuthor, CreateCirclePostDto, CreateCommentDto,
    PostComment, PostCommentWithAuthor, PostStatus, UpdateCirclePostDto,
};
use crate::services::circle_service::CircleService;
use anyhow::{anyhow, Result};
use serde_json;
use sqlx::Row;
use uuid::Uuid;

pub struct CirclePostService;

impl CirclePostService {
    // Post CRUD operations
    pub async fn create_post(
        pool: &DbPool,
        author_id: Uuid,
        dto: CreateCirclePostDto,
    ) -> Result<CirclePost> {
        // Check if user is a member of the circle
        let is_member = Self::is_circle_member(pool, dto.circle_id, author_id).await?;
        if !is_member {
            return Err(anyhow!("You must be a member of the circle to post"));
        }

        // Check for sensitive words
        Self::check_sensitive_words(pool, &dto.title).await?;
        Self::check_sensitive_words(pool, &dto.content).await?;

        let mut tx = pool.begin().await?;

        // Create the post
        let post_id = Uuid::new_v4();
        let images_json = serde_json::to_string(&dto.images)?;
        
        sqlx::query(
            r#"
            INSERT INTO circle_posts (id, author_id, circle_id, title, content, images, status)
            VALUES (?, ?, ?, ?, ?, ?, 'active')
            "#,
        )
        .bind(post_id.to_string())
        .bind(author_id.to_string())
        .bind(dto.circle_id.to_string())
        .bind(&dto.title)
        .bind(&dto.content)
        .bind(&images_json)
        .execute(&mut *tx)
        .await?;

        // Update post count in circle
        CircleService::update_post_count(&mut tx, dto.circle_id, 1).await?;

        // Fetch the created post
        let post = sqlx::query(
            r#"
            SELECT id, author_id, circle_id, title, content, images, likes, comments,
                   status, created_at, updated_at
            FROM circle_posts
            WHERE id = ?
            "#,
        )
        .bind(post_id.to_string())
        .fetch_one(&mut *tx)
        .await?;

        let post = parse_post_row(&post)?;

        tx.commit().await?;

        Ok(post)
    }

    pub async fn get_posts(
        pool: &DbPool,
        circle_id: Option<Uuid>,
        author_id: Option<Uuid>,
        user_id: Option<Uuid>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<CirclePostWithAuthor>, i64)> {
        let offset = (page - 1) * page_size;

        // Build query with filters
        let mut count_query = String::from("SELECT COUNT(*) FROM circle_posts p WHERE p.status = 'active'");
        let mut list_query = String::from(
            r#"
            SELECT p.id, p.author_id, p.circle_id, p.title, p.content, p.images,
                   p.likes, p.comments, p.created_at, p.updated_at,
                   u.name as author_name, c.name as circle_name,
                   CASE WHEN pl.id IS NOT NULL THEN TRUE ELSE FALSE END as is_liked
            FROM circle_posts p
            JOIN users u ON p.author_id = u.id
            JOIN circles c ON p.circle_id = c.id
            LEFT JOIN post_likes pl ON p.id = pl.post_id AND pl.user_id = ?
            WHERE p.status = 'active'
            "#,
        );

        let mut params = vec![];

        if let Some(cid) = circle_id {
            count_query.push_str(" AND p.circle_id = ?");
            list_query.push_str(" AND p.circle_id = ?");
            params.push(cid.to_string());
        }

        if let Some(aid) = author_id {
            count_query.push_str(" AND p.author_id = ?");
            list_query.push_str(" AND p.author_id = ?");
            params.push(aid.to_string());
        }

        list_query.push_str(" ORDER BY p.created_at DESC LIMIT ? OFFSET ?");

        // Get total count
        let mut count_query_builder = sqlx::query(&count_query);
        for param in &params {
            count_query_builder = count_query_builder.bind(param);
        }
        let total: i64 = count_query_builder
            .fetch_one(pool)
            .await?
            .get::<i64, _>(0);

        // Get posts list
        let mut list_query_builder = sqlx::query(&list_query)
            .bind(user_id.unwrap_or(Uuid::nil()).to_string());
        for param in params {
            list_query_builder = list_query_builder.bind(param);
        }
        list_query_builder = list_query_builder.bind(page_size).bind(offset);

        let rows = list_query_builder.fetch_all(pool).await?;

        let posts = rows
            .into_iter()
            .map(|row| parse_post_with_author_row(&row))
            .collect::<Result<Vec<_>>>()?;

        Ok((posts, total))
    }

    pub async fn get_post_by_id(
        pool: &DbPool,
        id: Uuid,
        user_id: Option<Uuid>,
    ) -> Result<CirclePostWithAuthor> {
        let row = sqlx::query(
            r#"
            SELECT p.id, p.author_id, p.circle_id, p.title, p.content, p.images,
                   p.likes, p.comments, p.created_at, p.updated_at,
                   u.name as author_name, c.name as circle_name,
                   CASE WHEN pl.id IS NOT NULL THEN TRUE ELSE FALSE END as is_liked
            FROM circle_posts p
            JOIN users u ON p.author_id = u.id
            JOIN circles c ON p.circle_id = c.id
            LEFT JOIN post_likes pl ON p.id = pl.post_id AND pl.user_id = ?
            WHERE p.id = ? AND p.status = 'active'
            "#,
        )
        .bind(user_id.unwrap_or(Uuid::nil()).to_string())
        .bind(id.to_string())
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| anyhow!("Post not found"))?;

        parse_post_with_author_row(&row)
    }

    pub async fn update_post(
        pool: &DbPool,
        id: Uuid,
        author_id: Uuid,
        dto: UpdateCirclePostDto,
    ) -> Result<CirclePost> {
        // Check if user is the author
        let post = Self::get_post_simple(pool, id).await?;
        if post.author_id != author_id {
            return Err(anyhow!("Only the author can update the post"));
        }

        // Check for sensitive words if updating title or content
        if let Some(ref title) = dto.title {
            Self::check_sensitive_words(pool, title).await?;
        }
        if let Some(ref content) = dto.content {
            Self::check_sensitive_words(pool, content).await?;
        }

        // Build dynamic update query
        let mut query = String::from("UPDATE circle_posts SET ");
        let mut first = true;
        
        if dto.title.is_some() {
            if !first { query.push_str(", "); }
            query.push_str("title = ?");
            first = false;
        }
        
        if dto.content.is_some() {
            if !first { query.push_str(", "); }
            query.push_str("content = ?");
            first = false;
        }
        
        if dto.images.is_some() {
            if !first { query.push_str(", "); }
            query.push_str("images = ?");
            first = false;
        }
        
        if first {
            return Err(anyhow!("No fields to update"));
        }
        
        query.push_str(", updated_at = CURRENT_TIMESTAMP WHERE id = ?");
        
        // Bind parameters
        let mut query_builder = sqlx::query(&query);
        
        if let Some(title) = dto.title {
            query_builder = query_builder.bind(title);
        }
        
        if let Some(content) = dto.content {
            query_builder = query_builder.bind(content);
        }
        
        if let Some(images) = dto.images {
            let images_json = serde_json::to_string(&images)?;
            query_builder = query_builder.bind(images_json);
        }
        
        query_builder = query_builder.bind(id.to_string());
        
        query_builder.execute(pool).await?;

        Self::get_post_simple(pool, id).await
    }

    pub async fn delete_post(
        pool: &DbPool,
        id: Uuid,
        user_id: Uuid,
        is_admin: bool,
    ) -> Result<()> {
        let mut tx = pool.begin().await?;

        // Check if user can delete (author or admin)
        let post = Self::get_post_simple(&pool, id).await?;
        if !is_admin && post.author_id != user_id {
            return Err(anyhow!("No permission to delete this post"));
        }

        // Soft delete
        sqlx::query("UPDATE circle_posts SET status = 'deleted' WHERE id = ?")
            .bind(id.to_string())
            .execute(&mut *tx)
            .await?;

        // Update post count
        CircleService::update_post_count(&mut tx, post.circle_id, -1).await?;

        tx.commit().await?;

        Ok(())
    }

    // Like operations
    pub async fn toggle_like(
        pool: &DbPool,
        post_id: Uuid,
        user_id: Uuid,
    ) -> Result<bool> {
        let mut tx = pool.begin().await?;

        // Check if already liked
        let existing = sqlx::query(
            "SELECT id FROM post_likes WHERE post_id = ? AND user_id = ?"
        )
        .bind(post_id.to_string())
        .bind(user_id.to_string())
        .fetch_optional(&mut *tx)
        .await?;

        let liked = if existing.is_some() {
            // Unlike
            sqlx::query("DELETE FROM post_likes WHERE post_id = ? AND user_id = ?")
                .bind(post_id.to_string())
                .bind(user_id.to_string())
                .execute(&mut *tx)
                .await?;

            sqlx::query("UPDATE circle_posts SET likes = likes - 1 WHERE id = ? AND likes > 0")
                .bind(post_id.to_string())
                .execute(&mut *tx)
                .await?;

            false
        } else {
            // Like
            let like_id = Uuid::new_v4();
            sqlx::query(
                "INSERT INTO post_likes (id, post_id, user_id) VALUES (?, ?, ?)"
            )
            .bind(like_id.to_string())
            .bind(post_id.to_string())
            .bind(user_id.to_string())
            .execute(&mut *tx)
            .await?;

            sqlx::query("UPDATE circle_posts SET likes = likes + 1 WHERE id = ?")
                .bind(post_id.to_string())
                .execute(&mut *tx)
                .await?;

            true
        };

        tx.commit().await?;

        Ok(liked)
    }

    // Comment operations
    pub async fn create_comment(
        pool: &DbPool,
        post_id: Uuid,
        user_id: Uuid,
        dto: CreateCommentDto,
    ) -> Result<PostComment> {
        // Check for sensitive words
        Self::check_sensitive_words(pool, &dto.content).await?;

        let mut tx = pool.begin().await?;

        // Create comment
        let comment_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO post_comments (id, post_id, user_id, content)
            VALUES (?, ?, ?, ?)
            "#,
        )
        .bind(comment_id.to_string())
        .bind(post_id.to_string())
        .bind(user_id.to_string())
        .bind(&dto.content)
        .execute(&mut *tx)
        .await?;

        // Update comment count
        sqlx::query("UPDATE circle_posts SET comments = comments + 1 WHERE id = ?")
            .bind(post_id.to_string())
            .execute(&mut *tx)
            .await?;

        // Fetch the created comment
        let comment = sqlx::query(
            r#"
            SELECT id, post_id, user_id, content, is_deleted, created_at, updated_at
            FROM post_comments
            WHERE id = ?
            "#,
        )
        .bind(comment_id.to_string())
        .fetch_one(&mut *tx)
        .await?;

        let comment = parse_comment_row(&comment)?;

        tx.commit().await?;

        Ok(comment)
    }

    pub async fn get_comments(
        pool: &DbPool,
        post_id: Uuid,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<PostCommentWithAuthor>, i64)> {
        let offset = (page - 1) * page_size;

        // Get total count
        let total: i64 = sqlx::query(
            "SELECT COUNT(*) FROM post_comments WHERE post_id = ? AND is_deleted = FALSE"
        )
        .bind(post_id.to_string())
        .fetch_one(pool)
        .await?
        .get(0)?;

        // Get comments
        let rows = sqlx::query(
            r#"
            SELECT c.id, c.post_id, c.user_id, c.content, c.is_deleted,
                   c.created_at, c.updated_at, u.name as user_name
            FROM post_comments c
            JOIN users u ON c.user_id = u.id
            WHERE c.post_id = ? AND c.is_deleted = FALSE
            ORDER BY c.created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(post_id.to_string())
        .bind(page_size)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let comments = rows
            .into_iter()
            .map(|row| parse_comment_with_author_row(&row))
            .collect::<Result<Vec<_>>>()?;

        Ok((comments, total))
    }

    pub async fn delete_comment(
        pool: &DbPool,
        comment_id: Uuid,
        user_id: Uuid,
        is_admin: bool,
    ) -> Result<()> {
        let mut tx = pool.begin().await?;

        // Check if user can delete (author or admin)
        let comment = sqlx::query(
            "SELECT user_id, post_id FROM post_comments WHERE id = ?"
        )
        .bind(comment_id.to_string())
        .fetch_optional(&mut *tx)
        .await?
        .ok_or_else(|| anyhow!("Comment not found"))?;

        let comment_user_id: String = comment.get::<String, _>("user_id");
        let post_id: String = comment.get::<String, _>("post_id");

        if !is_admin && comment_user_id != user_id.to_string() {
            return Err(anyhow!("No permission to delete this comment"));
        }

        // Soft delete
        sqlx::query("UPDATE post_comments SET is_deleted = TRUE WHERE id = ?")
            .bind(comment_id.to_string())
            .execute(&mut *tx)
            .await?;

        // Update comment count
        sqlx::query("UPDATE circle_posts SET comments = comments - 1 WHERE id = ? AND comments > 0")
            .bind(post_id)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }

    // Helper methods
    async fn is_circle_member(pool: &DbPool, circle_id: Uuid, user_id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            "SELECT id FROM circle_members WHERE circle_id = ? AND user_id = ?"
        )
        .bind(circle_id.to_string())
        .bind(user_id.to_string())
        .fetch_optional(pool)
        .await?;

        Ok(result.is_some())
    }

    async fn get_post_simple(pool: &DbPool, id: Uuid) -> Result<CirclePost> {
        let row = sqlx::query(
            r#"
            SELECT id, author_id, circle_id, title, content, images, likes, comments,
                   status, created_at, updated_at
            FROM circle_posts
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_one(pool)
        .await?;

        parse_post_row(&row)
    }

    async fn check_sensitive_words(pool: &DbPool, text: &str) -> Result<()> {
        let sensitive_words: Vec<String> = sqlx::query(
            "SELECT word FROM sensitive_words WHERE is_active = TRUE"
        )
        .fetch_all(pool)
        .await?
        .into_iter()
        .map(|row| row.get::<String, _>("word"))
        .collect::<Vec<String>>();

        let lower_text = text.to_lowercase();
        for word in sensitive_words {
            if lower_text.contains(&word.to_lowercase()) {
                return Err(anyhow!("Content contains sensitive words"));
            }
        }

        Ok(())
    }

    pub async fn get_user_posts(
        pool: &DbPool,
        user_id: Uuid,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<CirclePostWithAuthor>, i64)> {
        Self::get_posts(pool, None, Some(user_id), Some(user_id), page, page_size).await
    }

    pub async fn get_circle_posts(
        pool: &DbPool,
        circle_id: Uuid,
        user_id: Option<Uuid>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<CirclePostWithAuthor>, i64)> {
        Self::get_posts(pool, Some(circle_id), None, user_id, page, page_size).await
    }
}

fn parse_post_row(row: &sqlx::mysql::MySqlRow) -> Result<CirclePost> {
    let id_str: String = row.get("id");
    let author_id_str: String = row.get("author_id");
    let circle_id_str: String = row.get("circle_id");
    let images: serde_json::Value = row.get("images");
    let status_str: String = row.get("status");
    
    Ok(CirclePost {
        id: Uuid::parse_str(&id_str)?,
        author_id: Uuid::parse_str(&author_id_str)?,
        circle_id: Uuid::parse_str(&circle_id_str)?,
        title: row.get("title"),
        content: row.get("content"),
        images: serde_json::from_value(images)?,
        likes: row.get("likes"),
        comments: row.get("comments"),
        status: match status_str.as_str() {
            "active" => PostStatus::Active,
            "deleted" => PostStatus::Deleted,
            _ => return Err(anyhow!("Invalid post status")),
        },
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

fn parse_post_with_author_row(row: &sqlx::mysql::MySqlRow) -> Result<CirclePostWithAuthor> {
    let id_str: String = row.get("id");
    let author_id_str: String = row.get("author_id");
    let circle_id_str: String = row.get("circle_id");
    let images: serde_json::Value = row.get("images");
    
    Ok(CirclePostWithAuthor {
        id: Uuid::parse_str(&id_str)?,
        author_id: Uuid::parse_str(&author_id_str)?,
        author_name: row.get("author_name"),
        circle_id: Uuid::parse_str(&circle_id_str)?,
        circle_name: row.get("circle_name"),
        title: row.get("title"),
        content: row.get("content"),
        images: serde_json::from_value(images)?,
        likes: row.get("likes"),
        comments: row.get("comments"),
        is_liked: row.get("is_liked"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

fn parse_comment_row(row: &sqlx::mysql::MySqlRow) -> Result<PostComment> {
    let id_str: String = row.get("id");
    let post_id_str: String = row.get("post_id");
    let user_id_str: String = row.get("user_id");
    
    Ok(PostComment {
        id: Uuid::parse_str(&id_str)?,
        post_id: Uuid::parse_str(&post_id_str)?,
        user_id: Uuid::parse_str(&user_id_str)?,
        content: row.get("content"),
        is_deleted: row.get("is_deleted"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}

fn parse_comment_with_author_row(row: &sqlx::mysql::MySqlRow) -> Result<PostCommentWithAuthor> {
    let id_str: String = row.get("id");
    let post_id_str: String = row.get("post_id");
    let user_id_str: String = row.get("user_id");
    
    Ok(PostCommentWithAuthor {
        id: Uuid::parse_str(&id_str)?,
        post_id: Uuid::parse_str(&post_id_str)?,
        user_id: Uuid::parse_str(&user_id_str)?,
        user_name: row.get("user_name"),
        content: row.get("content"),
        is_deleted: row.get("is_deleted"),
        created_at: row.get("created_at"),
        updated_at: row.get("updated_at"),
    })
}