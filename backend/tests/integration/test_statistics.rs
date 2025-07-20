use crate::common::TestApp;
use axum::http::StatusCode;
use backend::{
    models::user::LoginDto,
    utils::test_helpers::{create_test_doctor, create_test_user},
};
use chrono::{Duration, Local};
use uuid::Uuid;

async fn get_auth_token(app: &mut TestApp, account: &str, password: &str) -> String {
    let login_dto = LoginDto {
        account: account.to_string(),
        password: password.to_string(),
    };

    let (_, body) = app.post("/api/v1/auth/login", login_dto).await;
    body["data"]["token"].as_str().unwrap().to_string()
}

async fn setup_test_data(app: &mut TestApp) {
    // Create some test data for statistics
    let (doctor_user_id, _, _) = create_test_user(&app.pool, "doctor").await;
    let (doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;

    let (patient_user_id, _, _) = create_test_user(&app.pool, "patient").await;

    // Create appointments
    for i in 0..5 {
        sqlx::query(
            r#"
            INSERT INTO appointments (id, patient_id, doctor_id, appointment_date, time_slot, 
                                     visit_type, symptoms, has_visited_before, status, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, 'offline', '测试症状', false, ?, NOW(), NOW())
            "#
        )
        .bind(Uuid::new_v4().to_string())
        .bind(patient_user_id.to_string())
        .bind(doctor_id.to_string())
        .bind((Local::now() - Duration::days(i)).naive_utc())
        .bind("09:00-10:00")
        .bind(if i < 3 { "completed" } else { "pending" })
        .execute(&app.pool)
        .await
        .unwrap();
    }

    // Create prescriptions
    for i in 0..3 {
        sqlx::query(
            r#"
            INSERT INTO prescriptions (id, code, doctor_id, patient_id, patient_name, 
                                      diagnosis, medicines, instructions, prescription_date, created_at)
            VALUES (?, ?, ?, ?, '测试患者', '测试诊断', '[]', '测试说明', NOW(), NOW())
            "#
        )
        .bind(Uuid::new_v4().to_string())
        .bind(format!("RX{:08}", i + 1))
        .bind(doctor_id.to_string())
        .bind(patient_user_id.to_string())
        .execute(&app.pool)
        .await
        .unwrap();
    }
}

#[tokio::test]
async fn test_dashboard_stats() {
    let mut app = TestApp::new().await;
    setup_test_data(&mut app).await;

    // Create admin
    let (_, admin_account, admin_password) = create_test_user(&app.pool, "admin").await;
    let admin_token = get_auth_token(&mut app, &admin_account, &admin_password).await;

    // Get dashboard stats
    let (status, body) = app
        .get_with_auth("/api/v1/statistics/dashboard", &admin_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert!(body["data"]["total_users"].as_i64().unwrap() > 0);
    assert!(body["data"]["total_appointments"].as_i64().unwrap() >= 5);
    assert!(body["data"]["total_prescriptions"].as_i64().unwrap() >= 3);

    // Non-admin cannot access
    let (_, patient_account, patient_password) = create_test_user(&app.pool, "patient").await;
    let patient_token = get_auth_token(&mut app, &patient_account, &patient_password).await;

    let (status, _) = app
        .get_with_auth("/api/v1/statistics/dashboard", &patient_token)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_doctor_statistics() {
    let mut app = TestApp::new().await;

    // Create doctor
    let (doctor_user_id, doctor_account, doctor_password) =
        create_test_user(&app.pool, "doctor").await;
    let (doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;
    let doctor_token = get_auth_token(&mut app, &doctor_account, &doctor_password).await;

    // Create some appointments
    let (patient_user_id, _, _) = create_test_user(&app.pool, "patient").await;
    for i in 0..3 {
        sqlx::query(
            r#"
            INSERT INTO appointments (id, patient_id, doctor_id, appointment_date, time_slot, 
                                     visit_type, symptoms, has_visited_before, status, created_at, updated_at)
            VALUES (?, ?, ?, DATE_ADD(NOW(), INTERVAL ? DAY), '09:00-10:00', 'offline', '测试', false, 'pending', NOW(), NOW())
            "#
        )
        .bind(Uuid::new_v4().to_string())
        .bind(patient_user_id.to_string())
        .bind(doctor_id.to_string())
        .bind(i)
        .execute(&app.pool)
        .await
        .unwrap();
    }

    // Doctor can access their own stats
    let (status, body) = app
        .get_with_auth(
            &format!("/api/v1/statistics/doctor/{}", doctor_id),
            &doctor_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["total_appointments"], 3);

    // Doctor cannot access other doctor's stats
    let (other_doctor_user_id, _, _) = create_test_user(&app.pool, "doctor").await;
    let (other_doctor_id, _) = create_test_doctor(&app.pool, other_doctor_user_id).await;

    let (status, _) = app
        .get_with_auth(
            &format!("/api/v1/statistics/doctor/{}", other_doctor_id),
            &doctor_token,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_patient_statistics() {
    let mut app = TestApp::new().await;

    // Create patient
    let (patient_user_id, patient_account, patient_password) =
        create_test_user(&app.pool, "patient").await;
    let patient_token = get_auth_token(&mut app, &patient_account, &patient_password).await;

    // Create doctor and appointments
    let (doctor_user_id, _, _) = create_test_user(&app.pool, "doctor").await;
    let (doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;

    for i in 0..4 {
        sqlx::query!(
            r#"
            INSERT INTO appointments (patient_id, doctor_id, appointment_date, time_slot, 
                                     visit_type, symptoms, has_visited_before, status)
            VALUES (?, ?, NOW() + INTERVAL ? DAY, '09:00-10:00', 'offline', '测试', false, ?)
            "#,
            patient_user_id.to_string(),
            doctor_id.to_string(),
            i,
            if i < 2 { "completed" } else { "pending" }
        )
        .execute(&app.pool)
        .await
        .unwrap();
    }

    // Patient can access their stats
    let (status, body) = app
        .get_with_auth("/api/v1/statistics/patient", &patient_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["total_appointments"], 4);
    assert_eq!(body["data"]["completed_appointments"], 2);
    assert_eq!(body["data"]["upcoming_appointments"], 2);

    // Non-patient cannot access
    let (_, doctor_account, doctor_password) = create_test_user(&app.pool, "doctor").await;
    let doctor_token = get_auth_token(&mut app, &doctor_account, &doctor_password).await;

    let (status, _) = app
        .get_with_auth("/api/v1/statistics/patient", &doctor_token)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_appointment_trends() {
    let mut app = TestApp::new().await;
    setup_test_data(&mut app).await;

    // Create admin
    let (_, admin_account, admin_password) = create_test_user(&app.pool, "admin").await;
    let admin_token = get_auth_token(&mut app, &admin_account, &admin_password).await;

    // Get appointment trends
    let end_date = Local::now().naive_local().date();
    let start_date = end_date - Duration::days(7);

    let (status, body) = app
        .get_with_auth(
            &format!(
                "/api/v1/statistics/appointment-trends?start_date={}&end_date={}",
                start_date, end_date
            ),
            &admin_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["data"].is_array());
}

#[tokio::test]
async fn test_public_statistics() {
    let mut app = TestApp::new().await;
    setup_test_data(&mut app).await;

    // Department statistics (public)
    let (status, body) = app.get("/api/v1/statistics/departments").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["data"].is_array());

    // Top doctors (public)
    let (status, body) = app.get("/api/v1/statistics/top-doctors").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["data"].is_array());

    // Top content (public)
    let (status, body) = app.get("/api/v1/statistics/top-content").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["data"].is_array());
}

#[tokio::test]
async fn test_admin_only_statistics() {
    let mut app = TestApp::new().await;

    // Create admin and patient
    let (_, admin_account, admin_password) = create_test_user(&app.pool, "admin").await;
    let admin_token = get_auth_token(&mut app, &admin_account, &admin_password).await;

    let (_, patient_account, patient_password) = create_test_user(&app.pool, "patient").await;
    let patient_token = get_auth_token(&mut app, &patient_account, &patient_password).await;

    // Test various admin-only endpoints
    let admin_endpoints = vec![
        "/api/v1/statistics/time-slots",
        "/api/v1/statistics/content",
        "/api/v1/statistics/live-streams",
        "/api/v1/statistics/circles",
        "/api/v1/statistics/user-growth",
        "/api/v1/statistics/appointment-heatmap",
    ];

    for endpoint in admin_endpoints {
        // Admin can access
        let (status, _) = app.get_with_auth(endpoint, &admin_token).await;
        assert_eq!(status, StatusCode::OK);

        // Non-admin cannot access
        let (status, _) = app.get_with_auth(endpoint, &patient_token).await;
        assert_eq!(status, StatusCode::FORBIDDEN);
    }
}

#[tokio::test]
async fn test_export_data() {
    let mut app = TestApp::new().await;
    setup_test_data(&mut app).await;

    // Create admin
    let (_, admin_account, admin_password) = create_test_user(&app.pool, "admin").await;
    let admin_token = get_auth_token(&mut app, &admin_account, &admin_password).await;

    // Export appointments
    let (status, body) = app
        .get_with_auth(
            "/api/v1/statistics/export?export_type=Appointments&format=CSV",
            &admin_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["data"]["data"].is_string());
    assert_eq!(body["data"]["format"], "csv");

    // Non-admin cannot export
    let (_, patient_account, patient_password) = create_test_user(&app.pool, "patient").await;
    let patient_token = get_auth_token(&mut app, &patient_account, &patient_password).await;

    let (status, _) = app
        .get_with_auth(
            "/api/v1/statistics/export?export_type=Appointments&format=CSV",
            &patient_token,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_user_growth_statistics() {
    let mut app = TestApp::new().await;

    // Create admin
    let (_, admin_account, admin_password) = create_test_user(&app.pool, "admin").await;
    let admin_token = get_auth_token(&mut app, &admin_account, &admin_password).await;

    // Create users over several days
    for i in 0..5 {
        let created_at = Local::now() - Duration::days(i);
        let role = if i % 2 == 0 { "patient" } else { "doctor" };

        sqlx::query(
            r#"
            INSERT INTO users (id, account, name, password, gender, phone, email, role, status, created_at)
            VALUES (?, ?, '测试用户', 'hash', '男', ?, ?, ?, 'active', ?)
            "#
        )
        .bind(Uuid::new_v4().to_string())
        .bind(format!("growth_test_{}", i))
        .bind(format!("1380000000{}", i))
        .bind(format!("growth_test_{}@test.com", i))
        .bind(role)
        .bind(created_at.naive_utc())
        .execute(&app.pool)
        .await
        .unwrap();
    }

    // Get user growth stats
    let end_date = Local::now().naive_local().date();
    let start_date = end_date - Duration::days(7);

    let (status, body) = app
        .get_with_auth(
            &format!(
                "/api/v1/statistics/user-growth?start_date={}&end_date={}",
                start_date, end_date
            ),
            &admin_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["data"].is_array());
    assert!(!body["data"].as_array().unwrap().is_empty());
}
