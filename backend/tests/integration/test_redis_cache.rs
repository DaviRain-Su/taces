use backend::{
    config::{database, redis},
    services::{
        cache_service::{CacheService, CacheKeys, CacheDurations},
        session_service::SessionService,
    },
    models::user::{User, UserRole, UserStatus},
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use chrono::Utc;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestData {
    id: String,
    name: String,
    value: i32,
}

#[tokio::test]
async fn test_cache_set_and_get() {
    let redis = redis::create_redis_pool_optional().await;
    
    let test_data = TestData {
        id: "test-1".to_string(),
        name: "Test Item".to_string(),
        value: 42,
    };
    
    let key = "test:cache:item";
    
    // Set data in cache
    let result = CacheService::set(&redis, key, &test_data, CacheDurations::SHORT).await;
    assert!(result.is_ok());
    
    // Get data from cache
    let cached_data: Option<TestData> = CacheService::get(&redis, key).await;
    assert!(cached_data.is_some());
    assert_eq!(cached_data.unwrap(), test_data);
    
    // Clean up
    let _ = CacheService::delete(&redis, key).await;
}

#[tokio::test]
async fn test_cache_delete() {
    let redis = redis::create_redis_pool_optional().await;
    
    let key = "test:cache:delete";
    let value = "test value";
    
    // Set and verify
    let _ = CacheService::set(&redis, key, &value, CacheDurations::SHORT).await;
    assert!(CacheService::exists(&redis, key).await);
    
    // Delete and verify
    let result = CacheService::delete(&redis, key).await;
    assert!(result.is_ok());
    assert!(!CacheService::exists(&redis, key).await);
}

#[tokio::test]
async fn test_cache_exists() {
    let redis = redis::create_redis_pool_optional().await;
    
    let key = "test:cache:exists";
    let value = "test value";
    
    // Should not exist initially
    assert!(!CacheService::exists(&redis, key).await);
    
    // Set and verify existence
    let _ = CacheService::set(&redis, key, &value, CacheDurations::SHORT).await;
    assert!(CacheService::exists(&redis, key).await);
    
    // Clean up
    let _ = CacheService::delete(&redis, key).await;
}

#[tokio::test]
async fn test_cache_delete_pattern() {
    let redis = redis::create_redis_pool_optional().await;
    
    // Set multiple keys with pattern
    let keys = vec![
        "test:pattern:1",
        "test:pattern:2",
        "test:pattern:3",
        "test:other:1",
    ];
    
    for key in &keys {
        let _ = CacheService::set(&redis, key, &"value", CacheDurations::SHORT).await;
    }
    
    // Delete by pattern
    let result = CacheService::delete_pattern(&redis, "test:pattern:*").await;
    assert!(result.is_ok());
    
    // Verify pattern keys deleted
    assert!(!CacheService::exists(&redis, "test:pattern:1").await);
    assert!(!CacheService::exists(&redis, "test:pattern:2").await);
    assert!(!CacheService::exists(&redis, "test:pattern:3").await);
    
    // Verify other key still exists
    assert!(CacheService::exists(&redis, "test:other:1").await);
    
    // Clean up
    let _ = CacheService::delete(&redis, "test:other:1").await;
}

#[tokio::test]
async fn test_session_create_and_get() {
    let redis = redis::create_redis_pool_optional().await;
    
    let user = User {
        id: Uuid::new_v4(),
        account: "test_user".to_string(),
        name: "Test User".to_string(),
        password: "hashed".to_string(),
        gender: "male".to_string(),
        phone: "1234567890".to_string(),
        email: Some("test@example.com".to_string()),
        birthday: None,
        role: UserRole::Patient,
        status: UserStatus::Active,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    let token = "test_token_123";
    
    // Create session
    let result = SessionService::create_session(&redis, token, &user).await;
    assert!(result.is_ok());
    
    // Get session
    let session = SessionService::get_session(&redis, token).await;
    assert!(session.is_some());
    
    let session_data = session.unwrap();
    assert_eq!(session_data.user_id, user.id);
    assert_eq!(session_data.email, user.email.unwrap());
    assert_eq!(session_data.role, "patient");
    
    // Clean up
    let _ = SessionService::invalidate_session(&redis, token).await;
}

#[tokio::test]
async fn test_session_invalidate() {
    let redis = redis::create_redis_pool_optional().await;
    
    let user = User {
        id: Uuid::new_v4(),
        account: "test_user2".to_string(),
        name: "Test User 2".to_string(),
        password: "hashed".to_string(),
        gender: "female".to_string(),
        phone: "0987654321".to_string(),
        email: Some("test2@example.com".to_string()),
        birthday: None,
        role: UserRole::Doctor,
        status: UserStatus::Active,
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };
    
    let token = "test_token_456";
    
    // Create and verify session exists
    let _ = SessionService::create_session(&redis, token, &user).await;
    assert!(SessionService::is_session_valid(&redis, token).await);
    
    // Invalidate session
    let result = SessionService::invalidate_session(&redis, token).await;
    assert!(result.is_ok());
    
    // Verify session no longer exists
    assert!(!SessionService::is_session_valid(&redis, token).await);
    assert!(SessionService::get_session(&redis, token).await.is_none());
}

#[tokio::test]
async fn test_cache_keys() {
    // Test various cache key formats
    let user_id = Uuid::new_v4();
    let user_key = CacheKeys::user(&user_id.to_string());
    assert!(user_key.starts_with("user:"));
    assert!(user_key.contains(&user_id.to_string()));
    
    let email = "test@example.com";
    let email_key = CacheKeys::user_email(email);
    assert!(email_key.starts_with("user:email:"));
    assert!(email_key.contains(email));
    
    let token = "test_token";
    let session_key = CacheKeys::session(token);
    assert!(session_key.starts_with("session:"));
    assert!(session_key.contains(token));
    
    let dept_list_key = CacheKeys::department_list();
    assert_eq!(dept_list_key, "departments:list");
}

#[tokio::test]
async fn test_cache_with_no_redis() {
    // Test that cache operations gracefully handle None redis
    let redis: Option<redis::RedisPool> = None;
    
    let key = "test:no:redis";
    let value = "test value";
    
    // All operations should handle None gracefully
    let set_result = CacheService::set(&redis, key, &value, CacheDurations::SHORT).await;
    assert!(set_result.is_ok());
    
    let get_result: Option<String> = CacheService::get(&redis, key).await;
    assert!(get_result.is_none());
    
    assert!(!CacheService::exists(&redis, key).await);
    
    let delete_result = CacheService::delete(&redis, key).await;
    assert!(delete_result.is_ok());
}