use crate::{
    config::redis::RedisPool,
    models::user::User,
    services::cache_service::{CacheService, CacheKeys, CacheDurations},
    utils::errors::AppError,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SessionData {
    pub user_id: Uuid,
    pub email: String,
    pub role: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub last_accessed: chrono::DateTime<chrono::Utc>,
}

pub struct SessionService;

impl SessionService {
    /// Create a new session
    pub async fn create_session(
        redis: &Option<RedisPool>,
        token: &str,
        user: &User,
    ) -> Result<(), AppError> {
        let session_data = SessionData {
            user_id: user.id,
            email: user.email.clone().unwrap_or_default(),
            role: user.role.to_string(),
            created_at: chrono::Utc::now(),
            last_accessed: chrono::Utc::now(),
        };

        let cache_key = CacheKeys::session(token);
        
        CacheService::set(redis, &cache_key, &session_data, CacheDurations::DAY)
            .await
            .map_err(|e| {
                tracing::warn!("Failed to create session: {}", e);
                AppError::InternalServerError("Failed to create session".to_string())
            })?;

        Ok(())
    }

    /// Get session data
    pub async fn get_session(
        redis: &Option<RedisPool>,
        token: &str,
    ) -> Option<SessionData> {
        let cache_key = CacheKeys::session(token);
        
        if let Some(mut session) = CacheService::get::<SessionData>(redis, &cache_key).await {
            // Update last accessed time
            session.last_accessed = chrono::Utc::now();
            
            // Update in cache (ignore errors)
            let _ = CacheService::set(redis, &cache_key, &session, CacheDurations::DAY).await;
            
            Some(session)
        } else {
            None
        }
    }

    /// Invalidate a session
    pub async fn invalidate_session(
        redis: &Option<RedisPool>,
        token: &str,
    ) -> Result<(), AppError> {
        let cache_key = CacheKeys::session(token);
        
        CacheService::delete(redis, &cache_key)
            .await
            .map_err(|e| {
                tracing::warn!("Failed to invalidate session: {}", e);
                AppError::InternalServerError("Failed to invalidate session".to_string())
            })?;

        Ok(())
    }

    /// Invalidate all sessions for a user
    pub async fn invalidate_user_sessions(
        _redis: &Option<RedisPool>,
        user_id: Uuid,
    ) -> Result<u64, AppError> {
        // This would require maintaining a separate index of sessions per user
        // For now, we'll return 0 as this is a more complex implementation
        tracing::warn!("Invalidating all sessions for user {} not fully implemented", user_id);
        Ok(0)
    }

    /// Check if session is valid
    pub async fn is_session_valid(
        redis: &Option<RedisPool>,
        token: &str,
    ) -> bool {
        CacheService::exists(redis, &CacheKeys::session(token)).await
    }

    /// Extend session TTL
    pub async fn extend_session(
        redis: &Option<RedisPool>,
        token: &str,
    ) -> Result<(), AppError> {
        if let Some(session) = Self::get_session(redis, token).await {
            let cache_key = CacheKeys::session(token);
            
            CacheService::set(redis, &cache_key, &session, CacheDurations::DAY)
                .await
                .map_err(|e| {
                    tracing::warn!("Failed to extend session: {}", e);
                    AppError::InternalServerError("Failed to extend session".to_string())
                })?;
        }

        Ok(())
    }
}