#[cfg(test)]
mod tests {
    use backend::utils::jwt::{create_token, decode_token, Claims};
    use uuid::Uuid;

    #[test]
    fn test_create_and_decode_token() {
        let user_id = Uuid::new_v4();
        let role = "patient".to_string();
        let secret = "test_secret_key";
        let expiration = 3600;
        
        let token = create_token(user_id, role.clone(), secret, expiration).unwrap();
        assert!(!token.is_empty());
        
        let claims = decode_token(&token, secret).unwrap();
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.role, role);
    }

    #[test]
    fn test_decode_token_with_wrong_secret() {
        let user_id = Uuid::new_v4();
        let role = "patient".to_string();
        let secret = "test_secret_key";
        let wrong_secret = "wrong_secret_key";
        let expiration = 3600;
        
        let token = create_token(user_id, role, secret, expiration).unwrap();
        let result = decode_token(&token, wrong_secret);
        
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_expired_token() {
        use jsonwebtoken::{encode, Header, EncodingKey};
        use chrono::{Utc, Duration};
        
        let user_id = Uuid::new_v4();
        let role = "patient".to_string();
        let secret = "test_secret_key";
        
        // Create claims with expired timestamp
        let mut claims = Claims::new(user_id, role, 3600);
        claims.exp = (Utc::now() - Duration::hours(1)).timestamp(); // 1 hour ago
        
        // Manually encode the token
        let encoding_key = EncodingKey::from_secret(secret.as_ref());
        let token = encode(&Header::default(), &claims, &encoding_key).unwrap();
        
        let result = decode_token(&token, secret);
        assert!(result.is_err());
    }

    #[test]
    fn test_decode_invalid_token() {
        let secret = "test_secret_key";
        let invalid_token = "invalid.token.here";
        
        let result = decode_token(invalid_token, secret);
        assert!(result.is_err());
    }

    #[test]
    fn test_claims_creation() {
        let user_id = Uuid::new_v4();
        let role = "doctor".to_string();
        let expiration_seconds = 3600;
        
        let claims = Claims::new(user_id, role.clone(), expiration_seconds);
        
        assert_eq!(claims.sub, user_id);
        assert_eq!(claims.role, role);
        assert!(claims.exp > claims.iat);
        assert_eq!(claims.exp - claims.iat, expiration_seconds);
    }
}