#[cfg(test)]
mod tests {
    use backend::services::cache_service::{CacheDurations, CacheKeys};
    use std::time::Duration;
    use uuid::Uuid;

    #[test]
    fn test_cache_keys() {
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

    #[test]
    fn test_cache_durations() {
        assert_eq!(CacheDurations::SHORT, Duration::from_secs(60));
        assert_eq!(CacheDurations::MEDIUM, Duration::from_secs(300));
        assert_eq!(CacheDurations::LONG, Duration::from_secs(3600));
        assert_eq!(CacheDurations::DAY, Duration::from_secs(86400));
        assert_eq!(CacheDurations::WEEK, Duration::from_secs(604800));
    }
}
