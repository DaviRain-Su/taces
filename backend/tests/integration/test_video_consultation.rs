use crate::common::TestApp;
use axum::http::StatusCode;
use backend::utils::test_helpers::{create_test_doctor, create_test_user};
use chrono::{Duration, Utc};
//use serial_test::serial;
use serde_json::json;
use uuid::Uuid;

async fn get_auth_token(app: &mut TestApp, account: &str, password: &str) -> String {
    let login_data = json!({
        "account": account,
        "password": password
    });

    let (_, body) = app.post("/api/v1/auth/login", login_data).await;
    body["data"]["token"].as_str().unwrap().to_string()
}

#[tokio::test]
//#[serial]
async fn test_create_video_consultation() {
    let mut app = TestApp::new().await;

    // Create patient and doctor
    let (patient_id, _patient_email, _patient_password) =
        create_test_user(&app.pool, "patient").await;
    let (doctor_user_id, doctor_email, doctor_password) =
        create_test_user(&app.pool, "doctor").await;
    let (doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;

    // Create appointment
    let appointment_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO appointments (id, patient_id, doctor_id, appointment_date, time_slot, 
                                 visit_type, symptoms, has_visited_before, status, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, 'online_video', '测试症状', false, 'pending', NOW(), NOW())
        "#
    )
    .bind(appointment_id.to_string())
    .bind(patient_id.to_string())
    .bind(doctor_id.to_string())
    .bind((Utc::now() + Duration::hours(2)).naive_utc())
    .bind("09:00-10:00")
    .execute(&app.pool)
    .await
    .unwrap();

    // Update appointment to confirmed status
    let update_query = r#"
        UPDATE appointments SET status = 'confirmed' WHERE id = ?
    "#;
    sqlx::query(update_query)
        .bind(appointment_id.to_string())
        .execute(&app.pool)
        .await
        .unwrap();

    // Login as doctor
    let doctor_token = get_auth_token(&mut app, &doctor_email, &doctor_password).await;

    // Create video consultation
    let create_dto = json!({
        "appointment_id": appointment_id,
        "doctor_id": doctor_id,
        "patient_id": patient_id,
        "scheduled_start_time": (Utc::now() + Duration::hours(1)).to_rfc3339(),
        "chief_complaint": "头痛、失眠"
    });

    let (status, body) = app
        .post_with_auth("/api/v1/video-consultations", create_dto, &doctor_token)
        .await;

    if status != StatusCode::CREATED {
        println!(
            "Create video consultation failed: status={:?}, body={:?}",
            status, body
        );
    }
    assert_eq!(status, StatusCode::CREATED);
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["data"]["status"].as_str().unwrap(), "waiting");
    assert!(body["data"]["room_id"].as_str().is_some());
}

