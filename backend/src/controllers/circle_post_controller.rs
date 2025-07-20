use crate::middleware::auth::AuthUser;
use crate::models::{ApiResponse, CreateCirclePostDto, CreateCommentDto, UpdateCirclePostDto};
use crate::services::circle_post_service::CirclePostService;
use crate::AppState;
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::Json,
};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize)]
pub struct PostQuery {
    pub circle_id: Option<Uuid>,
    pub author_id: Option<Uuid>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Deserialize)]
pub struct PaginationQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

// Post endpoints
pub async fn create_post(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Json(dto): Json<CreateCirclePostDto>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Validate input
    if let Err(e) = dto.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(&format!("Validation error: {}", e))),
        ));
    }

    let post = CirclePostService::create_post(&state.pool, auth_user.user_id, dto)
        .await
        .map_err(|e| {
            if e.to_string().contains("sensitive words") {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::error("Content contains sensitive words")),
                )
            } else if e.to_string().contains("must be a member") {
                (
                    StatusCode::FORBIDDEN,
                    Json(ApiResponse::error(
                        "You must be a member of the circle to post",
                    )),
                )
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!("Failed to create post: {}", e))),
                )
            }
        })?;

    Ok(Json(ApiResponse::success(
        "Post created successfully",
        serde_json::to_value(&post).unwrap(),
    )))
}

pub async fn get_posts(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Query(query): Query<PostQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(10).min(100);

    let (posts, total) = CirclePostService::get_posts(
        &state.pool,
        query.circle_id,
        query.author_id,
        Some(auth_user.user_id),
        page,
        page_size,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Failed to get posts: {}", e))),
        )
    })?;

    Ok(Json(ApiResponse::success(
        "Posts retrieved successfully",
        serde_json::json!({
            "posts": posts,
            "pagination": {
                "page": page,
                "page_size": page_size,
                "total": total,
                "total_pages": (total as f64 / page_size as f64).ceil() as i64,
            }
        }),
    )))
}

pub async fn get_post_by_id(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    let post = CirclePostService::get_post_by_id(&state.pool, id, Some(auth_user.user_id))
        .await
        .map_err(|e| {
            if e.to_string().contains("not found") {
                (
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::error("Post not found")),
                )
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!("Failed to get post: {}", e))),
                )
            }
        })?;

    Ok(Json(ApiResponse::success(
        "Post retrieved successfully",
        serde_json::to_value(&post).unwrap(),
    )))
}

pub async fn update_post(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateCirclePostDto>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Validate input
    if let Err(e) = dto.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(&format!("Validation error: {}", e))),
        ));
    }

    let post = CirclePostService::update_post(&state.pool, id, auth_user.user_id, dto)
        .await
        .map_err(|e| {
            if e.to_string().contains("Only the author") {
                (
                    StatusCode::FORBIDDEN,
                    Json(ApiResponse::error("Only the author can update the post")),
                )
            } else if e.to_string().contains("sensitive words") {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::error("Content contains sensitive words")),
                )
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!("Failed to update post: {}", e))),
                )
            }
        })?;

    Ok(Json(ApiResponse::success(
        "Post updated successfully",
        serde_json::to_value(&post).unwrap(),
    )))
}

pub async fn delete_post(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    let is_admin = auth_user.role == "admin";
    CirclePostService::delete_post(&state.pool, id, auth_user.user_id, is_admin)
        .await
        .map_err(|e| {
            if e.to_string().contains("No permission") {
                (
                    StatusCode::FORBIDDEN,
                    Json(ApiResponse::error("No permission to delete this post")),
                )
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!("Failed to delete post: {}", e))),
                )
            }
        })?;

    Ok(Json(ApiResponse::success("Post deleted successfully", ())))
}

