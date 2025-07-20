use crate::common::TestApp;
use axum::http::StatusCode;
use serde_json::json;

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
    let login_data = json!({
        "email": "patient1@example.com",
        "password": "patient123"
    });

    let (status, body) = app.post("/api/v1/auth/login", login_data).await;
    assert_eq!(status, StatusCode::OK);

    let token = body["data"]["token"].as_str().unwrap();

    // List files (should be empty)
    let (status, body) = app.get_with_auth("/api/v1/files", token).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());
}
