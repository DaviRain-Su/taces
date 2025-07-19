use crate::common::{create_test_app, create_test_appointment, create_test_doctor, create_test_user, TestApp};
use backend::models::video_consultation::*;
use backend::models::appointment::AppointmentStatus;
use backend::models::UserRole;
use chrono::{Duration, Utc};
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_create_video_consultation() {
    let app = TestApp::new().await;
    
    // Create patient and doctor
    let patient = create_test_user(&app, "patient@test.com", UserRole::Patient).await;
    let doctor = create_test_doctor(&app, "doctor@test.com").await;
    
    // Create appointment
    let appointment = create_test_appointment(&app, &patient.id, &doctor.id).await;
    
    // Update appointment to confirmed status
    let update_query = r#"
        UPDATE appointments SET status = 'confirmed' WHERE id = ?
    "#;
    sqlx::query(update_query)
        .bind(&appointment.id)
        .execute(&app.db)
        .await
        .unwrap();
    
    // Login as doctor
    let doctor_token = app.login_user(&doctor.email, "password123").await;
    
    // Create video consultation
    let create_dto = serde_json::json!({
        "appointment_id": appointment.id,
        "doctor_id": doctor.id,
        "patient_id": patient.id,
        "scheduled_start_time": (Utc::now() + Duration::hours(1)).to_rfc3339(),
        "chief_complaint": "头痛、失眠"
    });
    
    let response = app.client
        .post(&format!("{}/api/v1/video-consultations", app.address))
        .header("Authorization", format!("Bearer {}", doctor_token))
        .json(&create_dto)
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 201);
    
    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["data"]["status"].as_str().unwrap(), "waiting");
    assert!(body["data"]["room_id"].as_str().is_some());
}

#[tokio::test]
#[serial]
async fn test_get_video_consultation() {
    let app = TestApp::new().await;
    
    // Create patient and doctor
    let patient = create_test_user(&app, "patient@test.com", UserRole::Patient).await;
    let doctor = create_test_doctor(&app, "doctor@test.com").await;
    
    // Create consultation directly in database
    let consultation_id = uuid::Uuid::new_v4();
    let appointment_id = uuid::Uuid::new_v4();
    let room_id = format!("room_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
    
    let query = r#"
        INSERT INTO video_consultations (
            id, appointment_id, doctor_id, patient_id, room_id,
            status, scheduled_start_time, chief_complaint,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, 'waiting', ?, ?, ?, ?)
    "#;
    
    let now = Utc::now();
    sqlx::query(query)
        .bind(&consultation_id)
        .bind(&appointment_id)
        .bind(&doctor.id)
        .bind(&patient.id)
        .bind(&room_id)
        .bind(&now + Duration::hours(1))
        .bind("头痛")
        .bind(&now)
        .bind(&now)
        .execute(&app.db)
        .await
        .unwrap();
    
    // Login as doctor and get consultation
    let doctor_token = app.login_user(&doctor.email, "password123").await;
    
    let response = app.client
        .get(&format!("{}/api/v1/video-consultations/{}", app.address, consultation_id))
        .header("Authorization", format!("Bearer {}", doctor_token))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["data"]["id"].as_str().unwrap(), consultation_id.to_string());
    assert_eq!(body["data"]["room_id"].as_str().unwrap(), room_id);
}

#[tokio::test]
#[serial]
async fn test_join_room() {
    let app = TestApp::new().await;
    
    // Create patient and doctor
    let patient = create_test_user(&app, "patient@test.com", UserRole::Patient).await;
    let doctor = create_test_doctor(&app, "doctor@test.com").await;
    
    // Create consultation
    let consultation_id = uuid::Uuid::new_v4();
    let appointment_id = uuid::Uuid::new_v4();
    let room_id = format!("room_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
    
    let query = r#"
        INSERT INTO video_consultations (
            id, appointment_id, doctor_id, patient_id, room_id,
            status, scheduled_start_time, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, 'waiting', ?, ?, ?)
    "#;
    
    let now = Utc::now();
    sqlx::query(query)
        .bind(&consultation_id)
        .bind(&appointment_id)
        .bind(&doctor.id)
        .bind(&patient.id)
        .bind(&room_id)
        .bind(&now + Duration::hours(1))
        .bind(&now)
        .bind(&now)
        .execute(&app.db)
        .await
        .unwrap();
    
    // Doctor joins room
    let doctor_token = app.login_user(&doctor.email, "password123").await;
    
    let response = app.client
        .post(&format!("{}/api/v1/video-consultations/room/{}/join", app.address, room_id))
        .header("Authorization", format!("Bearer {}", doctor_token))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["data"]["room_id"].as_str().unwrap(), room_id);
    assert_eq!(body["data"]["role"].as_str().unwrap(), "doctor");
    assert!(body["data"]["token"].as_str().is_some());
    assert!(body["data"]["ice_servers"].is_array());
}

