use crate::common::TestApp;
use axum::http::StatusCode;
use backend::{
    models::user::LoginDto,
    utils::test_helpers::{create_test_doctor, create_test_user},
};
use chrono::{Duration, Utc};
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
async fn test_live_stream_crud() {
    let mut app = TestApp::new().await;

    // Create users
    let (_admin_id, admin_account, admin_password) = create_test_user(&app.pool, "admin").await;
    let (doctor_id, doctor_account, doctor_password) = create_test_user(&app.pool, "doctor").await;
    let (_patient_id, patient_account, patient_password) =
        create_test_user(&app.pool, "patient").await;
    let (_doctor_record_id, _) = create_test_doctor(&app.pool, doctor_id).await;

    let admin_token = get_auth_token(&mut app, &admin_account, &admin_password).await;
    let doctor_token = get_auth_token(&mut app, &doctor_account, &doctor_password).await;
    let patient_token = get_auth_token(&mut app, &patient_account, &patient_password).await;

    // Patient cannot create live stream
    let create_dto = json!({
        "title": "中医养生讲座",
        "scheduled_time": (Utc::now() + Duration::hours(2)).to_rfc3339()
    });

    let (status, _) = app
        .post_with_auth("/api/v1/live-streams", create_dto.clone(), &patient_token)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Doctor creates live stream
    let (status, body) = app
        .post_with_auth("/api/v1/live-streams", create_dto, &doctor_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());
    let stream_id = body["data"]["id"].as_str().unwrap();

    // List live streams (public)
    let (status, body) = app.get("/api/v1/live-streams").await;
    assert_eq!(status, StatusCode::OK);
    let streams = body["data"].as_array().unwrap();
    assert!(!streams.is_empty());

    // Get live stream by ID (public)
    let (status, body) = app
        .get(&format!("/api/v1/live-streams/{}", stream_id))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["title"], "中医养生讲座");
    assert_eq!(body["data"]["status"], "scheduled");

    // Update live stream
    let update_dto = json!({
        "title": "中医养生讲座（更新版）"
    });

    let (status, body) = app
        .put_with_auth(
            &format!("/api/v1/live-streams/{}", stream_id),
            update_dto,
            &doctor_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["title"], "中医养生讲座（更新版）");

    // Start live stream
    let start_dto = json!({
        "stream_url": "https://live.example.com/stream123",
        "qr_code": "https://example.com/qr/stream123"
    });

    let (status, body) = app
        .post_with_auth(
            &format!("/api/v1/live-streams/{}/start", stream_id),
            start_dto,
            &doctor_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["status"], "live");
    assert!(body["data"]["stream_url"].is_string());

    // End live stream
    let (status, body) = app
        .post_with_auth(
            &format!("/api/v1/live-streams/{}/end", stream_id),
            json!({}),
            &doctor_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["status"], "ended");

    // Cannot delete ended stream
    let (status, _) = app
        .delete_with_auth(
            &format!("/api/v1/live-streams/{}", stream_id),
            &doctor_token,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);

    // Admin creates a scheduled stream to test deletion
    let create_dto = json!({
        "title": "测试删除直播",
        "scheduled_time": (Utc::now() + Duration::hours(3)).to_rfc3339()
    });

    let (status, body) = app
        .post_with_auth("/api/v1/live-streams", create_dto, &admin_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    let deletable_stream_id = body["data"]["id"].as_str().unwrap();

    // Delete scheduled stream
    let (status, _) = app
        .delete_with_auth(
            &format!("/api/v1/live-streams/{}", deletable_stream_id),
            &admin_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Verify deletion
    let (status, _) = app
        .get(&format!("/api/v1/live-streams/{}", deletable_stream_id))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_live_stream_permissions() {
    let mut app = TestApp::new().await;

    // Create users
    let (_admin_id, admin_account, admin_password) = create_test_user(&app.pool, "admin").await;
    let (doctor1_id, doctor1_account, doctor1_password) =
        create_test_user(&app.pool, "doctor").await;
    let (doctor2_id, doctor2_account, doctor2_password) =
        create_test_user(&app.pool, "doctor").await;
    let (_doctor1_record_id, _) = create_test_doctor(&app.pool, doctor1_id).await;
    let (_doctor2_record_id, _) = create_test_doctor(&app.pool, doctor2_id).await;

    let admin_token = get_auth_token(&mut app, &admin_account, &admin_password).await;
    let doctor1_token = get_auth_token(&mut app, &doctor1_account, &doctor1_password).await;
    let doctor2_token = get_auth_token(&mut app, &doctor2_account, &doctor2_password).await;

    // Doctor1 creates live stream
    let create_dto = json!({
        "title": "Doctor1的直播",
        "scheduled_time": (Utc::now() + Duration::hours(1)).to_rfc3339()
    });

    let (status, body) = app
        .post_with_auth("/api/v1/live-streams", create_dto, &doctor1_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    let stream_id = body["data"]["id"].as_str().unwrap();

    // Doctor2 cannot update doctor1's stream
    let update_dto = json!({
        "title": "尝试修改别人的直播"
    });

    let (status, _) = app
        .put_with_auth(
            &format!("/api/v1/live-streams/{}", stream_id),
            update_dto.clone(),
            &doctor2_token,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Admin can update any stream
    let (status, body) = app
        .put_with_auth(
            &format!("/api/v1/live-streams/{}", stream_id),
            update_dto,
            &admin_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["title"], "尝试修改别人的直播");

    // Doctor2 cannot start doctor1's stream
    let start_dto = json!({
        "stream_url": "https://live.example.com/unauthorized"
    });

    let (status, _) = app
        .post_with_auth(
            &format!("/api/v1/live-streams/{}/start", stream_id),
            start_dto.clone(),
            &doctor2_token,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Admin can start any stream
    let (status, body) = app
        .post_with_auth(
            &format!("/api/v1/live-streams/{}/start", stream_id),
            start_dto,
            &admin_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["status"], "live");
}

#[tokio::test]
async fn test_live_stream_status_transitions() {
    let mut app = TestApp::new().await;

    // Create doctor
    let (doctor_id, doctor_account, doctor_password) = create_test_user(&app.pool, "doctor").await;
    let (_doctor_record_id, _) = create_test_doctor(&app.pool, doctor_id).await;
    let doctor_token = get_auth_token(&mut app, &doctor_account, &doctor_password).await;

    // Create scheduled stream
    let create_dto = json!({
        "title": "状态转换测试",
        "scheduled_time": (Utc::now() + Duration::hours(1)).to_rfc3339()
    });

    let (status, body) = app
        .post_with_auth("/api/v1/live-streams", create_dto, &doctor_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    let stream_id = body["data"]["id"].as_str().unwrap();

    // Cannot end a scheduled stream
    let (status, body) = app
        .post_with_auth(
            &format!("/api/v1/live-streams/{}/end", stream_id),
            json!({}),
            &doctor_token,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("not currently live"));

    // Start the stream
    let start_dto = json!({
        "stream_url": "https://live.example.com/test"
    });

    let (status, _) = app
        .post_with_auth(
            &format!("/api/v1/live-streams/{}/start", stream_id),
            start_dto.clone(),
            &doctor_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Cannot start again
    let (status, body) = app
        .post_with_auth(
            &format!("/api/v1/live-streams/{}/start", stream_id),
            start_dto,
            &doctor_token,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("not in scheduled status"));

    // End the stream
    let (status, _) = app
        .post_with_auth(
            &format!("/api/v1/live-streams/{}/end", stream_id),
            json!({}),
            &doctor_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Cannot end again
    let (status, body) = app
        .post_with_auth(
            &format!("/api/v1/live-streams/{}/end", stream_id),
            json!({}),
            &doctor_token,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("not currently live"));

    // Cannot update ended stream
    let update_dto = json!({
        "title": "尝试更新已结束的直播"
    });

    let (status, body) = app
        .put_with_auth(
            &format!("/api/v1/live-streams/{}", stream_id),
            update_dto,
            &doctor_token,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("Cannot update ended"));
}

#[tokio::test]
async fn test_upcoming_live_streams() {
    let mut app = TestApp::new().await;

    // Create doctor
    let (doctor_id, doctor_account, doctor_password) = create_test_user(&app.pool, "doctor").await;
    let (_doctor_record_id, _) = create_test_doctor(&app.pool, doctor_id).await;
    let doctor_token = get_auth_token(&mut app, &doctor_account, &doctor_password).await;

    // Create multiple scheduled streams
    let future_times = [
        Duration::hours(1),
        Duration::hours(2),
        Duration::hours(3),
        Duration::days(1),
    ];

    for (i, duration) in future_times.iter().enumerate() {
        let create_dto = json!({
            "title": format!("未来直播 {}", i + 1),
            "scheduled_time": (Utc::now() + *duration).to_rfc3339()
        });

        let (status, _) = app
            .post_with_auth("/api/v1/live-streams", create_dto, &doctor_token)
            .await;
        assert_eq!(status, StatusCode::OK);
    }

    // Create a past stream (should fail)
    let past_dto = json!({
        "title": "过去的直播",
        "scheduled_time": (Utc::now() - Duration::hours(1)).to_rfc3339()
    });

    let (status, body) = app
        .post_with_auth("/api/v1/live-streams", past_dto, &doctor_token)
        .await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("must be in the future"));

    // Get upcoming streams
    let (status, body) = app.get("/api/v1/live-streams/upcoming").await;
    assert_eq!(status, StatusCode::OK);
    let upcoming = body["data"].as_array().unwrap();
    assert!(upcoming.len() >= 4);

    // Check they are ordered by scheduled time
    for i in 1..upcoming.len() {
        let prev_time = upcoming[i - 1]["scheduled_time"].as_str().unwrap();
        let curr_time = upcoming[i]["scheduled_time"].as_str().unwrap();
        assert!(prev_time <= curr_time);
    }
}

#[tokio::test]
async fn test_my_live_streams() {
    let mut app = TestApp::new().await;

    // Create two doctors
    let (doctor1_id, doctor1_account, doctor1_password) =
        create_test_user(&app.pool, "doctor").await;
    let (doctor2_id, doctor2_account, doctor2_password) =
        create_test_user(&app.pool, "doctor").await;
    let (_doctor1_record_id, _) = create_test_doctor(&app.pool, doctor1_id).await;
    let (_doctor2_record_id, _) = create_test_doctor(&app.pool, doctor2_id).await;

    let doctor1_token = get_auth_token(&mut app, &doctor1_account, &doctor1_password).await;
    let doctor2_token = get_auth_token(&mut app, &doctor2_account, &doctor2_password).await;

    // Doctor1 creates 2 streams
    for i in 1..=2 {
        let create_dto = json!({
            "title": format!("Doctor1 直播 {}", i),
            "scheduled_time": (Utc::now() + Duration::hours(i as i64)).to_rfc3339()
        });

        let (status, _) = app
            .post_with_auth("/api/v1/live-streams", create_dto, &doctor1_token)
            .await;
        assert_eq!(status, StatusCode::OK);
    }

    // Doctor2 creates 1 stream
    let create_dto = json!({
        "title": "Doctor2 直播",
        "scheduled_time": (Utc::now() + Duration::hours(1)).to_rfc3339()
    });

    let (status, _) = app
        .post_with_auth("/api/v1/live-streams", create_dto, &doctor2_token)
        .await;
    assert_eq!(status, StatusCode::OK);

    // Get doctor1's streams
    let (status, body) = app
        .get_with_auth("/api/v1/live-streams/my", &doctor1_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    let my_streams = body["data"].as_array().unwrap();
    assert_eq!(my_streams.len(), 2);
    assert!(my_streams
        .iter()
        .all(|s| s["host_name"].as_str().unwrap().contains("doctor")));

    // Get doctor2's streams
    let (status, body) = app
        .get_with_auth("/api/v1/live-streams/my", &doctor2_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    let my_streams = body["data"].as_array().unwrap();
    assert_eq!(my_streams.len(), 1);
    assert_eq!(my_streams[0]["title"], "Doctor2 直播");
}
