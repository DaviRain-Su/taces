use crate::common::TestApp;
use axum::http::StatusCode;
use backend::{
    models::{appointment::*, user::LoginDto},
    utils::test_helpers::{create_test_doctor, create_test_user},
};
use chrono::{Duration, Utc};
use serde_json::json;

async fn get_auth_token(app: &mut TestApp, account: &str, password: &str) -> String {
    let login_dto = LoginDto {
        account: account.to_string(),
        password: password.to_string(),
    };

    let (_, body) = app.post("/api/v1/auth/login", login_dto).await;
    body["data"]["token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_create_appointment() {
    let mut app = TestApp::new().await;

    // Create patient and doctor
    let (patient_user_id, patient_account, patient_password) =
        create_test_user(&app.pool, "patient").await;
    let (doctor_user_id, _, _) = create_test_user(&app.pool, "doctor").await;
    let (doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;

    let patient_token = get_auth_token(&mut app, &patient_account, &patient_password).await;

    // Create appointment
    let tomorrow = Utc::now() + Duration::days(1);
    let appointment_dto = CreateAppointmentDto {
        patient_id: patient_user_id,
        doctor_id: doctor_id,
        appointment_date: tomorrow,
        time_slot: "09:00-10:00".to_string(),
        visit_type: VisitType::Offline,
        symptoms: "头痛、失眠".to_string(),
        has_visited_before: false,
    };

    let (status, body) = app
        .post_with_auth("/api/v1/appointments", appointment_dto, &patient_token)
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["patient_id"], patient_user_id.to_string());
    assert_eq!(body["data"]["doctor_id"], doctor_id.to_string());
    assert_eq!(body["data"]["status"], "pending");
}

#[tokio::test]
async fn test_list_appointments() {
    let mut app = TestApp::new().await;

    // Create admin user
    let (_, admin_account, admin_password) = create_test_user(&app.pool, "admin").await;
    let admin_token = get_auth_token(&mut app, &admin_account, &admin_password).await;

    // Create appointments
    let (patient_user_id, _, _) = create_test_user(&app.pool, "patient").await;
    let (doctor_user_id, _, _) = create_test_user(&app.pool, "doctor").await;
    let (doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;

    // Create multiple appointments
    for i in 0..3 {
        let appointment = CreateAppointmentDto {
            patient_id: patient_user_id,
            doctor_id: doctor_id,
            appointment_date: Utc::now() + Duration::days(i + 1),
            time_slot: format!("{}:00-{}:00", 9 + i, 10 + i),
            visit_type: VisitType::Offline,
            symptoms: "测试症状".to_string(),
            has_visited_before: false,
        };

        let _ = app
            .post_with_auth("/api/v1/appointments", appointment, &admin_token)
            .await;
    }

    // List appointments
    let (status, body) = app
        .get_with_auth("/api/v1/appointments?page=1&page_size=10", &admin_token)
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert!(body["data"]["items"].is_array());
    assert!(body["data"]["items"].as_array().unwrap().len() >= 3);
}

#[tokio::test]
async fn test_get_appointment_by_id() {
    let mut app = TestApp::new().await;

    // Create appointment
    let (patient_user_id, patient_account, patient_password) =
        create_test_user(&app.pool, "patient").await;
    let (doctor_user_id, _, _) = create_test_user(&app.pool, "doctor").await;
    let (doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;

    let patient_token = get_auth_token(&mut app, &patient_account, &patient_password).await;

    let appointment_dto = CreateAppointmentDto {
        patient_id: patient_user_id,
        doctor_id: doctor_id,
        appointment_date: Utc::now() + Duration::days(1),
        time_slot: "09:00-10:00".to_string(),
        visit_type: VisitType::Offline,
        symptoms: "测试症状".to_string(),
        has_visited_before: false,
    };

    let (_, create_body) = app
        .post_with_auth("/api/v1/appointments", appointment_dto, &patient_token)
        .await;

    let appointment_id = create_body["data"]["id"].as_str().unwrap();

    // Get appointment by ID
    let (status, body) = app
        .get_with_auth(
            &format!("/api/v1/appointments/{}", appointment_id),
            &patient_token,
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["id"], appointment_id);
}

#[tokio::test]
async fn test_update_appointment() {
    let mut app = TestApp::new().await;

    // Create appointment
    let (patient_user_id, patient_account, patient_password) =
        create_test_user(&app.pool, "patient").await;
    let (doctor_user_id, _, _) = create_test_user(&app.pool, "doctor").await;
    let (doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;

    let patient_token = get_auth_token(&mut app, &patient_account, &patient_password).await;

    let appointment_dto = CreateAppointmentDto {
        patient_id: patient_user_id,
        doctor_id: doctor_id,
        appointment_date: Utc::now() + Duration::days(1),
        time_slot: "09:00-10:00".to_string(),
        visit_type: VisitType::Offline,
        symptoms: "原始症状".to_string(),
        has_visited_before: false,
    };

    let (_, create_body) = app
        .post_with_auth("/api/v1/appointments", appointment_dto, &patient_token)
        .await;

    let appointment_id = create_body["data"]["id"].as_str().unwrap();

    // Update appointment
    let update_dto = UpdateAppointmentDto {
        appointment_date: Some(Utc::now() + Duration::days(2)),
        time_slot: Some("14:00-15:00".to_string()),
        status: None,
    };

    let (status, body) = app
        .put_with_auth(
            &format!("/api/v1/appointments/{}", appointment_id),
            update_dto,
            &patient_token,
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["time_slot"], "14:00-15:00");
}

#[tokio::test]
async fn test_cancel_appointment() {
    let mut app = TestApp::new().await;

    // Create appointment
    let (patient_user_id, patient_account, patient_password) =
        create_test_user(&app.pool, "patient").await;
    let (doctor_user_id, _, _) = create_test_user(&app.pool, "doctor").await;
    let (doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;

    let patient_token = get_auth_token(&mut app, &patient_account, &patient_password).await;

    let appointment_dto = CreateAppointmentDto {
        patient_id: patient_user_id,
        doctor_id: doctor_id,
        appointment_date: Utc::now() + Duration::days(1),
        time_slot: "09:00-10:00".to_string(),
        visit_type: VisitType::Offline,
        symptoms: "测试症状".to_string(),
        has_visited_before: false,
    };

    let (_, create_body) = app
        .post_with_auth("/api/v1/appointments", appointment_dto, &patient_token)
        .await;

    let appointment_id = create_body["data"]["id"].as_str().unwrap();

    // Cancel appointment
    let (status, body) = app
        .put_with_auth(
            &format!("/api/v1/appointments/{}/cancel", appointment_id),
            json!({}),
            &patient_token,
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["status"], "cancelled");
}

#[tokio::test]
async fn test_get_doctor_appointments() {
    let mut app = TestApp::new().await;

    // Create doctor and patient
    let (doctor_user_id, doctor_account, doctor_password) =
        create_test_user(&app.pool, "doctor").await;
    let (doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;
    let (patient_user_id, _, _) = create_test_user(&app.pool, "patient").await;

    let doctor_token = get_auth_token(&mut app, &doctor_account, &doctor_password).await;

    // Create appointments for the doctor
    for i in 0..3 {
        let appointment_dto = CreateAppointmentDto {
            patient_id: patient_user_id,
            doctor_id: doctor_id,
            appointment_date: Utc::now() + Duration::days(i + 1),
            time_slot: format!("{}:00-{}:00", 9 + i, 10 + i),
            visit_type: VisitType::Offline,
            symptoms: "测试症状".to_string(),
            has_visited_before: false,
        };

        let _ = app
            .post_with_auth("/api/v1/appointments", appointment_dto, &doctor_token)
            .await;
    }

    // Get doctor's appointments
    let (status, body) = app
        .get_with_auth(
            &format!("/api/v1/appointments/doctor/{}?page=1&page_size=10", doctor_id),
            &doctor_token,
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert!(body["data"]["items"].is_array());
    assert_eq!(body["data"]["items"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_get_patient_appointments() {
    let mut app = TestApp::new().await;

    // Create patient and doctor
    let (patient_user_id, patient_account, patient_password) =
        create_test_user(&app.pool, "patient").await;
    let (doctor_user_id, _, _) = create_test_user(&app.pool, "doctor").await;
    let (doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;

    let patient_token = get_auth_token(&mut app, &patient_account, &patient_password).await;

    // Create appointments for the patient
    for i in 0..2 {
        let appointment_dto = CreateAppointmentDto {
            patient_id: patient_user_id,
            doctor_id: doctor_id,
            appointment_date: Utc::now() + Duration::days(i + 1),
            time_slot: format!("{}:00-{}:00", 9 + i, 10 + i),
            visit_type: VisitType::Offline,
            symptoms: "测试症状".to_string(),
            has_visited_before: false,
        };

        let _ = app
            .post_with_auth("/api/v1/appointments", appointment_dto, &patient_token)
            .await;
    }

    // Get patient's appointments
    let (status, body) = app
        .get_with_auth(
            &format!("/api/v1/appointments/patient/{}?page=1&page_size=10", patient_user_id),
            &patient_token,
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert!(body["data"]["items"].is_array());
    assert_eq!(body["data"]["items"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_get_available_slots() {
    let mut app = TestApp::new().await;

    // Create doctor
    let (doctor_user_id, _, _) = create_test_user(&app.pool, "doctor").await;
    let (doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;

    // Create patient and get token
    let (_, patient_account, patient_password) = create_test_user(&app.pool, "patient").await;
    let patient_token = get_auth_token(&mut app, &patient_account, &patient_password).await;

    // Get available slots
    let tomorrow = Utc::now() + Duration::days(1);
    let (status, body) = app
        .get_with_auth(
            &format!(
                "/api/v1/appointments/available-slots?doctor_id={}&date={}",
                doctor_id, tomorrow
            ),
            &patient_token,
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert!(body["data"].is_array());
    // Should have multiple time slots available
    assert!(body["data"].as_array().unwrap().len() > 0);
}

#[tokio::test]
async fn test_appointment_authorization() {
    let mut app = TestApp::new().await;

    // Create two patients and a doctor
    let (patient1_user_id, patient1_account, patient1_password) =
        create_test_user(&app.pool, "patient").await;
    let (_patient2_user_id, patient2_account, patient2_password) =
        create_test_user(&app.pool, "patient").await;
    let (doctor_user_id, _, _) = create_test_user(&app.pool, "doctor").await;
    let (doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;

    let patient1_token = get_auth_token(&mut app, &patient1_account, &patient1_password).await;
    let patient2_token = get_auth_token(&mut app, &patient2_account, &patient2_password).await;

    // Patient 1 creates an appointment
    let appointment_dto = CreateAppointmentDto {
        patient_id: patient1_user_id,
        doctor_id: doctor_id,
        appointment_date: Utc::now() + Duration::days(1),
        time_slot: "09:00-10:00".to_string(),
        visit_type: VisitType::Offline,
        symptoms: "测试症状".to_string(),
        has_visited_before: false,
    };

    let (_, create_body) = app
        .post_with_auth("/api/v1/appointments", appointment_dto, &patient1_token)
        .await;

    let appointment_id = create_body["data"]["id"].as_str().unwrap();

    // Patient 2 tries to update patient 1's appointment (should fail)
    let update_dto = UpdateAppointmentDto {
        appointment_date: None,
        time_slot: Some("14:00-15:00".to_string()),
        status: None,
    };

    let (status, body) = app
        .put_with_auth(
            &format!("/api/v1/appointments/{}", appointment_id),
            update_dto,
            &patient2_token,
        )
        .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["success"], false);
}

#[tokio::test]
async fn test_appointment_conflict() {
    let mut app = TestApp::new().await;

    // Create patient and doctor
    let (patient_user_id, patient_account, patient_password) =
        create_test_user(&app.pool, "patient").await;
    let (doctor_user_id, _, _) = create_test_user(&app.pool, "doctor").await;
    let (doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;

    let patient_token = get_auth_token(&mut app, &patient_account, &patient_password).await;

    let appointment_date = Utc::now() + Duration::days(1);
    let time_slot = "09:00-10:00".to_string();

    // Create first appointment
    let appointment_dto = CreateAppointmentDto {
        patient_id: patient_user_id,
        doctor_id: doctor_id,
        appointment_date: appointment_date,
        time_slot: time_slot.clone(),
        visit_type: VisitType::Offline,
        symptoms: "测试症状1".to_string(),
        has_visited_before: false,
    };

    let (status, _) = app
        .post_with_auth("/api/v1/appointments", appointment_dto, &patient_token)
        .await;

    assert_eq!(status, StatusCode::OK);

    // Try to create conflicting appointment (same doctor, date, and time)
    let conflicting_appointment = CreateAppointmentDto {
        patient_id: patient_user_id,
        doctor_id: doctor_id,
        appointment_date: appointment_date,
        time_slot: time_slot,
        visit_type: VisitType::Offline,
        symptoms: "测试症状2".to_string(),
        has_visited_before: false,
    };

    let (status, body) = app
        .post_with_auth("/api/v1/appointments", conflicting_appointment, &patient_token)
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["success"], false);
    assert!(body["error"].as_str().unwrap().contains("时间段已被预约"));
}