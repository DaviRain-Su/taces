use crate::common::TestApp;
use axum::http::StatusCode;
use serde_json::json;

#[tokio::test]
async fn test_video_consultation_routes_exist() {
    let mut app = TestApp::new().await;

    // Try to access video consultation endpoint without auth - should get 401
    let (status, _) = app.get("/api/v1/video-consultations").await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_video_consultation_with_auth() {
    let mut app = TestApp::new().await;

    // Create a doctor user first
    use backend::utils::test_helpers::{create_test_user, create_test_doctor};
    let (doctor_user_id, doctor_account, doctor_password) = create_test_user(&app.pool, "doctor").await;
    let (_doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;

    // Login with the created user
    let login_data = json!({
        "account": doctor_account,
        "password": doctor_password
    });

    let (status, body) = app.post("/api/v1/auth/login", login_data).await;
    assert_eq!(status, StatusCode::OK);

    let token = body["data"]["token"].as_str().unwrap();

    // List consultations (should be empty or have some data)
    let (status, body) = app
        .get_with_auth("/api/v1/video-consultations", token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());
}
