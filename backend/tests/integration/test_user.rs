use crate::common::TestApp;
use axum::http::StatusCode;
use backend::{
    models::user::{LoginDto, UpdateUserDto},
    utils::test_helpers::create_test_user,
};
use serde_json::json;

async fn get_auth_token(app: &mut TestApp, account: &str, password: &str) -> String {
    let login_dto = LoginDto {
        account: account.to_string(),
        password: password.to_string(),
    };

    let (status, body) = app.post("/api/v1/auth/login", login_dto).await;
    println!("Login response: status={:?}, body={:?}", status, body);
    assert_eq!(status, StatusCode::OK, "Login failed: {:?}", body);
    body["data"]["token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_get_user_by_id() {
    let mut app = TestApp::new().await;

    // Create test users
    let (patient_id, patient_account, patient_password) =
        create_test_user(&app.pool, "patient").await;
    let (admin_id, admin_account, admin_password) = create_test_user(&app.pool, "admin").await;

    // Get tokens
    let patient_token = get_auth_token(&mut app, &patient_account, &patient_password).await;
    let admin_token = get_auth_token(&mut app, &admin_account, &admin_password).await;

    // Patient can view their own profile
    println!("Patient token: {}", patient_token);
    println!("Requesting: /api/v1/users/{}", patient_id);
    let (status, body) = app
        .get_with_auth(&format!("/api/v1/users/{}", patient_id), &patient_token)
        .await;
    println!("Response: status={:?}, body={:?}", status, body);

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["id"], patient_id.to_string());

    // Patient cannot view other user's profile
    let (status, body) = app
        .get_with_auth(&format!("/api/v1/users/{}", admin_id), &patient_token)
        .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["success"], false);

    // Admin can view any profile
    let (status, body) = app
        .get_with_auth(&format!("/api/v1/users/{}", patient_id), &admin_token)
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["id"], patient_id.to_string());
}

#[tokio::test]
async fn test_list_users_admin_only() {
    let mut app = TestApp::new().await;

    // Create test users
    let (_, patient_account, patient_password) = create_test_user(&app.pool, "patient").await;
    let (_, admin_account, admin_password) = create_test_user(&app.pool, "admin").await;

    // Get tokens
    let patient_token = get_auth_token(&mut app, &patient_account, &patient_password).await;
    let admin_token = get_auth_token(&mut app, &admin_account, &admin_password).await;

    // Patient cannot list users
    let (status, body) = app.get_with_auth("/api/v1/users", &patient_token).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["success"], false);

    // Admin can list users
    let (status, body) = app.get_with_auth("/api/v1/users", &admin_token).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert!(body["data"].is_array());
}

#[tokio::test]
async fn test_update_user() {
    let mut app = TestApp::new().await;

    // Create test user
    let (user_id, user_account, user_password) = create_test_user(&app.pool, "patient").await;
    let token = get_auth_token(&mut app, &user_account, &user_password).await;

    // Update user profile
    let update_dto = UpdateUserDto {
        name: Some("更新后的名字".to_string()),
        gender: Some("女".to_string()),
        phone: Some("13900139000".to_string()),
        email: Some("updated@example.com".to_string()),
        birthday: None,
        status: None,
    };

    let (status, body) = app
        .put_with_auth(&format!("/api/v1/users/{}", user_id), update_dto, &token)
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["name"], "更新后的名字");
    assert_eq!(body["data"]["gender"], "女");
    assert_eq!(body["data"]["phone"], "13900139000");
}

#[tokio::test]
async fn test_delete_user_admin_only() {
    let mut app = TestApp::new().await;

    // Create test users
    let (target_id, _, _) = create_test_user(&app.pool, "patient").await;
    let (_, patient_account, patient_password) = create_test_user(&app.pool, "patient").await;
    let (_, admin_account, admin_password) = create_test_user(&app.pool, "admin").await;

    // Get tokens
    let patient_token = get_auth_token(&mut app, &patient_account, &patient_password).await;
    let admin_token = get_auth_token(&mut app, &admin_account, &admin_password).await;

    // Patient cannot delete users
    let (status, body) = app
        .delete_with_auth(&format!("/api/v1/users/{}", target_id), &patient_token)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["success"], false);

    // Admin can delete users
    let (status, body) = app
        .delete_with_auth(&format!("/api/v1/users/{}", target_id), &admin_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
}

#[tokio::test]
async fn test_batch_delete_users() {
    let mut app = TestApp::new().await;

    // Create test users
    let (user1_id, _, _) = create_test_user(&app.pool, "patient").await;
    let (user2_id, _, _) = create_test_user(&app.pool, "patient").await;
    let (_, admin_account, admin_password) = create_test_user(&app.pool, "admin").await;

    let admin_token = get_auth_token(&mut app, &admin_account, &admin_password).await;

    // Batch delete
    let delete_request = json!({
        "ids": [user1_id, user2_id]
    });

    let (status, body) = app
        .delete_with_auth_body("/api/v1/users/batch/delete", delete_request, &admin_token)
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("2 users deleted"));
}

#[tokio::test]
async fn test_export_users() {
    let mut app = TestApp::new().await;

    // Create test users
    let (_, admin_account, admin_password) = create_test_user(&app.pool, "admin").await;
    create_test_user(&app.pool, "patient").await;
    create_test_user(&app.pool, "doctor").await;

    let admin_token = get_auth_token(&mut app, &admin_account, &admin_password).await;

    // Export users
    let (status, body) = app
        .get_with_auth("/api/v1/users/batch/export", &admin_token)
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert!(body["data"].is_string());
    assert!(body["data"]
        .as_str()
        .unwrap()
        .contains("Account,Name,Gender,Phone,Email,Role,Status,Created At"));
}
