#[cfg(test)]
mod tests {
    use backend::services::cache_service::{CacheKeys, CacheDurations};
    use uuid::Uuid;

    #[test]
    fn test_cache_keys() {
        let user_id = Uuid::new_v4();
        let user_key = CacheKeys::user(&user_id);
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

    #[test]
    fn test_cache_durations() {
        assert_eq!(CacheDurations::SHORT, 300);
        assert_eq!(CacheDurations::MEDIUM, 3600);
        assert_eq!(CacheDurations::LONG, 86400);
        assert_eq!(CacheDurations::SESSION, 7200);
    }
}