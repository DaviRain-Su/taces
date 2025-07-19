#[cfg(test)]
mod tests {
    use backend::utils::password::{hash_password, verify_password};

    #[test]
    fn test_hash_password() {
        let password = "test_password123";
        let hashed = hash_password(password).unwrap();

        assert_ne!(password, hashed);
        assert!(hashed.len() > 50);
    }

    #[test]
    fn test_verify_password_correct() {
        let password = "test_password123";
        let hashed = hash_password(password).unwrap();

        assert!(verify_password(password, &hashed).unwrap());
    }

    #[test]
    fn test_verify_password_incorrect() {
        let password = "test_password123";
        let wrong_password = "wrong_password";
        let hashed = hash_password(password).unwrap();

        assert!(!verify_password(wrong_password, &hashed).unwrap());
    }

    #[test]
    fn test_different_hashes_for_same_password() {
        let password = "test_password123";
        let hash1 = hash_password(password).unwrap();
        let hash2 = hash_password(password).unwrap();

        assert_ne!(hash1, hash2);
        assert!(verify_password(password, &hash1).unwrap());
        assert!(verify_password(password, &hash2).unwrap());
    }
}
