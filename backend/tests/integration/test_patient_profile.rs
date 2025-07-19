use crate::common::TestApp;
use axum::http::StatusCode;
use backend::{models::user::LoginDto, utils::test_helpers::create_test_user};
use serde_json::json;

async fn get_auth_token(app: &mut TestApp, account: &str, password: &str) -> String {
    let login_dto = LoginDto {
        account: account.to_string(),
        password: password.to_string(),
    };

    let (status, body) = app.post("/api/v1/auth/login", login_dto).await;
    assert_eq!(status, StatusCode::OK, "Login failed: {:?}", body);
    body["data"]["token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_patient_profile_crud() {
    let mut app = TestApp::new().await;

    // Create patient user
    let (_patient_id, patient_account, patient_password) =
        create_test_user(&app.pool, "patient").await;
    let patient_token = get_auth_token(&mut app, &patient_account, &patient_password).await;

    // Create first profile (self, should be default)
    let self_profile = json!({
        "name": "张三",
        "id_number": "110101900101123",  // 15-digit ID
        "phone": "13800138000",
        "gender": "男",
        "birthday": "1990-01-01",
        "relationship": "self"
    });

    let (status, body) = app
        .post_with_auth("/api/v1/patient-profiles", self_profile, &patient_token)
        .await;
    println!(
        "Create self profile response: status={:?}, body={:?}",
        status, body
    );
    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());
    let self_profile_id = body["data"]["id"].as_str().unwrap();
    assert!(body["data"]["is_default"].as_bool().unwrap());

    // Create family member profile
    let family_profile = json!({
        "name": "李四",
        "id_number": "110101500101124",  // 15-digit ID
        "phone": "13900139000",
        "gender": "女",
        "birthday": "1950-01-01",
        "relationship": "family"
    });

    let (status, body) = app
        .post_with_auth("/api/v1/patient-profiles", family_profile, &patient_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    let family_profile_id = body["data"]["id"].as_str().unwrap();
    assert!(!body["data"]["is_default"].as_bool().unwrap()); // Should not be default

    // List profiles
    let (status, body) = app
        .get_with_auth("/api/v1/patient-profiles", &patient_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    let profiles = body["data"].as_array().unwrap();
    assert_eq!(profiles.len(), 2);

    // Get specific profile
    let (status, body) = app
        .get_with_auth(
            &format!("/api/v1/patient-profiles/{}", self_profile_id),
            &patient_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["name"].as_str().unwrap(), "张三");

    // Get default profile
    let (status, body) = app
        .get_with_auth("/api/v1/patient-profiles/default", &patient_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["id"].as_str().unwrap(), self_profile_id);

    // Update profile
    let update_dto = json!({
        "phone": "13700137000",
        "relationship": "family"
    });

    let (status, body) = app
        .put_with_auth(
            &format!("/api/v1/patient-profiles/{}", family_profile_id),
            update_dto,
            &patient_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["phone"].as_str().unwrap(), "13700137000");

    // Set family profile as default
    let (status, _) = app
        .put_with_auth(
            &format!("/api/v1/patient-profiles/{}/default", family_profile_id),
            json!({}),
            &patient_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Verify default changed
    let (status, body) = app
        .get_with_auth("/api/v1/patient-profiles/default", &patient_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["id"].as_str().unwrap(), family_profile_id);

    // Try to delete self profile (should fail)
    let (status, body) = app
        .delete_with_auth(
            &format!("/api/v1/patient-profiles/{}", self_profile_id),
            &patient_token,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        body["message"].as_str().unwrap(),
        "Cannot delete self profile"
    );

    // Delete family profile
    let (status, _) = app
        .delete_with_auth(
            &format!("/api/v1/patient-profiles/{}", family_profile_id),
            &patient_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Verify deletion (and self profile becomes default again)
    let (status, body) = app
        .get_with_auth("/api/v1/patient-profiles", &patient_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"].as_array().unwrap().len(), 1);

    let (status, body) = app
        .get_with_auth("/api/v1/patient-profiles/default", &patient_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["id"].as_str().unwrap(), self_profile_id);
}

#[tokio::test]
async fn test_patient_profile_permissions() {
    let mut app = TestApp::new().await;

    // Create different role users
    let (_patient1_id, patient1_account, patient1_password) =
        create_test_user(&app.pool, "patient").await;
    let (_patient2_id, patient2_account, patient2_password) =
        create_test_user(&app.pool, "patient").await;
    let (_doctor_id, doctor_account, doctor_password) = create_test_user(&app.pool, "doctor").await;

    let patient1_token = get_auth_token(&mut app, &patient1_account, &patient1_password).await;
    let patient2_token = get_auth_token(&mut app, &patient2_account, &patient2_password).await;
    let doctor_token = get_auth_token(&mut app, &doctor_account, &doctor_password).await;

    // Patient1 creates a profile
    let profile_dto = json!({
        "name": "患者一",
        "id_number": "110101900101125",
        "phone": "13800138000",
        "gender": "男",
        "relationship": "self"
    });

    let (status, body) = app
        .post_with_auth("/api/v1/patient-profiles", profile_dto, &patient1_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    let profile_id = body["data"]["id"].as_str().unwrap();

    // Doctor cannot create profiles
    let profile_dto = json!({
        "name": "医生测试",
        "id_number": "110101900101126",
        "phone": "13800138001",
        "gender": "男",
        "relationship": "self"
    });
    let (status, _) = app
        .post_with_auth("/api/v1/patient-profiles", profile_dto, &doctor_token)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Patient2 cannot see Patient1's profiles
    let (status, body) = app
        .get_with_auth("/api/v1/patient-profiles", &patient2_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"].as_array().unwrap().len(), 0);

    // Patient2 cannot access Patient1's specific profile
    let (status, _) = app
        .get_with_auth(
            &format!("/api/v1/patient-profiles/{}", profile_id),
            &patient2_token,
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Patient2 cannot update Patient1's profile
    let update_dto = json!({
        "phone": "13700137000"
    });
    let (status, _) = app
        .put_with_auth(
            &format!("/api/v1/patient-profiles/{}", profile_id),
            update_dto,
            &patient2_token,
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Patient2 cannot delete Patient1's profile
    let (status, _) = app
        .delete_with_auth(
            &format!("/api/v1/patient-profiles/{}", profile_id),
            &patient2_token,
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_patient_profile_id_validation() {
    let mut app = TestApp::new().await;

    // Create patient user
    let (_patient_id, patient_account, patient_password) =
        create_test_user(&app.pool, "patient").await;
    let patient_token = get_auth_token(&mut app, &patient_account, &patient_password).await;

    // Test invalid ID number formats
    let invalid_ids = vec![
        "12345",               // Too short
        "1234567890123456789", // Too long
        "11010119900101123X",  // Invalid checksum
        "110101199013011234",  // Invalid month (13)
        "110101199001321234",  // Invalid day
    ];

    for invalid_id in invalid_ids {
        let profile_dto = json!({
            "name": "测试",
            "id_number": invalid_id,
            "phone": "13800138000",
            "gender": "男",
            "relationship": "family"
        });

        let (status, body) = app
            .post_with_auth("/api/v1/patient-profiles", profile_dto, &patient_token)
            .await;
        println!(
            "Testing invalid ID {}: status={:?}, body={:?}",
            invalid_id, status, body
        );
        assert!(
            status == StatusCode::BAD_REQUEST,
            "ID {} should be invalid",
            invalid_id
        );
    }

    // Test valid 15-digit ID (doesn't require checksum)
    let valid_15_digit = json!({
        "name": "测试2",
        "id_number": "110101900101123",   // 15 digits
        "phone": "13800138001",
        "gender": "女",
        "relationship": "friend"
    });

    let (status, _) = app
        .post_with_auth("/api/v1/patient-profiles", valid_15_digit, &patient_token)
        .await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_patient_profile_duplicate_id() {
    let mut app = TestApp::new().await;

    // Create two patient users
    let (_patient1_id, patient1_account, patient1_password) =
        create_test_user(&app.pool, "patient").await;
    let (_patient2_id, patient2_account, patient2_password) =
        create_test_user(&app.pool, "patient").await;

    let patient1_token = get_auth_token(&mut app, &patient1_account, &patient1_password).await;
    let patient2_token = get_auth_token(&mut app, &patient2_account, &patient2_password).await;

    // Patient1 creates a profile
    let profile_dto = json!({
        "name": "张三",
        "id_number": "110101900101127",
        "phone": "13800138000",
        "gender": "男",
        "relationship": "self"
    });

    let (status, _) = app
        .post_with_auth(
            "/api/v1/patient-profiles",
            profile_dto.clone(),
            &patient1_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Patient2 tries to create profile with same ID number
    let (status, body) = app
        .post_with_auth("/api/v1/patient-profiles", profile_dto, &patient2_token)
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        body["message"].as_str().unwrap(),
        "This ID number is already registered"
    );
}

#[tokio::test]
async fn test_patient_profile_first_default() {
    let mut app = TestApp::new().await;

    // Create patient user
    let (_patient_id, patient_account, patient_password) =
        create_test_user(&app.pool, "patient").await;
    let patient_token = get_auth_token(&mut app, &patient_account, &patient_password).await;

    // Create first profile as family (should still be default)
    let family_profile = json!({
        "name": "家人",
        "id_number": "110101900101128",
        "phone": "13800138000",
        "gender": "男",
        "relationship": "family"
    });

    let (status, body) = app
        .post_with_auth("/api/v1/patient-profiles", family_profile, &patient_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body["data"]["is_default"].as_bool().unwrap(),
        "First profile should be default"
    );

    // Create self profile (should become new default)
    let self_profile = json!({
        "name": "本人",
        "id_number": "110101900101129",
        "phone": "13800138001",
        "gender": "男",
        "relationship": "self"
    });

    let (status, body) = app
        .post_with_auth("/api/v1/patient-profiles", self_profile, &patient_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(
        body["data"]["is_default"].as_bool().unwrap(),
        "Self profile should be default"
    );

    // Verify family profile is no longer default
    let (status, body) = app
        .get_with_auth("/api/v1/patient-profiles", &patient_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    let profiles = body["data"].as_array().unwrap();

    for profile in profiles {
        if profile["relationship"] == "family" {
            assert!(
                !profile["is_default"].as_bool().unwrap(),
                "Family profile should not be default"
            );
        }
    }
}
