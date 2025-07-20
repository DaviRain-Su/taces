use crate::config::redis::RedisPool;
use redis::AsyncCommands;
use serde::{Deserialize, Serialize};
use std::time::Duration;

pub struct CacheService;

impl CacheService {
    /// Get a value from cache
    pub async fn get<T: for<'de> Deserialize<'de>>(
        redis: &Option<RedisPool>,
        key: &str,
    ) -> Option<T> {
        let redis = redis.as_ref()?;
        let mut conn = redis.clone();

        match conn.get::<_, String>(key).await {
            Ok(data) => match serde_json::from_str(&data) {
                Ok(value) => Some(value),
                Err(e) => {
                    tracing::warn!("Failed to deserialize cached value: {}", e);
                    None
                }
            },
            Err(e) => {
                tracing::debug!("Cache miss for key {}: {}", key, e);
                None
            }
        }
    }

    /// Set a value in cache with expiration
    pub async fn set<T: Serialize>(
        redis: &Option<RedisPool>,
        key: &str,
        value: &T,
        expiration: Duration,
    ) -> Result<(), String> {
        let redis = redis.as_ref().ok_or("Redis not available")?;
        let mut conn = redis.clone();

        let data = serde_json::to_string(value)
            .map_err(|e| format!("Failed to serialize value: {}", e))?;

        conn.set_ex::<_, _, ()>(key, data, expiration.as_secs())
            .await
            .map_err(|e| format!("Failed to set cache: {}", e))?;

        Ok(())
    }

    /// Delete a value from cache
    pub async fn delete(redis: &Option<RedisPool>, key: &str) -> Result<(), String> {
        let redis = redis.as_ref().ok_or("Redis not available")?;
        let mut conn = redis.clone();

        conn.del::<_, ()>(key)
            .await
            .map_err(|e| format!("Failed to delete cache: {}", e))?;

        Ok(())
    }

    /// Delete multiple values matching a pattern
    pub async fn delete_pattern(redis: &Option<RedisPool>, pattern: &str) -> Result<u64, String> {
        let redis = redis.as_ref().ok_or("Redis not available")?;
        let mut conn = redis.clone();

        // Get all keys matching the pattern
        let keys: Vec<String> = conn
            .keys(pattern)
            .await
            .map_err(|e| format!("Failed to get keys: {}", e))?;

        if keys.is_empty() {
            return Ok(0);
        }

        // Delete all matching keys
        let count: u64 = conn
            .del(&keys)
            .await
            .map_err(|e| format!("Failed to delete keys: {}", e))?;

        Ok(count)
    }

    /// Check if a key exists
    pub async fn exists(redis: &Option<RedisPool>, key: &str) -> bool {
        if let Some(redis) = redis.as_ref() {
            let mut conn = redis.clone();
            matches!(conn.exists::<_, bool>(key).await, Ok(true))
        } else {
            false
        }
    }

    /// Set a value with no expiration
    pub async fn set_persistent<T: Serialize>(
        redis: &Option<RedisPool>,
        key: &str,
        value: &T,
    ) -> Result<(), String> {
        let redis = redis.as_ref().ok_or("Redis not available")?;
        let mut conn = redis.clone();

        let data = serde_json::to_string(value)
            .map_err(|e| format!("Failed to serialize value: {}", e))?;

        conn.set::<_, _, ()>(key, data)
            .await
            .map_err(|e| format!("Failed to set cache: {}", e))?;

        Ok(())
    }

    /// Increment a counter
    pub async fn increment(
        redis: &Option<RedisPool>,
        key: &str,
        increment: i64,
    ) -> Result<i64, String> {
        let redis = redis.as_ref().ok_or("Redis not available")?;
        let mut conn = redis.clone();

        conn.incr(key, increment)
            .await
            .map_err(|e| format!("Failed to increment counter: {}", e))
    }

    /// Get TTL of a key
    pub async fn ttl(redis: &Option<RedisPool>, key: &str) -> Option<i64> {
        let redis = redis.as_ref()?;
        let mut conn = redis.clone();

        match conn.ttl(key).await {
            Ok(ttl) => Some(ttl),
            Err(_) => None,
        }
    }
}

// Cache key builders
pub struct CacheKeys;

impl CacheKeys {
    pub fn user(user_id: &str) -> String {
        format!("user:{}", user_id)
    }

    pub fn user_email(email: &str) -> String {
        format!("user:email:{}", email)
    }

    pub fn doctor(doctor_id: &str) -> String {
        format!("doctor:{}", doctor_id)
    }

    pub fn appointment(appointment_id: &str) -> String {
        format!("appointment:{}", appointment_id)
    }

    pub fn appointment_slots(doctor_id: &str, date: &str) -> String {
        format!("appointment_slots:{}:{}", doctor_id, date)
    }

    pub fn prescription(prescription_id: &str) -> String {
        format!("prescription:{}", prescription_id)
    }

    pub fn department_list() -> String {
        "departments:list".to_string()
    }

    pub fn content_article(article_id: &str) -> String {
        format!("content:article:{}", article_id)
    }

    pub fn content_video(video_id: &str) -> String {
        format!("content:video:{}", video_id)
    }

    pub fn statistics_dashboard() -> String {
        "statistics:dashboard".to_string()
    }

    pub fn statistics_doctor(doctor_id: &str) -> String {
        format!("statistics:doctor:{}", doctor_id)
    }

    pub fn circle_members(circle_id: &str) -> String {
        format!("circle:{}:members", circle_id)
    }

    pub fn user_circles(user_id: &str) -> String {
        format!("user:{}:circles", user_id)
    }

    pub fn webrtc_signals(room_id: &str, user_id: &str) -> String {
        format!("webrtc:signals:{}:{}", room_id, user_id)
    }

    pub fn session(token: &str) -> String {
        format!("session:{}", token)
    }

    pub fn rate_limit(ip: &str, endpoint: &str) -> String {
        format!("rate_limit:{}:{}", ip, endpoint)
    }
}

// Cache durations
pub struct CacheDurations;

impl CacheDurations {
    pub const SHORT: Duration = Duration::from_secs(60); // 1 minute
    pub const MEDIUM: Duration = Duration::from_secs(300); // 5 minutes
    pub const LONG: Duration = Duration::from_secs(3600); // 1 hour
    pub const DAY: Duration = Duration::from_secs(86400); // 24 hours
    pub const WEEK: Duration = Duration::from_secs(604800); // 7 days
}
