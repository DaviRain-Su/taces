use crate::common::TestApp;
use axum::http::StatusCode;
use backend::{
    models::user::LoginDto,
    utils::test_helpers::{create_test_doctor, create_test_user},
};
use serde_json::json;
use uuid::Uuid;

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
async fn test_patient_group_crud() {
    let mut app = TestApp::new().await;

    // Create doctor and patients
    let (doctor_user_id, doctor_account, doctor_password) =
        create_test_user(&app.pool, "doctor").await;
    let (_doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;
    let doctor_token = get_auth_token(&mut app, &doctor_account, &doctor_password).await;

    let (patient1_id, _, _) = create_test_user(&app.pool, "patient").await;
    let (patient2_id, _, _) = create_test_user(&app.pool, "patient").await;

    // Create patient group
    let create_dto = json!({
        "group_name": "慢性病管理组"
    });

    let (status, body) = app
        .post_with_auth("/api/v1/patient-groups", create_dto, &doctor_token)
        .await;
    println!(
        "Create group response: status={:?}, body={:?}",
        status, body
    );
    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());
    let group_id = body["data"]["id"].as_str().unwrap();

    // List groups
    let (status, body) = app
        .get_with_auth("/api/v1/patient-groups", &doctor_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    let groups = body["data"].as_array().unwrap();
    assert_eq!(groups.len(), 1);
    assert_eq!(groups[0]["group_name"].as_str().unwrap(), "慢性病管理组");
    assert_eq!(groups[0]["member_count"].as_u64().unwrap(), 0);

    // Add members
    let add_members_dto = json!({
        "patient_ids": [patient1_id, patient2_id]
    });

    let (status, body) = app
        .post_with_auth(
            &format!("/api/v1/patient-groups/{}/members", group_id),
            add_members_dto,
            &doctor_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());

    // Get group with members
    let (status, body) = app
        .get_with_auth(
            &format!("/api/v1/patient-groups/{}", group_id),
            &doctor_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["members"].as_array().unwrap().len(), 2);

    // Update group name
    let update_dto = json!({
        "group_name": "糖尿病管理组"
    });

    let (status, body) = app
        .put_with_auth(
            &format!("/api/v1/patient-groups/{}", group_id),
            update_dto,
            &doctor_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["group_name"].as_str().unwrap(), "糖尿病管理组");

    // Remove one member
    let remove_members_dto = json!({
        "patient_ids": [patient1_id]
    });

    let (status, _body) = app
        .delete_with_auth_body(
            &format!("/api/v1/patient-groups/{}/members", group_id),
            remove_members_dto,
            &doctor_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Verify member removed
    let (status, body) = app
        .get_with_auth(
            &format!("/api/v1/patient-groups/{}", group_id),
            &doctor_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["members"].as_array().unwrap().len(), 1);

    // Delete group
    let (status, body) = app
        .delete_with_auth(
            &format!("/api/v1/patient-groups/{}", group_id),
            &doctor_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());

    // Verify deletion
    let (status, body) = app
        .get_with_auth("/api/v1/patient-groups", &doctor_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_patient_group_limit() {
    let mut app = TestApp::new().await;

    // Create doctor
    let (doctor_user_id, doctor_account, doctor_password) =
        create_test_user(&app.pool, "doctor").await;
    let (_doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;
    let doctor_token = get_auth_token(&mut app, &doctor_account, &doctor_password).await;

    // Create 5 groups (maximum allowed)
    for i in 1..=5 {
        let create_dto = json!({
            "group_name": format!("分组{}", i)
        });

        let (status, _) = app
            .post_with_auth("/api/v1/patient-groups", create_dto, &doctor_token)
            .await;
        assert_eq!(status, StatusCode::OK);
    }

    // Try to create 6th group
    let create_dto = json!({
        "group_name": "分组6"
    });

    let (status, body) = app
        .post_with_auth("/api/v1/patient-groups", create_dto, &doctor_token)
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(
        body["message"].as_str().unwrap(),
        "Maximum 5 patient groups allowed"
    );
}

#[tokio::test]
async fn test_patient_group_permissions() {
    let mut app = TestApp::new().await;

    // Create two doctors
    let (doctor1_user_id, doctor1_account, doctor1_password) =
        create_test_user(&app.pool, "doctor").await;
    let (_doctor1_id, _) = create_test_doctor(&app.pool, doctor1_user_id).await;
    let doctor1_token = get_auth_token(&mut app, &doctor1_account, &doctor1_password).await;

    let (doctor2_user_id, doctor2_account, doctor2_password) =
        create_test_user(&app.pool, "doctor").await;
    let (_doctor2_id, _) = create_test_doctor(&app.pool, doctor2_user_id).await;
    let doctor2_token = get_auth_token(&mut app, &doctor2_account, &doctor2_password).await;

    // Create patient
    let (_patient_id, patient_account, patient_password) =
        create_test_user(&app.pool, "patient").await;
    let patient_token = get_auth_token(&mut app, &patient_account, &patient_password).await;

    // Doctor1 creates a group
    let create_dto = json!({
        "group_name": "医生1的分组"
    });

    let (status, body) = app
        .post_with_auth("/api/v1/patient-groups", create_dto, &doctor1_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    let group_id = body["data"]["id"].as_str().unwrap();

    // Patient cannot create groups
    let create_dto = json!({
        "group_name": "患者的分组"
    });
    let (status, _) = app
        .post_with_auth("/api/v1/patient-groups", create_dto, &patient_token)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Doctor2 cannot see Doctor1's groups
    let (status, body) = app
        .get_with_auth("/api/v1/patient-groups", &doctor2_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"].as_array().unwrap().len(), 0);

    // Doctor2 cannot update Doctor1's group
    let update_dto = json!({
        "group_name": "尝试修改"
    });
    let (status, _) = app
        .put_with_auth(
            &format!("/api/v1/patient-groups/{}", group_id),
            update_dto,
            &doctor2_token,
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);

    // Doctor2 cannot delete Doctor1's group
    let (status, _) = app
        .delete_with_auth(
            &format!("/api/v1/patient-groups/{}", group_id),
            &doctor2_token,
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_patient_group_duplicate_name() {
    let mut app = TestApp::new().await;

    // Create doctor
    let (doctor_user_id, doctor_account, doctor_password) =
        create_test_user(&app.pool, "doctor").await;
    let (_doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;
    let doctor_token = get_auth_token(&mut app, &doctor_account, &doctor_password).await;

    // Create first group
    let create_dto = json!({
        "group_name": "重复名称组"
    });

    let (status, _) = app
        .post_with_auth("/api/v1/patient-groups", create_dto.clone(), &doctor_token)
        .await;
    assert_eq!(status, StatusCode::OK);

    // Try to create group with same name
    let (status, body) = app
        .post_with_auth("/api/v1/patient-groups", create_dto, &doctor_token)
        .await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(
        body["message"].as_str().unwrap(),
        "Group name already exists"
    );
}

#[tokio::test]
async fn test_patient_group_send_message() {
    let mut app = TestApp::new().await;

    // Create doctor and patients
    let (doctor_user_id, doctor_account, doctor_password) =
        create_test_user(&app.pool, "doctor").await;
    let (_doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;
    let doctor_token = get_auth_token(&mut app, &doctor_account, &doctor_password).await;

    let (patient1_id, _, _) = create_test_user(&app.pool, "patient").await;
    let (patient2_id, _, _) = create_test_user(&app.pool, "patient").await;

    // Create group and add members
    let create_dto = json!({
        "group_name": "消息测试组"
    });

    let (status, body) = app
        .post_with_auth("/api/v1/patient-groups", create_dto, &doctor_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    let group_id = body["data"]["id"].as_str().unwrap();

    let add_members_dto = json!({
        "patient_ids": [patient1_id, patient2_id]
    });

    app.post_with_auth(
        &format!("/api/v1/patient-groups/{}/members", group_id),
        add_members_dto,
        &doctor_token,
    )
    .await;

    // Send message
    let message_dto = json!({
        "message": "各位患者，请记得明天来复诊"
    });

    let (status, body) = app
        .post_with_auth(
            &format!("/api/v1/patient-groups/{}/message", group_id),
            message_dto,
            &doctor_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());

    // Should return phone numbers that would receive the message
    let phone_numbers = body["data"].as_array().unwrap();
    assert_eq!(phone_numbers.len(), 2);
}

#[tokio::test]
async fn test_patient_group_member_validation() {
    let mut app = TestApp::new().await;

    // Create doctor and non-patient user
    let (doctor_user_id, doctor_account, doctor_password) =
        create_test_user(&app.pool, "doctor").await;
    let (_doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;
    let doctor_token = get_auth_token(&mut app, &doctor_account, &doctor_password).await;

    let (admin_id, _, _) = create_test_user(&app.pool, "admin").await;

    // Create group
    let create_dto = json!({
        "group_name": "验证测试组"
    });

    let (status, body) = app
        .post_with_auth("/api/v1/patient-groups", create_dto, &doctor_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    let group_id = body["data"]["id"].as_str().unwrap();

    // Try to add non-patient user
    let add_members_dto = json!({
        "patient_ids": [admin_id]
    });

    let (status, body) = app
        .post_with_auth(
            &format!("/api/v1/patient-groups/{}/members", group_id),
            add_members_dto,
            &doctor_token,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["message"].as_str().unwrap().contains("not a patient"));

    // Try to add non-existent user
    let fake_id = Uuid::new_v4();
    let add_members_dto = json!({
        "patient_ids": [fake_id]
    });

    let (status, body) = app
        .post_with_auth(
            &format!("/api/v1/patient-groups/{}/members", group_id),
            add_members_dto,
            &doctor_token,
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body["message"].as_str().unwrap().contains("not found"));
}