pub async fn get_user_posts(
    Extension(_auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(user_id): Path<Uuid>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(10).min(100);

    let (posts, total) = CirclePostService::get_user_posts(&state.pool, user_id, page, page_size)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&format!(
                    "Failed to get user posts: {}",
                    e
                ))),
            )
        })?;

    Ok(Json(ApiResponse::success(
        "User posts retrieved successfully",
        serde_json::json!({
            "posts": posts,
            "pagination": {
                "page": page,
                "page_size": page_size,
                "total": total,
                "total_pages": (total as f64 / page_size as f64).ceil() as i64,
            }
        }),
    )))
}

pub async fn get_circle_posts(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(circle_id): Path<Uuid>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(10).min(100);

    let (posts, total) = CirclePostService::get_circle_posts(
        &state.pool,
        circle_id,
        Some(auth_user.user_id),
        page,
        page_size,
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!(
                "Failed to get circle posts: {}",
                e
            ))),
        )
    })?;

    Ok(Json(ApiResponse::success(
        "Circle posts retrieved successfully",
        serde_json::json!({
            "posts": posts,
            "pagination": {
                "page": page,
                "page_size": page_size,
                "total": total,
                "total_pages": (total as f64 / page_size as f64).ceil() as i64,
            }
        }),
    )))
}

// Like endpoints
pub async fn toggle_like(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(post_id): Path<Uuid>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    let liked = CirclePostService::toggle_like(&state.pool, post_id, auth_user.user_id)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&format!("Failed to toggle like: {}", e))),
            )
        })?;

    Ok(Json(ApiResponse::success(
        if liked {
            "Post liked successfully"
        } else {
            "Post unliked successfully"
        },
        serde_json::json!({ "liked": liked }),
    )))
}

// Comment endpoints
pub async fn create_comment(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(post_id): Path<Uuid>,
    Json(dto): Json<CreateCommentDto>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Validate input
    if let Err(e) = dto.validate() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(&format!("Validation error: {}", e))),
        ));
    }

    let comment = CirclePostService::create_comment(&state.pool, post_id, auth_user.user_id, dto)
        .await
        .map_err(|e| {
            if e.to_string().contains("sensitive words") {
                (
                    StatusCode::BAD_REQUEST,
                    Json(ApiResponse::error("Content contains sensitive words")),
                )
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!(
                        "Failed to create comment: {}",
                        e
                    ))),
                )
            }
        })?;

    Ok(Json(ApiResponse::success(
        "Comment created successfully",
        serde_json::to_value(&comment).unwrap(),
    )))
}

pub async fn get_comments(
    State(state): State<AppState>,
    Path(post_id): Path<Uuid>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<ApiResponse<serde_json::Value>>, (StatusCode, Json<ApiResponse<()>>)> {
    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).min(100);

    let (comments, total) = CirclePostService::get_comments(&state.pool, post_id, page, page_size)
        .await
        .map_err(|e| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::error(&format!(
                    "Failed to get comments: {}",
                    e
                ))),
            )
        })?;

    Ok(Json(ApiResponse::success(
        "Comments retrieved successfully",
        serde_json::json!({
            "comments": comments,
            "pagination": {
                "page": page,
                "page_size": page_size,
                "total": total,
                "total_pages": (total as f64 / page_size as f64).ceil() as i64,
            }
        }),
    )))
}

pub async fn delete_comment(
    Extension(auth_user): Extension<AuthUser>,
    State(state): State<AppState>,
    Path(comment_id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    let is_admin = auth_user.role == "admin";
    CirclePostService::delete_comment(&state.pool, comment_id, auth_user.user_id, is_admin)
        .await
        .map_err(|e| {
            if e.to_string().contains("No permission") {
                (
                    StatusCode::FORBIDDEN,
                    Json(ApiResponse::error("No permission to delete this comment")),
                )
            } else if e.to_string().contains("not found") {
                (
                    StatusCode::NOT_FOUND,
                    Json(ApiResponse::error("Comment not found")),
                )
            } else {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!(
                        "Failed to delete comment: {}",
                        e
                    ))),
                )
            }
        })?;

    Ok(Json(ApiResponse::success(
        "Comment deleted successfully",
        (),
    )))
}