#[tokio::test]
//#[serial]
async fn test_get_video_consultation() {
    let mut app = TestApp::new().await;

    // Create patient and doctor
    let (patient_id, _patient_email, _patient_password) =
        create_test_user(&app.pool, "patient").await;
    let (doctor_user_id, doctor_email, doctor_password) =
        create_test_user(&app.pool, "doctor").await;
    let (doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;

    // Create an appointment first
    let appointment_id = uuid::Uuid::new_v4();
    let now = Utc::now();
    let appointment_date = now + Duration::days(1);

    let appointment_query = r#"
        INSERT INTO appointments (
            id, patient_id, doctor_id, appointment_date, time_slot,
            visit_type, symptoms, has_visited_before, status,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, 'offline', ?, false, 'pending', ?, ?)
    "#;

    sqlx::query(appointment_query)
        .bind(appointment_id.to_string())
        .bind(patient_id.to_string())
        .bind(doctor_id.to_string())
        .bind(appointment_date)
        .bind("09:00-10:00")
        .bind("头痛")
        .bind(now)
        .bind(now)
        .execute(&app.pool)
        .await
        .unwrap();

    // Create consultation with the appointment
    let consultation_id = uuid::Uuid::new_v4();
    let room_id = format!("room_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));

    let query = r#"
        INSERT INTO video_consultations (
            id, appointment_id, doctor_id, patient_id, room_id,
            status, scheduled_start_time, chief_complaint,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, 'waiting', ?, ?, ?, ?)
    "#;

    sqlx::query(query)
        .bind(consultation_id.to_string())
        .bind(appointment_id.to_string())
        .bind(doctor_id.to_string())
        .bind(patient_id.to_string())
        .bind(&room_id)
        .bind(appointment_date)
        .bind("头痛")
        .bind(now)
        .bind(now)
        .execute(&app.pool)
        .await
        .unwrap();

    // Login as doctor and get consultation
    let doctor_token = get_auth_token(&mut app, &doctor_email, &doctor_password).await;

    let (status, body) = app
        .get_with_auth(
            &format!("/api/v1/video-consultations/{}", consultation_id),
            &doctor_token,
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(
        body["data"]["id"].as_str().unwrap(),
        consultation_id.to_string()
    );
    assert_eq!(body["data"]["room_id"].as_str().unwrap(), room_id);
}

#[tokio::test]
//#[serial]
async fn test_join_room() {
    let mut app = TestApp::new().await;

    // Create patient and doctor
    let (patient_id, _patient_email, _patient_password) =
        create_test_user(&app.pool, "patient").await;
    let (doctor_user_id, doctor_email, doctor_password) =
        create_test_user(&app.pool, "doctor").await;
    let (doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;

    // Create an appointment first
    let appointment_id = uuid::Uuid::new_v4();
    let now = Utc::now();
    let scheduled_time = now + Duration::hours(1);

    let appointment_query = r#"
        INSERT INTO appointments (
            id, patient_id, doctor_id, appointment_date, time_slot,
            visit_type, symptoms, has_visited_before, status,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, 'online_video', ?, false, 'confirmed', ?, ?)
    "#;

    sqlx::query(appointment_query)
        .bind(appointment_id.to_string())
        .bind(patient_id.to_string())
        .bind(doctor_id.to_string())
        .bind(scheduled_time.naive_utc())
        .bind("09:00-10:00")
        .bind("test symptoms")
        .bind(now)
        .bind(now)
        .execute(&app.pool)
        .await
        .unwrap();

    // Create consultation
    let consultation_id = uuid::Uuid::new_v4();
    let room_id = format!("room_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));

    let query = r#"
        INSERT INTO video_consultations (
            id, appointment_id, doctor_id, patient_id, room_id,
            status, scheduled_start_time, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, 'waiting', ?, ?, ?)
    "#;

    sqlx::query(query)
        .bind(consultation_id.to_string())
        .bind(appointment_id.to_string())
        .bind(doctor_id.to_string())
        .bind(patient_id.to_string())
        .bind(&room_id)
        .bind(scheduled_time)
        .bind(now)
        .bind(now)
        .execute(&app.pool)
        .await
        .unwrap();

    // Doctor joins room
    let doctor_token = get_auth_token(&mut app, &doctor_email, &doctor_password).await;

    let (status, body) = app
        .post_with_auth(
            &format!("/api/v1/video-consultations/room/{}/join", room_id),
            json!({}),
            &doctor_token,
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["data"]["room_id"].as_str().unwrap(), room_id);
    assert_eq!(body["data"]["role"].as_str().unwrap(), "doctor");
    assert!(body["data"]["token"].as_str().is_some());
    assert!(body["data"]["ice_servers"].is_array());
}

#[tokio::test]
//#[serial]
async fn test_start_consultation() {
    let mut app = TestApp::new().await;

    // Create patient and doctor
    let (patient_id, _patient_email, _patient_password) =
        create_test_user(&app.pool, "patient").await;
    let (doctor_user_id, doctor_email, doctor_password) =
        create_test_user(&app.pool, "doctor").await;
    let (doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;

    // Create an appointment first
    let appointment_id = uuid::Uuid::new_v4();
    let now = Utc::now();

    let appointment_query = r#"
        INSERT INTO appointments (
            id, patient_id, doctor_id, appointment_date, time_slot,
            visit_type, symptoms, has_visited_before, status,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, 'online_video', ?, false, 'confirmed', ?, ?)
    "#;

    sqlx::query(appointment_query)
        .bind(appointment_id.to_string())
        .bind(patient_id.to_string())
        .bind(doctor_id.to_string())
        .bind(now.naive_utc())
        .bind("09:00-10:00")
        .bind("test symptoms")
        .bind(now)
        .bind(now)
        .execute(&app.pool)
        .await
        .unwrap();

    // Create consultation
    let consultation_id = uuid::Uuid::new_v4();
    let room_id = format!("room_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));

    let query = r#"
        INSERT INTO video_consultations (
            id, appointment_id, doctor_id, patient_id, room_id,
            status, scheduled_start_time, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, 'waiting', ?, ?, ?)
    "#;

    sqlx::query(query)
        .bind(consultation_id.to_string())
        .bind(appointment_id.to_string())
        .bind(doctor_id.to_string())
        .bind(patient_id.to_string())
        .bind(&room_id)
        .bind(now)
        .bind(now)
        .bind(now)
        .execute(&app.pool)
        .await
        .unwrap();

    // Doctor starts consultation
    let doctor_token = get_auth_token(&mut app, &doctor_email, &doctor_password).await;

    let (status, _) = app
        .put_with_auth(
            &format!("/api/v1/video-consultations/{}/start", consultation_id),
            json!({}),
            &doctor_token,
        )
        .await;

    assert_eq!(status, StatusCode::OK);

    // Verify consultation status updated
    let check_query = r#"
        SELECT status, actual_start_time FROM video_consultations WHERE id = ?
    "#;

    #[derive(sqlx::FromRow)]
    struct ConsultationStatus {
        status: String,
        actual_start_time: Option<chrono::DateTime<chrono::Utc>>,
    }

    let result: ConsultationStatus = sqlx::query_as(check_query)
        .bind(consultation_id.to_string())
        .fetch_one(&app.pool)
        .await
        .unwrap();

    assert_eq!(result.status, "in_progress");
    assert!(result.actual_start_time.is_some());
}

#[tokio::test]
//#[serial]
async fn test_end_consultation() {
    let mut app = TestApp::new().await;

    // Create patient and doctor
    let (patient_id, _patient_email, _patient_password) =
        create_test_user(&app.pool, "patient").await;
    let (doctor_user_id, doctor_email, doctor_password) =
        create_test_user(&app.pool, "doctor").await;
    let (doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;

    // Create an appointment first
    let appointment_id = uuid::Uuid::new_v4();
    let now = Utc::now();
    let scheduled_time = now - Duration::minutes(30);

    let appointment_query = r#"
        INSERT INTO appointments (
            id, patient_id, doctor_id, appointment_date, time_slot,
            visit_type, symptoms, has_visited_before, status,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, 'online_video', ?, false, 'confirmed', ?, ?)
    "#;

    sqlx::query(appointment_query)
        .bind(appointment_id.to_string())
        .bind(patient_id.to_string())
        .bind(doctor_id.to_string())
        .bind(scheduled_time.naive_utc())
        .bind("09:00-10:00")
        .bind("test symptoms")
        .bind(now)
        .bind(now)
        .execute(&app.pool)
        .await
        .unwrap();

    // Create consultation in progress
    let consultation_id = uuid::Uuid::new_v4();
    let room_id = format!("room_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));

    let query = r#"
        INSERT INTO video_consultations (
            id, appointment_id, doctor_id, patient_id, room_id,
            status, scheduled_start_time, actual_start_time, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, 'in_progress', ?, ?, ?, ?)
    "#;

    sqlx::query(query)
        .bind(consultation_id.to_string())
        .bind(appointment_id.to_string())
        .bind(doctor_id.to_string())
        .bind(patient_id.to_string())
        .bind(&room_id)
        .bind(scheduled_time)
        .bind(now - Duration::minutes(25))
        .bind(now)
        .bind(now)
        .execute(&app.pool)
        .await
        .unwrap();

    // Doctor ends consultation
    let doctor_token = get_auth_token(&mut app, &doctor_email, &doctor_password).await;

    let end_dto = json!({
        "diagnosis": "神经性头痛，失眠症",
        "treatment_plan": "中药调理，针灸治疗",
        "notes": "患者症状明显，建议坚持治疗"
    });

    let (status, _) = app
        .put_with_auth(
            &format!("/api/v1/video-consultations/{}/end", consultation_id),
            end_dto,
            &doctor_token,
        )
        .await;

    assert_eq!(status, StatusCode::OK);

    // Verify consultation completed
    let check_query = r#"
        SELECT status, diagnosis, treatment_plan, notes, duration FROM video_consultations WHERE id = ?
    "#;

    #[derive(sqlx::FromRow)]
    struct ConsultationResult {
        status: String,
        diagnosis: String,
        #[allow(dead_code)]
        treatment_plan: String,
        #[allow(dead_code)]
        notes: String,
        duration: Option<i32>,
    }

    let result: ConsultationResult = sqlx::query_as(check_query)
        .bind(consultation_id.to_string())
        .fetch_one(&app.pool)
        .await
        .unwrap();

    assert_eq!(result.status, "completed");
    assert_eq!(result.diagnosis, "神经性头痛，失眠症");
    assert!(result.duration.is_some());
}

#[tokio::test]
//#[serial]
async fn test_rate_consultation() {
    let mut app = TestApp::new().await;

    // Create patient and doctor
    let (patient_id, patient_email, patient_password) =
        create_test_user(&app.pool, "patient").await;
    let (doctor_user_id, _doctor_email, _doctor_password) =
        create_test_user(&app.pool, "doctor").await;
    let (doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;

    // Create an appointment first
    let appointment_id = uuid::Uuid::new_v4();
    let now = Utc::now();
    let scheduled_time = now - Duration::hours(2);

    let appointment_query = r#"
        INSERT INTO appointments (
            id, patient_id, doctor_id, appointment_date, time_slot,
            visit_type, symptoms, has_visited_before, status,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, 'online_video', ?, false, 'completed', ?, ?)
    "#;

    sqlx::query(appointment_query)
        .bind(appointment_id.to_string())
        .bind(patient_id.to_string())
        .bind(doctor_id.to_string())
        .bind(scheduled_time.naive_utc())
        .bind("09:00-10:00")
        .bind("test symptoms")
        .bind(now)
        .bind(now)
        .execute(&app.pool)
        .await
        .unwrap();

    // Create completed consultation
    let consultation_id = uuid::Uuid::new_v4();
    let room_id = format!("room_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));

    let query = r#"
        INSERT INTO video_consultations (
            id, appointment_id, doctor_id, patient_id, room_id,
            status, scheduled_start_time, actual_start_time, end_time,
            duration, diagnosis, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, 'completed', ?, ?, ?, ?, ?, ?, ?)
    "#;

    sqlx::query(query)
        .bind(consultation_id.to_string())
        .bind(appointment_id.to_string())
        .bind(doctor_id.to_string())
        .bind(patient_id.to_string())
        .bind(&room_id)
        .bind(scheduled_time)
        .bind(scheduled_time)
        .bind(now - Duration::minutes(90))
        .bind(1800)
        .bind("头痛症")
        .bind(now)
        .bind(now)
        .execute(&app.pool)
        .await
        .unwrap();

    // Patient rates consultation
    let patient_token = get_auth_token(&mut app, &patient_email, &patient_password).await;

    let rate_dto = json!({
        "rating": 5,
        "feedback": "医生很专业，解答详细"
    });

    let (status, _) = app
        .post_with_auth(
            &format!("/api/v1/video-consultations/{}/rate", consultation_id),
            rate_dto,
            &patient_token,
        )
        .await;

    assert_eq!(status, StatusCode::OK);

    // Verify rating saved
    let check_query = r#"
        SELECT patient_rating, patient_feedback FROM video_consultations WHERE id = ?
    "#;

    #[derive(sqlx::FromRow)]
    struct ConsultationRating {
        patient_rating: i32,
        patient_feedback: String,
    }

    let result: ConsultationRating = sqlx::query_as(check_query)
        .bind(consultation_id.to_string())
        .fetch_one(&app.pool)
        .await
        .unwrap();

    assert_eq!(result.patient_rating, 5);
    assert_eq!(result.patient_feedback, "医生很专业，解答详细");
}

#[tokio::test]
//#[serial]
async fn test_webrtc_signaling() {
    let mut app = TestApp::new().await;

    // Create patient and doctor
    let (patient_id, patient_email, patient_password) =
        create_test_user(&app.pool, "patient").await;
    let (doctor_user_id, doctor_email, doctor_password) =
        create_test_user(&app.pool, "doctor").await;
    let (doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;

    // Create an appointment first
    let appointment_id = uuid::Uuid::new_v4();
    let now = Utc::now();

    let appointment_query = r#"
        INSERT INTO appointments (
            id, patient_id, doctor_id, appointment_date, time_slot,
            visit_type, symptoms, has_visited_before, status,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, 'online_video', ?, false, 'confirmed', ?, ?)
    "#;

    sqlx::query(appointment_query)
        .bind(appointment_id.to_string())
        .bind(patient_id.to_string())
        .bind(doctor_id.to_string())
        .bind(now.naive_utc())
        .bind("09:00-10:00")
        .bind("test symptoms")
        .bind(now)
        .bind(now)
        .execute(&app.pool)
        .await
        .unwrap();

    // Create consultation
    let consultation_id = uuid::Uuid::new_v4();
    let room_id = format!("room_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));

    let query = r#"
        INSERT INTO video_consultations (
            id, appointment_id, doctor_id, patient_id, room_id,
            status, scheduled_start_time, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, 'in_progress', ?, ?, ?)
    "#;

    sqlx::query(query)
        .bind(consultation_id.to_string())
        .bind(appointment_id.to_string())
        .bind(doctor_id.to_string())
        .bind(patient_id.to_string())
        .bind(&room_id)
        .bind(now)
        .bind(now)
        .bind(now)
        .execute(&app.pool)
        .await
        .unwrap();

    // Doctor sends signal
    let doctor_token = get_auth_token(&mut app, &doctor_email, &doctor_password).await;

    let signal_dto = json!({
        "room_id": room_id,
        "to_user_id": patient_id,
        "signal_type": "offer",
        "payload": {
            "sdp": "v=0\r\no=- 123456789 2 IN IP4 127.0.0.1\r\ns=-\r\nt=0 0\r\n..."
        }
    });

    let (status, _) = app
        .post_with_auth(
            "/api/v1/video-consultations/signal",
            signal_dto,
            &doctor_token,
        )
        .await;

    assert_eq!(status, StatusCode::OK);

    // Patient receives signals
    let patient_token = get_auth_token(&mut app, &patient_email, &patient_password).await;

    let (status, body) = app
        .get_with_auth(
            &format!("/api/v1/video-consultations/signal/{}", room_id),
            &patient_token,
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());

    let signals = body["data"].as_array().unwrap();
    assert_eq!(signals.len(), 1);
    assert_eq!(signals[0]["signal_type"].as_str().unwrap(), "offer");
    assert_eq!(
        signals[0]["from_user_id"].as_str().unwrap(),
        doctor_user_id.to_string()
    );
}

#[tokio::test]
//#[serial]
async fn test_consultation_templates() {
    let mut app = TestApp::new().await;

    // Create doctor
    let (doctor_user_id, doctor_email, doctor_password) =
        create_test_user(&app.pool, "doctor").await;
    let (_, _) = create_test_doctor(&app.pool, doctor_user_id).await;
    let doctor_token = get_auth_token(&mut app, &doctor_email, &doctor_password).await;

    // Create template
    let template_dto = json!({
        "name": "失眠症常规诊断",
        "chief_complaint": "失眠、多梦、早醒",
        "diagnosis": "失眠症",
        "treatment_plan": "安神补脑液，酸枣仁汤加减",
        "notes": "注意作息规律，避免熬夜"
    });

    let (status, body) = app
        .post_with_auth(
            "/api/v1/video-consultations/templates",
            template_dto,
            &doctor_token,
        )
        .await;

    assert_eq!(status, StatusCode::CREATED);

    let template_id = body["data"]["id"].as_str().unwrap();

    // List templates
    let (status, body) = app
        .get_with_auth("/api/v1/video-consultations/templates", &doctor_token)
        .await;

    assert_eq!(status, StatusCode::OK);

    let templates = body["data"].as_array().unwrap();
    assert!(!templates.is_empty());

    // Use template
    let (status, _) = app
        .post_with_auth(
            &format!("/api/v1/video-consultations/templates/{}/use", template_id),
            json!({}),
            &doctor_token,
        )
        .await;

    assert_eq!(status, StatusCode::OK);

    // Verify usage count increased
    let check_query = r#"
        SELECT usage_count FROM video_consultation_templates WHERE id = ?
    "#;

    #[derive(sqlx::FromRow)]
    struct TemplateUsage {
        usage_count: i32,
    }

    let result: TemplateUsage = sqlx::query_as(check_query)
        .bind(uuid::Uuid::parse_str(template_id).unwrap().to_string())
        .fetch_one(&app.pool)
        .await
        .unwrap();

    assert_eq!(result.usage_count, 1);
}

#[tokio::test]
//#[serial]
async fn test_consultation_statistics() {
    let mut app = TestApp::new().await;

    // Create doctor
    let (doctor_user_id, doctor_email, doctor_password) =
        create_test_user(&app.pool, "doctor").await;
    let (doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;
    let doctor_token = get_auth_token(&mut app, &doctor_email, &doctor_password).await;

    // Create some consultations with different statuses
    let now = Utc::now();

    // Create patient for first consultation
    let (patient1_id, _, _) = create_test_user(&app.pool, "patient").await;

    // Create appointment and completed consultation
    let appointment1_id = uuid::Uuid::new_v4();
    let scheduled_time1 = now - Duration::days(1);

    let appointment_query = r#"
        INSERT INTO appointments (
            id, patient_id, doctor_id, appointment_date, time_slot,
            visit_type, symptoms, has_visited_before, status,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, 'online_video', ?, false, 'completed', ?, ?)
    "#;

    sqlx::query(appointment_query)
        .bind(appointment1_id.to_string())
        .bind(patient1_id.to_string())
        .bind(doctor_id.to_string())
        .bind(scheduled_time1.naive_utc())
        .bind("09:00-10:00")
        .bind("test symptoms")
        .bind(now)
        .bind(now)
        .execute(&app.pool)
        .await
        .unwrap();

    // Completed consultation
    let query = r#"
        INSERT INTO video_consultations (
            id, appointment_id, doctor_id, patient_id, room_id,
            status, scheduled_start_time, duration, patient_rating,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, 'completed', ?, ?, ?, ?, ?)
    "#;

    sqlx::query(query)
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(appointment1_id.to_string())
        .bind(doctor_id.to_string())
        .bind(patient1_id.to_string())
        .bind(format!(
            "room_{}",
            uuid::Uuid::new_v4().to_string().replace("-", "")
        ))
        .bind(scheduled_time1)
        .bind(1800)
        .bind(5)
        .bind(now)
        .bind(now)
        .execute(&app.pool)
        .await
        .unwrap();

    // Create patient for second consultation
    let (patient2_id, _, _) = create_test_user(&app.pool, "patient").await;

    // Create appointment and no-show consultation
    let appointment2_id = uuid::Uuid::new_v4();
    let scheduled_time2 = now - Duration::days(2);

    sqlx::query(appointment_query)
        .bind(appointment2_id.to_string())
        .bind(patient2_id.to_string())
        .bind(doctor_id.to_string())
        .bind(scheduled_time2.naive_utc())
        .bind("10:00-11:00")
        .bind("test symptoms")
        .bind(now)
        .bind(now)
        .execute(&app.pool)
        .await
        .unwrap();

    // No-show consultation
    let no_show_query = r#"
        INSERT INTO video_consultations (
            id, appointment_id, doctor_id, patient_id, room_id,
            status, scheduled_start_time, duration, patient_rating,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, 'no_show', ?, NULL, NULL, ?, ?)
    "#;
    sqlx::query(no_show_query)
        .bind(uuid::Uuid::new_v4().to_string())
        .bind(appointment2_id.to_string())
        .bind(doctor_id.to_string())
        .bind(patient2_id.to_string())
        .bind(format!(
            "room_{}",
            uuid::Uuid::new_v4().to_string().replace("-", "")
        ))
        .bind(scheduled_time2)
        .bind(now)
        .bind(now)
        .execute(&app.pool)
        .await
        .unwrap();

    // Get statistics
    let (status, body) = app
        .get_with_auth("/api/v1/video-consultations/statistics", &doctor_token)
        .await;

    assert_eq!(status, StatusCode::OK);

    let stats = &body["data"];

    assert!(stats["total_consultations"].as_i64().unwrap() >= 2);
    assert!(stats["completed_consultations"].as_i64().unwrap() >= 1);
    assert!(stats["average_duration"].as_f64().is_some());
    assert!(stats["average_rating"].as_f64().is_some());
    assert!(stats["no_show_rate"].as_f64().is_some());
}

#[tokio::test]
//#[serial]
async fn test_authorization() {
    let mut app = TestApp::new().await;

    // Create two doctors
    let (doctor1_user_id, _, _) = create_test_user(&app.pool, "doctor").await;
    let (doctor1_id, _) = create_test_doctor(&app.pool, doctor1_user_id).await;
    let (doctor2_user_id, doctor2_email, doctor2_password) =
        create_test_user(&app.pool, "doctor").await;
    let (_doctor2_id, _) = create_test_doctor(&app.pool, doctor2_user_id).await;

    // Create patient for the consultation
    let (patient_id, _, _) = create_test_user(&app.pool, "patient").await;

    // Create an appointment first
    let appointment_id = uuid::Uuid::new_v4();
    let now = Utc::now();

    let appointment_query = r#"
        INSERT INTO appointments (
            id, patient_id, doctor_id, appointment_date, time_slot,
            visit_type, symptoms, has_visited_before, status,
            created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, 'online_video', ?, false, 'confirmed', ?, ?)
    "#;

    sqlx::query(appointment_query)
        .bind(appointment_id.to_string())
        .bind(patient_id.to_string())
        .bind(doctor1_id.to_string())
        .bind(now.naive_utc())
        .bind("09:00-10:00")
        .bind("test symptoms")
        .bind(now)
        .bind(now)
        .execute(&app.pool)
        .await
        .unwrap();

    // Create consultation for doctor1
    let consultation_id = uuid::Uuid::new_v4();
    let room_id = format!("room_{}", uuid::Uuid::new_v4().to_string().replace("-", ""));

    let query = r#"
        INSERT INTO video_consultations (
            id, appointment_id, doctor_id, patient_id, room_id,
            status, scheduled_start_time, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, 'waiting', ?, ?, ?)
    "#;

    sqlx::query(query)
        .bind(consultation_id.to_string())
        .bind(appointment_id.to_string())
        .bind(doctor1_id.to_string())
        .bind(patient_id.to_string())
        .bind(&room_id)
        .bind(now)
        .bind(now)
        .bind(now)
        .execute(&app.pool)
        .await
        .unwrap();

    // Doctor2 tries to access doctor1's consultation
    let doctor2_token = get_auth_token(&mut app, &doctor2_email, &doctor2_password).await;

    let (status, _) = app
        .get_with_auth(
            &format!("/api/v1/video-consultations/{}", consultation_id),
            &doctor2_token,
        )
        .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
}
