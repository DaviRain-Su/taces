use crate::{
    middleware::auth::AuthUser,
    models::{content::*, ApiResponse},
    services::content_service,
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    Extension, Json,
};
use serde::Deserialize;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Deserialize)]
pub struct ListQuery {
    page: Option<u32>,
    per_page: Option<u32>,
    category: Option<String>,
    status: Option<String>,
    search: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct CategoryQuery {
    content_type: Option<String>,
}

// Article controllers
pub async fn list_articles(
    State(app_state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ApiResponse<Vec<ArticleListItem>>>, (StatusCode, Json<ApiResponse<()>>)> {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);

    match content_service::list_articles(
        &app_state.pool,
        page,
        per_page,
        query.category,
        query.status,
        query.search,
    )
    .await
    {
        Ok(articles) => Ok(Json(ApiResponse::success(
            "Articles retrieved successfully",
            articles,
        ))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!(
                "Failed to retrieve articles: {}",
                e
            ))),
        )),
    }
}

pub async fn get_article(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Article>>, (StatusCode, Json<ApiResponse<()>>)> {
    match content_service::get_article_by_id(&app_state.pool, id).await {
        Ok(article) => Ok(Json(ApiResponse::success(
            "Article retrieved successfully",
            article,
        ))),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(&format!("Article not found: {}", e))),
        )),
    }
}

pub async fn create_article(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Json(dto): Json<CreateArticleDto>,
) -> Result<Json<ApiResponse<Article>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Only admin and doctor can create articles
    if auth_user.role != "admin" && auth_user.role != "doctor" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Insufficient permissions")),
        ));
    }

    dto.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(&format!("Validation error: {}", e))),
        )
    })?;

    // Get author name
    let author_name = match crate::services::user_service::get_user_by_id(&app_state.pool, auth_user.user_id).await {
        Ok(user) => user.name,
        Err(_) => "Unknown".to_string(),
    };

    match content_service::create_article(
        &app_state.pool,
        auth_user.user_id,
        author_name,
        &auth_user.role,
        dto,
    )
    .await
    {
        Ok(article) => Ok(Json(ApiResponse::success(
            "Article created successfully",
            article,
        ))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!(
                "Failed to create article: {}",
                e
            ))),
        )),
    }
}

pub async fn update_article(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateArticleDto>,
) -> Result<Json<ApiResponse<Article>>, (StatusCode, Json<ApiResponse<()>>)> {
    dto.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(&format!("Validation error: {}", e))),
        )
    })?;

    match content_service::update_article(
        &app_state.pool,
        id,
        auth_user.user_id,
        &auth_user.role,
        dto,
    )
    .await
    {
        Ok(article) => Ok(Json(ApiResponse::success(
            "Article updated successfully",
            article,
        ))),
        Err(e) => {
            if e.to_string().contains("Insufficient permissions") {
                Err((
                    StatusCode::FORBIDDEN,
                    Json(ApiResponse::error("Insufficient permissions")),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!(
                        "Failed to update article: {}",
                        e
                    ))),
                ))
            }
        }
    }
}

pub async fn publish_article(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<PublishArticleDto>,
) -> Result<Json<ApiResponse<Article>>, (StatusCode, Json<ApiResponse<()>>)> {
    match content_service::publish_article(
        &app_state.pool,
        id,
        auth_user.user_id,
        &auth_user.role,
        dto,
    )
    .await
    {
        Ok(article) => Ok(Json(ApiResponse::success(
            "Article published successfully",
            article,
        ))),
        Err(e) => {
            if e.to_string().contains("Insufficient permissions") {
                Err((
                    StatusCode::FORBIDDEN,
                    Json(ApiResponse::error("Insufficient permissions")),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!(
                        "Failed to publish article: {}",
                        e
                    ))),
                ))
            }
        }
    }
}

pub async fn unpublish_article(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Article>>, (StatusCode, Json<ApiResponse<()>>)> {
    match content_service::unpublish_article(
        &app_state.pool,
        id,
        auth_user.user_id,
        &auth_user.role,
    )
    .await
    {
        Ok(article) => Ok(Json(ApiResponse::success(
            "Article unpublished successfully",
            article,
        ))),
        Err(e) => {
            if e.to_string().contains("Insufficient permissions") {
                Err((
                    StatusCode::FORBIDDEN,
                    Json(ApiResponse::error("Insufficient permissions")),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!(
                        "Failed to unpublish article: {}",
                        e
                    ))),
                ))
            }
        }
    }
}

pub async fn delete_article(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    match content_service::delete_article(
        &app_state.pool,
        id,
        auth_user.user_id,
        &auth_user.role,
    )
    .await
    {
        Ok(_) => Ok(Json(ApiResponse::success(
            "Article deleted successfully",
            (),
        ))),
        Err(e) => {
            if e.to_string().contains("Insufficient permissions") {
                Err((
                    StatusCode::FORBIDDEN,
                    Json(ApiResponse::error("Insufficient permissions")),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!(
                        "Failed to delete article: {}",
                        e
                    ))),
                ))
            }
        }
    }
}

// Video controllers
pub async fn list_videos(
    State(app_state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<ApiResponse<Vec<VideoListItem>>>, (StatusCode, Json<ApiResponse<()>>)> {
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);

    match content_service::list_videos(
        &app_state.pool,
        page,
        per_page,
        query.category,
        query.status,
        query.search,
    )
    .await
    {
        Ok(videos) => Ok(Json(ApiResponse::success(
            "Videos retrieved successfully",
            videos,
        ))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!(
                "Failed to retrieve videos: {}",
                e
            ))),
        )),
    }
}

