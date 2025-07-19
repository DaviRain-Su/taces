use redis::{aio::ConnectionManager, Client};
use std::env;

pub type RedisPool = ConnectionManager;

pub async fn create_redis_pool() -> Result<RedisPool, redis::RedisError> {
    let redis_url = env::var("REDIS_URL").unwrap_or_else(|_| "redis://127.0.0.1:6379".to_string());
    
    let client = Client::open(redis_url)?;
    let connection_manager = ConnectionManager::new(client).await?;
    
    Ok(connection_manager)
}

pub async fn create_redis_pool_optional() -> Option<RedisPool> {
    match create_redis_pool().await {
        Ok(pool) => {
            tracing::info!("Redis connection established");
            Some(pool)
        }
        Err(e) => {
            tracing::warn!("Redis connection failed: {}. Cache features will be disabled.", e);
            None
        }
    }
}