use crate::common::TestApp;
use axum::http::StatusCode;
use backend::models::{CreateUserDto, LoginDto, UserRole};
use serde_json::json;
use sqlx::Row;
use uuid::Uuid;

// 辅助函数：创建测试用户并获取token
async fn create_test_user_with_token(
    app: &mut TestApp,
    account: &str,
    role: UserRole,
) -> (Uuid, String) {
    let user_dto = CreateUserDto {
        account: account.to_string(),
        name: format!("测试{}", account),
        password: "password123".to_string(),
        gender: "男".to_string(),
        phone: format!("138{:08}", rand::random::<u32>() % 100000000),
        email: Some(format!("{}@example.com", account)),
        birthday: None,
        role,
    };

    // 注册用户
    let (status, body) = app.post("/api/v1/auth/register", user_dto).await;
    assert_eq!(status, StatusCode::OK);
    let user_id = Uuid::parse_str(body["data"]["id"].as_str().unwrap()).unwrap();

    // 登录获取token
    let login_dto = LoginDto {
        account: account.to_string(),
        password: "password123".to_string(),
    };
    let (status, body) = app.post("/api/v1/auth/login", login_dto).await;
    assert_eq!(status, StatusCode::OK);
    let token = body["data"]["token"].as_str().unwrap().to_string();

    (user_id, token)
}

// 辅助函数：为医生创建医生档案
async fn create_doctor_profile(app: &mut TestApp, user_id: Uuid) -> Uuid {
    sqlx::query(
        r#"
        INSERT INTO doctors (id, user_id, certificate_type, id_number, hospital, department, title)
        VALUES (?, ?, 'ID_CARD', '110101199001011234', '香河香草中医诊所', '中医科', '主治医师')
        "#,
    )
    .bind(Uuid::new_v4().to_string())
    .bind(user_id.to_string())
    .execute(&app.pool)
    .await
    .unwrap();

    // 获取doctor_id
    let row = sqlx::query("SELECT id FROM doctors WHERE user_id = ?")
        .bind(user_id.to_string())
        .fetch_one(&app.pool)
        .await
        .unwrap();

    let doctor_id: String = row.try_get("id").unwrap();
    Uuid::parse_str(&doctor_id).unwrap()
}

#[tokio::test]
async fn test_create_review() {
    let mut app = TestApp::new().await;

    // 创建患者和医生
    let (patient_id, patient_token) =
        create_test_user_with_token(&mut app, "patient1", UserRole::Patient).await;
    let (doctor_user_id, _doctor_token) =
        create_test_user_with_token(&mut app, "doctor1", UserRole::Doctor).await;
    let doctor_id = create_doctor_profile(&mut app, doctor_user_id).await;

    // 创建一个已完成的预约
    let appointment_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO appointments (id, patient_id, doctor_id, appointment_date, time_slot, symptoms, status)
        VALUES (?, ?, ?, DATE_ADD(NOW(), INTERVAL 1 DAY), 'morning', '测试症状', 'completed')
        "#,
    )
    .bind(appointment_id.to_string())
    .bind(patient_id.to_string())
    .bind(doctor_id.to_string())
    .execute(&app.pool)
    .await
    .unwrap();

    // 创建评价
    let create_review = json!({
        "appointment_id": appointment_id,
        "rating": 5,
        "attitude_rating": 5,
        "professionalism_rating": 5,
        "efficiency_rating": 4,
        "comment": "董医生医术精湛，态度友好！",
        "is_anonymous": false
    });

    let (status, body) = app
        .post_with_auth("/api/v1/reviews", create_review, &patient_token)
        .await;

    if status != StatusCode::CREATED {
        println!("Create review failed: status={:?}, body={:?}", status, body);
    }
    assert_eq!(status, StatusCode::CREATED);
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["data"]["rating"].as_i64().unwrap(), 5);
    assert_eq!(
        body["data"]["comment"].as_str().unwrap(),
        "董医生医术精湛，态度友好！"
    );
}

