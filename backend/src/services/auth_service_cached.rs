use crate::{
    config::{database::DbPool, redis::RedisPool, Config},
    models::user::*,
    services::{auth_service, session_service::SessionService, user_service_cached},
    utils::jwt,
};
use anyhow::{anyhow, Result};

pub async fn register_user_cached(
    pool: &DbPool,
    redis: &Option<RedisPool>,
    dto: CreateUserDto,
) -> Result<User> {
    // Use regular auth service for registration
    let user = auth_service::register_user(pool, dto).await?;
    
    // Invalidate user list cache
    if let Some(redis_pool) = redis {
        if let Err(e) = redis::cmd("DEL")
            .arg("users:list:*")
            .query_async::<_, ()>(&mut redis_pool.clone())
            .await
        {
            tracing::warn!("Failed to invalidate user list cache: {}", e);
        }
    }
    
    Ok(user)
}

pub async fn login_cached(
    pool: &DbPool,
    redis: &Option<RedisPool>,
    config: &Config,
    dto: LoginDto,
) -> Result<LoginResponse> {
    // Use regular auth service for login but with caching and session creation
    let response = auth_service::login(pool, config, dto).await?;
    
    // Create session in Redis
    if let Err(e) = SessionService::create_session(redis, &response.token, &response.user).await {
        tracing::warn!("Failed to create session: {}", e);
        // Continue even if session creation fails
    }
    
    // Cache the user
    let cache_key = crate::services::cache_service::CacheKeys::user(&response.user.id.to_string());
    if let Err(e) = crate::services::cache_service::CacheService::set(
        redis,
        &cache_key,
        &response.user,
        crate::services::cache_service::CacheDurations::MEDIUM,
    )
    .await
    {
        tracing::warn!("Failed to cache user after login: {}", e);
    }
    
    Ok(response)
}

pub async fn logout_cached(
    redis: &Option<RedisPool>,
    token: &str,
) -> Result<()> {
    // Invalidate session
    SessionService::invalidate_session(redis, token).await?;
    Ok(())
}

pub async fn validate_token_cached(
    pool: &DbPool,
    redis: &Option<RedisPool>,
    config: &Config,
    token: &str,
) -> Result<User> {
    // First check if session exists in Redis
    if let Some(session) = SessionService::get_session(redis, token).await {
        // Session exists, get user from cache or DB
        return user_service_cached::get_user_by_id_cached(pool, redis, session.user_id).await
            .map_err(|e| anyhow!("Failed to get user: {}", e));
    }
    
    // No session found, validate JWT and create session
    let claims = crate::utils::jwt::decode_token(token, &config.jwt_secret)?;
    
    let user = user_service_cached::get_user_by_id_cached(pool, redis, claims.sub).await
        .map_err(|e| anyhow!("Failed to get user: {}", e))?;
    
    // Create session for valid token
    if let Err(e) = SessionService::create_session(redis, token, &user).await {
        tracing::warn!("Failed to create session for existing token: {}", e);
    }
    
    Ok(user)
}