use crate::{
    config::{database::DbPool, redis::RedisPool},
    models::user::*,
    services::{
        cache_service::{CacheDurations, CacheKeys, CacheService},
        user_service,
    },
    utils::errors::AppError,
};
use uuid::Uuid;

/// Get user by ID with caching
pub async fn get_user_by_id_cached(
    pool: &DbPool,
    redis: &Option<RedisPool>,
    id: Uuid,
) -> Result<User, AppError> {
    let cache_key = CacheKeys::user(&id.to_string());

    // Try cache first
    if let Some(user) = CacheService::get::<User>(redis, &cache_key).await {
        tracing::debug!("Cache hit for user {}", id);
        return Ok(user);
    }

    // Cache miss, fetch from database
    let user = user_service::get_user_by_id(pool, id)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Store in cache
    if let Err(e) = CacheService::set(redis, &cache_key, &user, CacheDurations::MEDIUM).await {
        tracing::warn!("Failed to cache user: {}", e);
    }

    Ok(user)
}

/// Get user by email with caching
pub async fn get_user_by_email_cached(
    pool: &DbPool,
    redis: &Option<RedisPool>,
    email: &str,
) -> Result<User, AppError> {
    let cache_key = CacheKeys::user_email(email);

    // Try cache first
    if let Some(user) = CacheService::get::<User>(redis, &cache_key).await {
        tracing::debug!("Cache hit for user email {}", email);
        return Ok(user);
    }

    // Cache miss, fetch from database
    let user = user_service::get_user_by_email(pool, email)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Store in cache (both by email and by ID)
    if let Err(e) = CacheService::set(redis, &cache_key, &user, CacheDurations::MEDIUM).await {
        tracing::warn!("Failed to cache user by email: {}", e);
    }

    // Also cache by ID
    let id_cache_key = CacheKeys::user(&user.id.to_string());
    if let Err(e) = CacheService::set(redis, &id_cache_key, &user, CacheDurations::MEDIUM).await {
        tracing::warn!("Failed to cache user by ID: {}", e);
    }

    Ok(user)
}

/// Create user and invalidate relevant caches
pub async fn create_user_cached(
    pool: &DbPool,
    redis: &Option<RedisPool>,
    dto: CreateUserDto,
) -> Result<User, AppError> {
    let user = user_service::create_user(pool, dto)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Invalidate user list cache
    if let Err(e) = CacheService::delete_pattern(redis, "users:list:*").await {
        tracing::warn!("Failed to invalidate user list cache: {}", e);
    }

    Ok(user)
}

/// Update user and invalidate caches
pub async fn update_user_cached(
    pool: &DbPool,
    redis: &Option<RedisPool>,
    id: Uuid,
    dto: UpdateUserDto,
) -> Result<User, AppError> {
    let user = user_service::update_user(pool, id, dto)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Invalidate user caches
    let cache_key = CacheKeys::user(&id.to_string());
    if let Err(e) = CacheService::delete(redis, &cache_key).await {
        tracing::warn!("Failed to invalidate user cache: {}", e);
    }

    // Invalidate email cache
    if let Some(email) = &user.email {
        let email_cache_key = CacheKeys::user_email(email);
        if let Err(e) = CacheService::delete(redis, &email_cache_key).await {
            tracing::warn!("Failed to invalidate user email cache: {}", e);
        }
    }

    // Invalidate user list cache
    if let Err(e) = CacheService::delete_pattern(redis, "users:list:*").await {
        tracing::warn!("Failed to invalidate user list cache: {}", e);
    }

    Ok(user)
}

/// Delete user and invalidate caches
pub async fn delete_user_cached(
    pool: &DbPool,
    redis: &Option<RedisPool>,
    id: Uuid,
) -> Result<(), AppError> {
    // Get user first to get email for cache invalidation
    if let Ok(user) = user_service::get_user_by_id(pool, id).await {
        // Delete user
        user_service::delete_user(pool, id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Invalidate user caches
        let cache_key = CacheKeys::user(&id.to_string());
        if let Err(e) = CacheService::delete(redis, &cache_key).await {
            tracing::warn!("Failed to invalidate user cache: {}", e);
        }

        // Invalidate email cache
        if let Some(email) = &user.email {
            let email_cache_key = CacheKeys::user_email(email);
            if let Err(e) = CacheService::delete(redis, &email_cache_key).await {
                tracing::warn!("Failed to invalidate user email cache: {}", e);
            }
        }

        // Invalidate user list cache
        if let Err(e) = CacheService::delete_pattern(redis, "users:list:*").await {
            tracing::warn!("Failed to invalidate user list cache: {}", e);
        }
    } else {
        user_service::delete_user(pool, id)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;
    }

    Ok(())
}

/// List users with caching (only for default queries)
pub async fn list_users_cached(
    pool: &DbPool,
    redis: &Option<RedisPool>,
    page: u32,
    per_page: u32,
    search: Option<String>,
    role: Option<String>,
) -> Result<Vec<User>, AppError> {
    // For searches and filters, skip cache
    if search.is_some() || role.is_some() {
        return user_service::list_users(pool, page, per_page, search, role, None)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()));
    }

    // Try to get from cache for default listing
    let cache_key = format!("users:list:page{}:size{}", page, per_page);

    if let Some(users) = CacheService::get::<Vec<User>>(redis, &cache_key).await {
        tracing::debug!("Cache hit for users list");
        return Ok(users);
    }

    // Cache miss, fetch from database
    let users = user_service::list_users(pool, page, per_page, None, None, None)
        .await
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

    // Store in cache
    if let Err(e) = CacheService::set(redis, &cache_key, &users, CacheDurations::SHORT).await {
        tracing::warn!("Failed to cache users list: {}", e);
    }

    Ok(users)
}