pub async fn get_video(
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<Video>>, (StatusCode, Json<ApiResponse<()>>)> {
    match content_service::get_video_by_id(&app_state.pool, id).await {
        Ok(video) => Ok(Json(ApiResponse::success(
            "Video retrieved successfully",
            video,
        ))),
        Err(e) => Err((
            StatusCode::NOT_FOUND,
            Json(ApiResponse::error(&format!("Video not found: {}", e))),
        )),
    }
}

pub async fn create_video(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Json(dto): Json<CreateVideoDto>,
) -> Result<Json<ApiResponse<Video>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Only admin and doctor can create videos
    if auth_user.role != "admin" && auth_user.role != "doctor" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Insufficient permissions")),
        ));
    }

    dto.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(&format!("Validation error: {}", e))),
        )
    })?;

    // Get author name
    let author_name = match crate::services::user_service::get_user_by_id(&app_state.pool, auth_user.user_id).await {
        Ok(user) => user.name,
        Err(_) => "Unknown".to_string(),
    };

    match content_service::create_video(
        &app_state.pool,
        auth_user.user_id,
        author_name,
        &auth_user.role,
        dto,
    )
    .await
    {
        Ok(video) => Ok(Json(ApiResponse::success(
            "Video created successfully",
            video,
        ))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!("Failed to create video: {}", e))),
        )),
    }
}

pub async fn update_video(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateVideoDto>,
) -> Result<Json<ApiResponse<Video>>, (StatusCode, Json<ApiResponse<()>>)> {
    dto.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(&format!("Validation error: {}", e))),
        )
    })?;

    match content_service::update_video(
        &app_state.pool,
        id,
        auth_user.user_id,
        &auth_user.role,
        dto,
    )
    .await
    {
        Ok(video) => Ok(Json(ApiResponse::success(
            "Video updated successfully",
            video,
        ))),
        Err(e) => {
            if e.to_string().contains("Insufficient permissions") {
                Err((
                    StatusCode::FORBIDDEN,
                    Json(ApiResponse::error("Insufficient permissions")),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!(
                        "Failed to update video: {}",
                        e
                    ))),
                ))
            }
        }
    }
}

pub async fn publish_video(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(dto): Json<PublishVideoDto>,
) -> Result<Json<ApiResponse<Video>>, (StatusCode, Json<ApiResponse<()>>)> {
    match content_service::publish_video(
        &app_state.pool,
        id,
        auth_user.user_id,
        &auth_user.role,
        dto,
    )
    .await
    {
        Ok(video) => Ok(Json(ApiResponse::success(
            "Video published successfully",
            video,
        ))),
        Err(e) => {
            if e.to_string().contains("Insufficient permissions") {
                Err((
                    StatusCode::FORBIDDEN,
                    Json(ApiResponse::error("Insufficient permissions")),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!(
                        "Failed to publish video: {}",
                        e
                    ))),
                ))
            }
        }
    }
}

pub async fn delete_video(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ApiResponse<()>>, (StatusCode, Json<ApiResponse<()>>)> {
    match content_service::delete_video(
        &app_state.pool,
        id,
        auth_user.user_id,
        &auth_user.role,
    )
    .await
    {
        Ok(_) => Ok(Json(ApiResponse::success("Video deleted successfully", ()))),
        Err(e) => {
            if e.to_string().contains("Insufficient permissions") {
                Err((
                    StatusCode::FORBIDDEN,
                    Json(ApiResponse::error("Insufficient permissions")),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!(
                        "Failed to delete video: {}",
                        e
                    ))),
                ))
            }
        }
    }
}

// Category controllers
pub async fn list_categories(
    State(app_state): State<AppState>,
    Query(query): Query<CategoryQuery>,
) -> Result<Json<ApiResponse<Vec<ContentCategory>>>, (StatusCode, Json<ApiResponse<()>>)> {
    match content_service::list_categories(&app_state.pool, query.content_type).await {
        Ok(categories) => Ok(Json(ApiResponse::success(
            "Categories retrieved successfully",
            categories,
        ))),
        Err(e) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::error(&format!(
                "Failed to retrieve categories: {}",
                e
            ))),
        )),
    }
}

pub async fn create_category(
    Extension(auth_user): Extension<AuthUser>,
    State(app_state): State<AppState>,
    Json(dto): Json<CreateCategoryDto>,
) -> Result<Json<ApiResponse<ContentCategory>>, (StatusCode, Json<ApiResponse<()>>)> {
    // Only admin can create categories
    if auth_user.role != "admin" {
        return Err((
            StatusCode::FORBIDDEN,
            Json(ApiResponse::error("Insufficient permissions")),
        ));
    }

    dto.validate().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::error(&format!("Validation error: {}", e))),
        )
    })?;

    match content_service::create_category(&app_state.pool, dto).await {
        Ok(category) => Ok(Json(ApiResponse::success(
            "Category created successfully",
            category,
        ))),
        Err(e) => {
            if e.to_string().contains("already exists") {
                Err((
                    StatusCode::CONFLICT,
                    Json(ApiResponse::error("Category name already exists")),
                ))
            } else {
                Err((
                    StatusCode::INTERNAL_SERVER_ERROR,
                    Json(ApiResponse::error(&format!(
                        "Failed to create category: {}",
                        e
                    ))),
                ))
            }
        }
    }
}