#[tokio::test]
async fn test_create_review_with_tags() {
    let mut app = TestApp::new().await;

    let (patient_id, patient_token) =
        create_test_user_with_token(&mut app, "patient2", UserRole::Patient).await;
    let (doctor_user_id, _doctor_token) =
        create_test_user_with_token(&mut app, "doctor2", UserRole::Doctor).await;
    let doctor_id = create_doctor_profile(&mut app, doctor_user_id).await;

    // 创建预约
    let appointment_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO appointments (id, patient_id, doctor_id, appointment_date, time_slot, symptoms, status)
        VALUES (?, ?, ?, DATE_ADD(NOW(), INTERVAL 1 DAY), 'morning', '测试症状', 'completed')
        "#,
    )
    .bind(appointment_id.to_string())
    .bind(patient_id.to_string())
    .bind(doctor_id.to_string())
    .execute(&app.pool)
    .await
    .unwrap();

    // 获取标签ID
    let tag_rows =
        sqlx::query("SELECT id FROM review_tags WHERE name IN ('医术精湛', '态度友好') LIMIT 2")
            .fetch_all(&app.pool)
            .await
            .unwrap();

    let tag_ids: Vec<String> = tag_rows
        .iter()
        .map(|row| row.try_get::<String, _>("id").unwrap())
        .collect();

    // 创建带标签的评价
    let create_review = json!({
        "appointment_id": appointment_id,
        "rating": 5,
        "attitude_rating": 5,
        "professionalism_rating": 5,
        "efficiency_rating": 5,
        "comment": "非常满意的就诊体验",
        "tag_ids": tag_ids,
        "is_anonymous": false
    });

    let (status, _body) = app
        .post_with_auth("/api/v1/reviews", create_review, &patient_token)
        .await;
    assert_eq!(status, StatusCode::CREATED);
}

#[tokio::test]
async fn test_cannot_review_uncompleted_appointment() {
    let mut app = TestApp::new().await;

    let (patient_id, patient_token) =
        create_test_user_with_token(&mut app, "patient3", UserRole::Patient).await;
    let (doctor_user_id, _doctor_token) =
        create_test_user_with_token(&mut app, "doctor3", UserRole::Doctor).await;
    let doctor_id = create_doctor_profile(&mut app, doctor_user_id).await;

    // 创建未完成的预约
    let appointment_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO appointments (id, patient_id, doctor_id, appointment_date, time_slot, symptoms, status)
        VALUES (?, ?, ?, DATE_ADD(NOW(), INTERVAL 1 DAY), 'morning', '测试症状', 'confirmed')
        "#,
    )
    .bind(appointment_id.to_string())
    .bind(patient_id.to_string())
    .bind(doctor_id.to_string())
    .execute(&app.pool)
    .await
    .unwrap();

    let create_review = json!({
        "appointment_id": appointment_id,
        "rating": 5,
        "attitude_rating": 5,
        "professionalism_rating": 5,
        "efficiency_rating": 5,
        "comment": "测试评价"
    });

    let (status, _body) = app
        .post_with_auth("/api/v1/reviews", create_review, &patient_token)
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_doctor_reply_to_review() {
    let mut app = TestApp::new().await;

    let (patient_id, patient_token) =
        create_test_user_with_token(&mut app, "patient4", UserRole::Patient).await;
    let (doctor_user_id, doctor_token) =
        create_test_user_with_token(&mut app, "doctor4", UserRole::Doctor).await;
    let doctor_id = create_doctor_profile(&mut app, doctor_user_id).await;

    // 创建预约和评价
    let appointment_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO appointments (id, patient_id, doctor_id, appointment_date, time_slot, symptoms, status)
        VALUES (?, ?, ?, DATE_ADD(NOW(), INTERVAL 1 DAY), 'morning', '测试症状', 'completed')
        "#,
    )
    .bind(appointment_id.to_string())
    .bind(patient_id.to_string())
    .bind(doctor_id.to_string())
    .execute(&app.pool)
    .await
    .unwrap();

    // 先创建评价
    let create_review = json!({
        "appointment_id": appointment_id,
        "rating": 5,
        "attitude_rating": 5,
        "professionalism_rating": 5,
        "efficiency_rating": 5,
        "comment": "非常满意"
    });

    let (status, body) = app
        .post_with_auth("/api/v1/reviews", create_review, &patient_token)
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let review_id = body["data"]["id"].as_str().unwrap();

    // 医生回复评价
    let reply = json!({
        "reply": "感谢您的认可，祝您身体健康！"
    });

    let (status, body) = app
        .post_with_auth(
            &format!("/api/v1/reviews/{}/reply", review_id),
            reply,
            &doctor_token,
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(
        body["data"]["reply"].as_str().unwrap(),
        "感谢您的认可，祝您身体健康！"
    );
}

