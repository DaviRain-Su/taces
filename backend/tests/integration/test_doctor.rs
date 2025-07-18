use crate::common::TestApp;
use backend::{
    models::{doctor::*, user::LoginDto},
    utils::test_helpers::{create_test_user, create_test_doctor},
};
use axum::http::StatusCode;
use serde_json::json;
use uuid::Uuid;

async fn get_auth_token(app: &mut TestApp, account: &str, password: &str) -> String {
    let login_dto = LoginDto {
        account: account.to_string(),
        password: password.to_string(),
    };
    
    let (_, body) = app.post("/api/v1/auth/login", login_dto).await;
    body["data"]["token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_create_doctor_profile() {
    let mut app = TestApp::new().await;
    
    // Create admin and doctor users
    let (_, admin_account, admin_password) = create_test_user(&app.pool, "admin").await;
    let (doctor_user_id, _, _) = create_test_user(&app.pool, "doctor").await;
    
    let admin_token = get_auth_token(&mut app, &admin_account, &admin_password).await;
    
    // Create doctor profile
    let doctor_dto = CreateDoctorDto {
        user_id: doctor_user_id,
        certificate_type: "医师资格证".to_string(),
        id_number: "110101199001011234".to_string(),
        hospital: "测试医院".to_string(),
        department: "中医科".to_string(),
        title: "主治医师".to_string(),
        introduction: Some("测试医生简介".to_string()),
        specialties: vec!["中医内科".to_string(), "针灸".to_string()],
        experience: Some("从医10年".to_string()),
    };
    
    let (status, body) = app.post_with_auth("/api/v1/doctors", doctor_dto, &admin_token).await;
    
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["user_id"], doctor_user_id.to_string());
    assert_eq!(body["data"]["hospital"], "测试医院");
}

#[tokio::test]
async fn test_list_doctors() {
    let mut app = TestApp::new().await;
    
    // Create doctor profiles
    let (doctor1_user_id, _, _) = create_test_user(&app.pool, "doctor").await;
    let (doctor2_user_id, _, _) = create_test_user(&app.pool, "doctor").await;
    
    create_test_doctor(&app.pool, doctor1_user_id).await;
    create_test_doctor(&app.pool, doctor2_user_id).await;
    
    // List doctors (no auth required for public listing)
    let (status, body) = app.get("/api/v1/doctors").await;
    
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert!(body["data"].is_array());
    assert!(body["data"].as_array().unwrap().len() >= 2);
}

#[tokio::test]
async fn test_get_doctor_by_id() {
    let mut app = TestApp::new().await;
    
    // Create doctor
    let (doctor_user_id, _, _) = create_test_user(&app.pool, "doctor").await;
    let doctor_id = create_test_doctor(&app.pool, doctor_user_id).await;
    
    // Get doctor by ID
    let (status, body) = app.get(&format!("/api/v1/doctors/{}", doctor_id)).await;
    
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["id"], doctor_id.to_string());
}

#[tokio::test]
async fn test_update_doctor_profile() {
    let mut app = TestApp::new().await;
    
    // Create doctor
    let (doctor_user_id, doctor_account, doctor_password) = create_test_user(&app.pool, "doctor").await;
    let doctor_id = create_test_doctor(&app.pool, doctor_user_id).await;
    
    let doctor_token = get_auth_token(&mut app, &doctor_account, &doctor_password).await;
    
    // Update doctor profile
    let update_dto = UpdateDoctorDto {
        hospital: Some("更新后的医院".to_string()),
        department: Some("针灸推拿科".to_string()),
        title: Some("副主任医师".to_string()),
        introduction: Some("更新后的简介".to_string()),
        specialties: Some(vec!["针灸".to_string(), "推拿".to_string()]),
        experience: Some("从医15年".to_string()),
    };
    
    let (status, body) = app.put_with_auth(
        &format!("/api/v1/doctors/{}", doctor_id),
        update_dto,
        &doctor_token
    ).await;
    
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["hospital"], "更新后的医院");
    assert_eq!(body["data"]["department"], "针灸推拿科");
}

#[tokio::test]
async fn test_update_doctor_photos() {
    let mut app = TestApp::new().await;
    
    // Create doctor
    let (doctor_user_id, doctor_account, doctor_password) = create_test_user(&app.pool, "doctor").await;
    let doctor_id = create_test_doctor(&app.pool, doctor_user_id).await;
    
    let doctor_token = get_auth_token(&mut app, &doctor_account, &doctor_password).await;
    
    // Update doctor photos
    let photos = DoctorPhotos {
        avatar: Some("https://example.com/avatar.jpg".to_string()),
        license_photo: Some("https://example.com/license.jpg".to_string()),
        id_card_front: Some("https://example.com/id_front.jpg".to_string()),
        id_card_back: Some("https://example.com/id_back.jpg".to_string()),
        title_cert: Some("https://example.com/title.jpg".to_string()),
    };
    
    let (status, body) = app.put_with_auth(
        &format!("/api/v1/doctors/{}/photos", doctor_id),
        photos,
        &doctor_token
    ).await;
    
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["avatar"], "https://example.com/avatar.jpg");
}

#[tokio::test]
async fn test_get_doctor_by_user_id() {
    let mut app = TestApp::new().await;
    
    // Create doctor
    let (doctor_user_id, doctor_account, doctor_password) = create_test_user(&app.pool, "doctor").await;
    let doctor_id = create_test_doctor(&app.pool, doctor_user_id).await;
    
    let doctor_token = get_auth_token(&mut app, &doctor_account, &doctor_password).await;
    
    // Get doctor by user ID
    let (status, body) = app.get_with_auth(
        &format!("/api/v1/doctors/by-user/{}", doctor_user_id),
        &doctor_token
    ).await;
    
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["id"], doctor_id.to_string());
    assert_eq!(body["data"]["user_id"], doctor_user_id.to_string());
}