#[tokio::test]
#[serial]
async fn test_start_consultation() {
    let app = TestApp::new().await;
    
    // Create patient and doctor
    let patient = create_test_user(&app, "patient@test.com", UserRole::Patient).await;
    let doctor = create_test_doctor(&app, "doctor@test.com").await;
    
    // Create consultation
    let consultation_id = uuid::Uuid::new_v4();
    let appointment_id = uuid::Uuid::new_v4();
    let room_id = format!("room_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
    
    let query = r#"
        INSERT INTO video_consultations (
            id, appointment_id, doctor_id, patient_id, room_id,
            status, scheduled_start_time, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, 'waiting', ?, ?, ?)
    "#;
    
    let now = Utc::now();
    sqlx::query(query)
        .bind(&consultation_id)
        .bind(&appointment_id)
        .bind(&doctor.id)
        .bind(&patient.id)
        .bind(&room_id)
        .bind(&now)
        .bind(&now)
        .bind(&now)
        .execute(&app.db)
        .await
        .unwrap();
    
    // Doctor starts consultation
    let doctor_token = app.login_user(&doctor.email, "password123").await;
    
    let response = app.client
        .put(&format!("{}/api/v1/video-consultations/{}/start", app.address, consultation_id))
        .header("Authorization", format!("Bearer {}", doctor_token))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    // Verify consultation status updated
    let check_query = r#"
        SELECT status, actual_start_time FROM video_consultations WHERE id = ?
    "#;
    
    let row = sqlx::query(check_query)
        .bind(&consultation_id)
        .fetch_one(&app.db)
        .await
        .unwrap();
    
    let status: String = row.try_get("status").unwrap();
    assert_eq!(status, "in_progress");
    
    let start_time: Option<chrono::DateTime<chrono::Utc>> = row.try_get("actual_start_time").unwrap();
    assert!(start_time.is_some());
}

#[tokio::test]
#[serial]
async fn test_end_consultation() {
    let app = TestApp::new().await;
    
    // Create patient and doctor
    let patient = create_test_user(&app, "patient@test.com", UserRole::Patient).await;
    let doctor = create_test_doctor(&app, "doctor@test.com").await;
    
    // Create consultation in progress
    let consultation_id = uuid::Uuid::new_v4();
    let appointment_id = uuid::Uuid::new_v4();
    let room_id = format!("room_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
    
    let query = r#"
        INSERT INTO video_consultations (
            id, appointment_id, doctor_id, patient_id, room_id,
            status, scheduled_start_time, actual_start_time, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, 'in_progress', ?, ?, ?, ?)
    "#;
    
    let now = Utc::now();
    sqlx::query(query)
        .bind(&consultation_id)
        .bind(&appointment_id)
        .bind(&doctor.id)
        .bind(&patient.id)
        .bind(&room_id)
        .bind(&now - Duration::minutes(30))
        .bind(&now - Duration::minutes(25))
        .bind(&now)
        .bind(&now)
        .execute(&app.db)
        .await
        .unwrap();
    
    // Doctor ends consultation
    let doctor_token = app.login_user(&doctor.email, "password123").await;
    
    let end_dto = serde_json::json!({
        "diagnosis": "神经性头痛，失眠症",
        "treatment_plan": "中药调理，针灸治疗",
        "notes": "患者症状明显，建议坚持治疗"
    });
    
    let response = app.client
        .put(&format!("{}/api/v1/video-consultations/{}/end", app.address, consultation_id))
        .header("Authorization", format!("Bearer {}", doctor_token))
        .json(&end_dto)
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    // Verify consultation completed
    let check_query = r#"
        SELECT status, diagnosis, treatment_plan, notes, duration FROM video_consultations WHERE id = ?
    "#;
    
    let row = sqlx::query(check_query)
        .bind(&consultation_id)
        .fetch_one(&app.db)
        .await
        .unwrap();
    
    let status: String = row.try_get("status").unwrap();
    assert_eq!(status, "completed");
    
    let diagnosis: String = row.try_get("diagnosis").unwrap();
    assert_eq!(diagnosis, "神经性头痛，失眠症");
    
    let duration: Option<i32> = row.try_get("duration").unwrap();
    assert!(duration.is_some());
}

#[tokio::test]
#[serial]
async fn test_rate_consultation() {
    let app = TestApp::new().await;
    
    // Create patient and doctor
    let patient = create_test_user(&app, "patient@test.com", UserRole::Patient).await;
    let doctor = create_test_doctor(&app, "doctor@test.com").await;
    
    // Create completed consultation
    let consultation_id = uuid::Uuid::new_v4();
    let appointment_id = uuid::Uuid::new_v4();
    let room_id = format!("room_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
    
    let query = r#"
        INSERT INTO video_consultations (
            id, appointment_id, doctor_id, patient_id, room_id,
            status, scheduled_start_time, actual_start_time, end_time,
            duration, diagnosis, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, 'completed', ?, ?, ?, ?, ?, ?, ?)
    "#;
    
    let now = Utc::now();
    sqlx::query(query)
        .bind(&consultation_id)
        .bind(&appointment_id)
        .bind(&doctor.id)
        .bind(&patient.id)
        .bind(&room_id)
        .bind(&now - Duration::hours(2))
        .bind(&now - Duration::hours(2))
        .bind(&now - Duration::minutes(90))
        .bind(1800)
        .bind("头痛症")
        .bind(&now)
        .bind(&now)
        .execute(&app.db)
        .await
        .unwrap();
    
    // Patient rates consultation
    let patient_token = app.login_user(&patient.email, "password123").await;
    
    let rate_dto = serde_json::json!({
        "rating": 5,
        "feedback": "医生很专业，解答详细"
    });
    
    let response = app.client
        .post(&format!("{}/api/v1/video-consultations/{}/rate", app.address, consultation_id))
        .header("Authorization", format!("Bearer {}", patient_token))
        .json(&rate_dto)
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    // Verify rating saved
    let check_query = r#"
        SELECT patient_rating, patient_feedback FROM video_consultations WHERE id = ?
    "#;
    
    let row = sqlx::query(check_query)
        .bind(&consultation_id)
        .fetch_one(&app.db)
        .await
        .unwrap();
    
    let rating: i32 = row.try_get("patient_rating").unwrap();
    assert_eq!(rating, 5);
    
    let feedback: String = row.try_get("patient_feedback").unwrap();
    assert_eq!(feedback, "医生很专业，解答详细");
}

#[tokio::test]
#[serial]
async fn test_webrtc_signaling() {
    let app = TestApp::new().await;
    
    // Create patient and doctor
    let patient = create_test_user(&app, "patient@test.com", UserRole::Patient).await;
    let doctor = create_test_doctor(&app, "doctor@test.com").await;
    
    // Create consultation
    let consultation_id = uuid::Uuid::new_v4();
    let appointment_id = uuid::Uuid::new_v4();
    let room_id = format!("room_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
    
    let query = r#"
        INSERT INTO video_consultations (
            id, appointment_id, doctor_id, patient_id, room_id,
            status, scheduled_start_time, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, 'in_progress', ?, ?, ?)
    "#;
    
    let now = Utc::now();
    sqlx::query(query)
        .bind(&consultation_id)
        .bind(&appointment_id)
        .bind(&doctor.id)
        .bind(&patient.id)
        .bind(&room_id)
        .bind(&now)
        .bind(&now)
        .bind(&now)
        .execute(&app.db)
        .await
        .unwrap();
    
    // Doctor sends signal
    let doctor_token = app.login_user(&doctor.email, "password123").await;
    
    let signal_dto = serde_json::json!({
        "room_id": room_id,
        "to_user_id": patient.id,
        "signal_type": "offer",
        "payload": {
            "sdp": "v=0\r\no=- 123456789 2 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\n..."
        }
    });
    
    let response = app.client
        .post(&format!("{}/api/v1/video-consultations/signal", app.address))
        .header("Authorization", format!("Bearer {}", doctor_token))
        .json(&signal_dto)
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    // Patient receives signals
    let patient_token = app.login_user(&patient.email, "password123").await;
    
    let response = app.client
        .get(&format!("{}/api/v1/video-consultations/signal/{}", app.address, room_id))
        .header("Authorization", format!("Bearer {}", patient_token))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["success"].as_bool().unwrap());
    
    let signals = body["data"].as_array().unwrap();
    assert_eq!(signals.len(), 1);
    assert_eq!(signals[0]["signal_type"].as_str().unwrap(), "offer");
    assert_eq!(signals[0]["from_user_id"].as_str().unwrap(), doctor.id.to_string());
}

#[tokio::test]
#[serial]
async fn test_consultation_templates() {
    let app = TestApp::new().await;
    
    // Create doctor
    let doctor = create_test_doctor(&app, "doctor@test.com").await;
    let doctor_token = app.login_user(&doctor.email, "password123").await;
    
    // Create template
    let template_dto = serde_json::json!({
        "name": "失眠症常规诊断",
        "chief_complaint": "失眠、多梦、早醒",
        "diagnosis": "失眠症",
        "treatment_plan": "安神补脑液，酸枣仁汤加减",
        "notes": "注意作息规律，避免熬夜"
    });
    
    let response = app.client
        .post(&format!("{}/api/v1/video-consultations/templates", app.address))
        .header("Authorization", format!("Bearer {}", doctor_token))
        .json(&template_dto)
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 201);
    
    let body: serde_json::Value = response.json().await.unwrap();
    let template_id = body["data"]["id"].as_str().unwrap();
    
    // List templates
    let response = app.client
        .get(&format!("{}/api/v1/video-consultations/templates", app.address))
        .header("Authorization", format!("Bearer {}", doctor_token))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.unwrap();
    let templates = body["data"].as_array().unwrap();
    assert!(!templates.is_empty());
    
    // Use template
    let response = app.client
        .post(&format!("{}/api/v1/video-consultations/templates/{}/use", app.address, template_id))
        .header("Authorization", format!("Bearer {}", doctor_token))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    // Verify usage count increased
    let check_query = r#"
        SELECT usage_count FROM video_consultation_templates WHERE id = ?
    "#;
    
    let row = sqlx::query(check_query)
        .bind(uuid::Uuid::parse_str(template_id).unwrap())
        .fetch_one(&app.db)
        .await
        .unwrap();
    
    let usage_count: i32 = row.try_get("usage_count").unwrap();
    assert_eq!(usage_count, 1);
}

#[tokio::test]
#[serial]
async fn test_consultation_statistics() {
    let app = TestApp::new().await;
    
    // Create doctor
    let doctor = create_test_doctor(&app, "doctor@test.com").await;
    let doctor_token = app.login_user(&doctor.email, "password123").await;
    
    // Create some consultations with different statuses
    let now = Utc::now();
    
    // Completed consultation
    let query = r#"
        INSERT INTO video_consultations (
            id, appointment_id, doctor_id, patient_id, room_id,
            status, scheduled_start_time, duration, patient_rating,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, 'completed', ?, ?, ?, ?, ?)
    "#;
    
    sqlx::query(query)
        .bind(&uuid::Uuid::new_v4())
        .bind(&uuid::Uuid::new_v4())
        .bind(&doctor.id)
        .bind(&uuid::Uuid::new_v4())
        .bind(&format!("room_{}", uuid::Uuid::new_v4().to_string().replace("-", "")))
        .bind(&now - Duration::days(1))
        .bind(1800)
        .bind(5)
        .bind(&now)
        .bind(&now)
        .execute(&app.db)
        .await
        .unwrap();
    
    // No-show consultation
    sqlx::query(query.replace("'completed'", "'no_show'").replace(", ?", ", NULL"))
        .bind(&uuid::Uuid::new_v4())
        .bind(&uuid::Uuid::new_v4())
        .bind(&doctor.id)
        .bind(&uuid::Uuid::new_v4())
        .bind(&format!("room_{}", uuid::Uuid::new_v4().to_string().replace("-", "")))
        .bind(&now - Duration::days(2))
        .bind(&now)
        .bind(&now)
        .execute(&app.db)
        .await
        .unwrap();
    
    // Get statistics
    let response = app.client
        .get(&format!("{}/api/v1/video-consultations/statistics", app.address))
        .header("Authorization", format!("Bearer {}", doctor_token))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.unwrap();
    let stats = &body["data"];
    
    assert!(stats["total_consultations"].as_i64().unwrap() >= 2);
    assert!(stats["completed_consultations"].as_i64().unwrap() >= 1);
    assert!(stats["average_duration"].as_f64().is_some());
    assert!(stats["average_rating"].as_f64().is_some());
    assert!(stats["no_show_rate"].as_f64().is_some());
}

#[tokio::test]
#[serial]
async fn test_authorization() {
    let app = TestApp::new().await;
    
    // Create two doctors
    let doctor1 = create_test_doctor(&app, "doctor1@test.com").await;
    let doctor2 = create_test_doctor(&app, "doctor2@test.com").await;
    
    // Create consultation for doctor1
    let consultation_id = uuid::Uuid::new_v4();
    let appointment_id = uuid::Uuid::new_v4();
    let room_id = format!("room_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));
    
    let query = r#"
        INSERT INTO video_consultations (
            id, appointment_id, doctor_id, patient_id, room_id,
            status, scheduled_start_time, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, 'waiting', ?, ?, ?)
    "#;
    
    let now = Utc::now();
    sqlx::query(query)
        .bind(&consultation_id)
        .bind(&appointment_id)
        .bind(&doctor1.id)
        .bind(&uuid::Uuid::new_v4())
        .bind(&room_id)
        .bind(&now)
        .bind(&now)
        .bind(&now)
        .execute(&app.db)
        .await
        .unwrap();
    
    // Doctor2 tries to access doctor1's consultation
    let doctor2_token = app.login_user(&doctor2.email, "password123").await;
    
    let response = app.client
        .get(&format!("{}/api/v1/video-consultations/{}", app.address, consultation_id))
        .header("Authorization", format!("Bearer {}", doctor2_token))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 403);
}