#[tokio::test]
async fn test_get_doctor_reviews() {
    let mut app = TestApp::new().await;

    let (doctor_user_id, _doctor_token) =
        create_test_user_with_token(&mut app, "doctor5", UserRole::Doctor).await;
    let doctor_id = create_doctor_profile(&mut app, doctor_user_id).await;

    // 公开接口，不需要认证
    let (status, body) = app
        .get(&format!("/api/v1/reviews/doctor/{}/reviews", doctor_id))
        .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());
    assert!(body["data"]["reviews"].is_array());
}

#[tokio::test]
async fn test_get_doctor_statistics() {
    let mut app = TestApp::new().await;

    let (doctor_user_id, _doctor_token) =
        create_test_user_with_token(&mut app, "doctor6", UserRole::Doctor).await;
    let doctor_id = create_doctor_profile(&mut app, doctor_user_id).await;

    // 公开接口，不需要认证
    let (status, body) = app
        .get(&format!("/api/v1/reviews/doctor/{}/statistics", doctor_id))
        .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());
    assert!(body["data"]["total_reviews"].is_i64());
    assert!(body["data"]["average_rating"].is_f64());
}

#[tokio::test]
async fn test_admin_manage_review_visibility() {
    let mut app = TestApp::new().await;

    let (_admin_id, admin_token) =
        create_test_user_with_token(&mut app, "admin1", UserRole::Admin).await;
    let (patient_id, patient_token) =
        create_test_user_with_token(&mut app, "patient5", UserRole::Patient).await;
    let (doctor_user_id, _doctor_token) =
        create_test_user_with_token(&mut app, "doctor7", UserRole::Doctor).await;
    let doctor_id = create_doctor_profile(&mut app, doctor_user_id).await;

    // 创建评价
    let appointment_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO appointments (id, patient_id, doctor_id, appointment_date, time_slot, symptoms, status)
        VALUES (?, ?, ?, DATE_ADD(NOW(), INTERVAL 1 DAY), 'morning', '测试症状', 'completed')
        "#,
    )
    .bind(appointment_id.to_string())
    .bind(patient_id.to_string())
    .bind(doctor_id.to_string())
    .execute(&app.pool)
    .await
    .unwrap();

    let create_review = json!({
        "appointment_id": appointment_id,
        "rating": 1,
        "attitude_rating": 1,
        "professionalism_rating": 1,
        "efficiency_rating": 1,
        "comment": "差评内容"
    });

    let (status, body) = app
        .post_with_auth("/api/v1/reviews", create_review, &patient_token)
        .await;
    assert_eq!(status, StatusCode::CREATED);
    let review_id = body["data"]["id"].as_str().unwrap();

    // 管理员隐藏评价
    let update_visibility = json!({
        "is_visible": false
    });

    let (status, _body) = app
        .put_with_auth(
            &format!("/api/v1/reviews/{}/visibility", review_id),
            update_visibility,
            &admin_token,
        )
        .await;

    assert_eq!(status, StatusCode::OK);
}
