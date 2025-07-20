use crate::common::TestApp;
use axum::http::StatusCode;
use backend::{
    models::user::LoginDto,
    utils::test_helpers::create_test_user,
};

async fn get_auth_token(app: &mut TestApp, account: &str, password: &str) -> String {
    let login_dto = LoginDto {
        account: account.to_string(),
        password: password.to_string(),
    };

    let (_, body) = app.post("/api/v1/auth/login", login_dto).await;
    body["data"]["token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_file_upload_routes_exist() {
    let mut app = TestApp::new().await;

    // Try to access file upload endpoint without auth - should get 401
    let (status, _) = app.get("/api/v1/files").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_file_upload_with_auth() {
    let mut app = TestApp::new().await;

    // Create user and login
    let (_user_id, account, password) = create_test_user(&app.pool, "patient").await;
    let token = get_auth_token(&mut app, &account, &password).await;

    // List files (should be empty)
    let (status, body) = app.get_with_auth("/api/v1/files", &token).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());
}
