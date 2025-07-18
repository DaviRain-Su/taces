use crate::common::TestApp;
use backend::models::user::{CreateUserDto, LoginDto, UserRole};
use http::StatusCode;
use serde_json::json;

#[tokio::test]
async fn test_register_success() {
    let mut app = TestApp::new().await;
    
    let user_dto = CreateUserDto {
        account: "test_user".to_string(),
        name: "测试用户".to_string(),
        password: "password123".to_string(),
        gender: "男".to_string(),
        phone: "13800138000".to_string(),
        email: Some("test@example.com".to_string()),
        birthday: None,
        role: UserRole::Patient,
    };
    
    let (status, body) = app.post("/api/v1/auth/register", user_dto).await;
    
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert_eq!(body["message"], "User registered successfully");
    assert_eq!(body["data"]["account"], "test_user");
    assert_eq!(body["data"]["role"], "patient");
}

#[tokio::test]
async fn test_register_duplicate_account() {
    let mut app = TestApp::new().await;
    
    let user_dto = CreateUserDto {
        account: "duplicate_user".to_string(),
        name: "测试用户1".to_string(),
        password: "password123".to_string(),
        gender: "男".to_string(),
        phone: "13800138001".to_string(),
        email: Some("test1@example.com".to_string()),
        birthday: None,
        role: UserRole::Patient,
    };
    
    // First registration should succeed
    let (status, _) = app.post("/api/v1/auth/register", user_dto.clone()).await;
    assert_eq!(status, StatusCode::OK);
    
    // Second registration with same account should fail
    let mut user_dto2 = user_dto;
    user_dto2.phone = "13800138002".to_string();
    user_dto2.email = Some("test2@example.com".to_string());
    
    let (status, body) = app.post("/api/v1/auth/register", user_dto2).await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(body["success"], false);
}

#[tokio::test]
async fn test_login_success() {
    let mut app = TestApp::new().await;
    
    // Register a user first
    let user_dto = CreateUserDto {
        account: "login_test".to_string(),
        name: "登录测试".to_string(),
        password: "password123".to_string(),
        gender: "女".to_string(),
        phone: "13800138003".to_string(),
        email: Some("login@example.com".to_string()),
        birthday: None,
        role: UserRole::Patient,
    };
    
    let (status, _) = app.post("/api/v1/auth/register", user_dto).await;
    assert_eq!(status, StatusCode::OK);
    
    // Login with correct credentials
    let login_dto = LoginDto {
        account: "login_test".to_string(),
        password: "password123".to_string(),
    };
    
    let (status, body) = app.post("/api/v1/auth/login", login_dto).await;
    
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert_eq!(body["message"], "Login successful");
    assert!(body["data"]["token"].is_string());
    assert_eq!(body["data"]["user"]["account"], "login_test");
}

#[tokio::test]
async fn test_login_wrong_password() {
    let mut app = TestApp::new().await;
    
    // Register a user first
    let user_dto = CreateUserDto {
        account: "wrong_pass_test".to_string(),
        name: "密码错误测试".to_string(),
        password: "correct_password".to_string(),
        gender: "男".to_string(),
        phone: "13800138004".to_string(),
        email: Some("wrong@example.com".to_string()),
        birthday: None,
        role: UserRole::Patient,
    };
    
    let (status, _) = app.post("/api/v1/auth/register", user_dto).await;
    assert_eq!(status, StatusCode::OK);
    
    // Login with wrong password
    let login_dto = LoginDto {
        account: "wrong_pass_test".to_string(),
        password: "wrong_password".to_string(),
    };
    
    let (status, body) = app.post("/api/v1/auth/login", login_dto).await;
    
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["success"], false);
}

#[tokio::test]
async fn test_login_nonexistent_user() {
    let mut app = TestApp::new().await;
    
    let login_dto = LoginDto {
        account: "nonexistent_user".to_string(),
        password: "password123".to_string(),
    };
    
    let (status, body) = app.post("/api/v1/auth/login", login_dto).await;
    
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["success"], false);
}

#[tokio::test]
async fn test_register_validation_errors() {
    let mut app = TestApp::new().await;
    
    // Test with invalid phone
    let user_dto = json!({
        "account": "validation_test",
        "name": "验证测试",
        "password": "pass", // Too short
        "gender": "男",
        "phone": "invalid_phone",
        "role": "patient"
    });
    
    let (status, body) = app.post("/api/v1/auth/register", user_dto).await;
    
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["success"], false);
    assert!(body["message"].as_str().unwrap().contains("Validation error"));
}