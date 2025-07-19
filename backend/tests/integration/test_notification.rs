use crate::common::TestApp;
use axum::http::StatusCode;
use backend::{
    models::{notification::*, user::LoginDto},
    utils::test_helpers::create_test_user,
};
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
async fn test_notification_lifecycle() {
    let mut app = TestApp::new().await;

    // Create user
    let (user_id, account, password) = create_test_user(&app.pool, "patient").await;
    let token = get_auth_token(&mut app, &account, &password).await;

    // Create admin for sending notifications
    let (_, admin_account, admin_password) = create_test_user(&app.pool, "admin").await;
    let admin_token = get_auth_token(&mut app, &admin_account, &admin_password).await;

    // Admin creates a notification for the user
    let notification_dto = json!({
        "user_id": user_id,
        "type": "appointment_reminder",
        "title": "预约提醒",
        "content": "您明天上午9点有预约",
        "related_id": null
    });

    // Note: We need a way to create notifications internally
    // For now, let's create it directly in the database
    sqlx::query!(
        r#"
        INSERT INTO notifications (user_id, type, title, content, status)
        VALUES ($1, 'appointment_reminder', $2, $3, 'unread')
        "#,
        user_id,
        "预约提醒",
        "您明天上午9点有预约"
    )
    .execute(&app.pool)
    .await
    .unwrap();

    // Get notifications list
    let (status, body) = app
        .get_with_auth("/api/v1/notifications?status=unread", &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["items"].as_array().unwrap().len(), 1);
    assert_eq!(body["data"]["items"][0]["title"], "预约提醒");
    assert_eq!(body["data"]["items"][0]["status"], "unread");

    let notification_id = body["data"]["items"][0]["id"].as_str().unwrap();

    // Get notification detail
    let (status, body) = app
        .get_with_auth(&format!("/api/v1/notifications/{}", notification_id), &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["title"], "预约提醒");

    // Mark as read
    let (status, body) = app
        .put_with_auth(
            &format!("/api/v1/notifications/{}/read", notification_id),
            json!({}),
            &token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);

    // Verify it's marked as read
    let (status, body) = app
        .get_with_auth("/api/v1/notifications?status=read", &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["items"].as_array().unwrap().len(), 1);
    assert_eq!(body["data"]["items"][0]["status"], "read");

    // Delete notification
    let (status, body) = app
        .delete_with_auth(&format!("/api/v1/notifications/{}", notification_id), &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);

    // Verify it's deleted
    let (status, body) = app
        .get_with_auth("/api/v1/notifications", &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["items"].as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn test_notification_stats() {
    let mut app = TestApp::new().await;

    // Create user
    let (user_id, account, password) = create_test_user(&app.pool, "patient").await;
    let token = get_auth_token(&mut app, &account, &password).await;

    // Create multiple notifications
    for i in 0..5 {
        sqlx::query!(
            r#"
            INSERT INTO notifications (user_id, type, title, content, status)
            VALUES ($1, 'system_announcement', $2, $3, $4)
            "#,
            user_id,
            format!("通知{}", i + 1),
            format!("这是第{}条通知", i + 1),
            if i < 3 { "unread" } else { "read" }
        )
        .execute(&app.pool)
        .await
        .unwrap();
    }

    // Get stats
    let (status, body) = app
        .get_with_auth("/api/v1/notifications/stats", &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["total_count"], 5);
    assert_eq!(body["data"]["unread_count"], 3);
    assert_eq!(body["data"]["read_count"], 2);
}

#[tokio::test]
async fn test_mark_all_as_read() {
    let mut app = TestApp::new().await;

    // Create user
    let (user_id, account, password) = create_test_user(&app.pool, "patient").await;
    let token = get_auth_token(&mut app, &account, &password).await;

    // Create multiple unread notifications
    for i in 0..3 {
        sqlx::query!(
            r#"
            INSERT INTO notifications (user_id, type, title, content, status)
            VALUES ($1, 'system_announcement', $2, $3, 'unread')
            "#,
            user_id,
            format!("通知{}", i + 1),
            format!("这是第{}条通知", i + 1)
        )
        .execute(&app.pool)
        .await
        .unwrap();
    }

    // Mark all as read
    let (status, body) = app
        .put_with_auth("/api/v1/notifications/read-all", json!({}), &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["count"], 3);

    // Verify all are read
    let (status, body) = app
        .get_with_auth("/api/v1/notifications/stats", &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["unread_count"], 0);
    assert_eq!(body["data"]["read_count"], 3);
}

#[tokio::test]
async fn test_notification_settings() {
    let mut app = TestApp::new().await;

    // Create user
    let (_, account, password) = create_test_user(&app.pool, "patient").await;
    let token = get_auth_token(&mut app, &account, &password).await;

    // Get initial settings (should be empty)
    let (status, body) = app
        .get_with_auth("/api/v1/notifications/settings", &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"].as_array().unwrap().len(), 0);

    // Update settings
    let update_dto = json!({
        "notification_type": "appointment_reminder",
        "enabled": true,
        "email_enabled": true,
        "sms_enabled": false,
        "push_enabled": true
    });

    let (status, body) = app
        .put_with_auth("/api/v1/notifications/settings", update_dto, &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["enabled"], true);
    assert_eq!(body["data"]["email_enabled"], true);
    assert_eq!(body["data"]["sms_enabled"], false);
    assert_eq!(body["data"]["push_enabled"], true);

    // Get updated settings
    let (status, body) = app
        .get_with_auth("/api/v1/notifications/settings", &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"].as_array().unwrap().len(), 1);
}

#[tokio::test]
async fn test_push_token_registration() {
    let mut app = TestApp::new().await;

    // Create user
    let (_, account, password) = create_test_user(&app.pool, "patient").await;
    let token = get_auth_token(&mut app, &account, &password).await;

    // Register push token
    let register_dto = json!({
        "device_type": "ios",
        "token": "fake-push-token-12345",
        "device_info": {
            "model": "iPhone 14 Pro",
            "os_version": "17.0"
        }
    });

    let (status, body) = app
        .post_with_auth("/api/v1/notifications/push-token", register_dto, &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["device_type"], "ios");
    assert_eq!(body["data"]["token"], "fake-push-token-12345");
    assert_eq!(body["data"]["active"], true);
}

#[tokio::test]
async fn test_system_announcement() {
    let mut app = TestApp::new().await;

    // Create admin and regular user
    let (_, admin_account, admin_password) = create_test_user(&app.pool, "admin").await;
    let admin_token = get_auth_token(&mut app, &admin_account, &admin_password).await;

    let (_, user_account, user_password) = create_test_user(&app.pool, "patient").await;
    let user_token = get_auth_token(&mut app, &user_account, &user_password).await;

    // Admin sends system announcement
    let announcement_dto = json!({
        "user_id": "00000000-0000-0000-0000-000000000000", // dummy, will be ignored
        "type": "system_announcement",
        "title": "系统维护通知",
        "content": "系统将于今晚10点进行维护"
    });

    let (status, body) = app
        .post_with_auth("/api/v1/notifications/announcement", announcement_dto.clone(), &admin_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["data"]["count"].as_i64().unwrap() > 0);

    // Regular user tries to send announcement (should fail)
    let (status, body) = app
        .post_with_auth("/api/v1/notifications/announcement", announcement_dto, &user_token)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["success"], false);
}

#[tokio::test]
async fn test_notification_pagination() {
    let mut app = TestApp::new().await;

    // Create user
    let (user_id, account, password) = create_test_user(&app.pool, "patient").await;
    let token = get_auth_token(&mut app, &account, &password).await;

    // Create 25 notifications
    for i in 0..25 {
        sqlx::query!(
            r#"
            INSERT INTO notifications (user_id, type, title, content, status)
            VALUES ($1, 'system_announcement', $2, $3, 'unread')
            "#,
            user_id,
            format!("通知{}", i + 1),
            format!("这是第{}条通知", i + 1)
        )
        .execute(&app.pool)
        .await
        .unwrap();
    }

    // Get first page
    let (status, body) = app
        .get_with_auth("/api/v1/notifications?page=1&page_size=10", &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["items"].as_array().unwrap().len(), 10);
    assert_eq!(body["data"]["total"], 25);
    assert_eq!(body["data"]["page"], 1);
    assert_eq!(body["data"]["page_size"], 10);

    // Get second page
    let (status, body) = app
        .get_with_auth("/api/v1/notifications?page=2&page_size=10", &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["items"].as_array().unwrap().len(), 10);
    assert_eq!(body["data"]["page"], 2);

    // Get third page
    let (status, body) = app
        .get_with_auth("/api/v1/notifications?page=3&page_size=10", &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["items"].as_array().unwrap().len(), 5);
    assert_eq!(body["data"]["page"], 3);
}

#[tokio::test]
async fn test_notification_authorization() {
    let mut app = TestApp::new().await;

    // Create two users
    let (user1_id, account1, password1) = create_test_user(&app.pool, "patient").await;
    let token1 = get_auth_token(&mut app, &account1, &password1).await;

    let (_, account2, password2) = create_test_user(&app.pool, "patient").await;
    let token2 = get_auth_token(&mut app, &account2, &password2).await;

    // Create notification for user1
    let notification_id = sqlx::query_scalar!(
        r#"
        INSERT INTO notifications (user_id, type, title, content, status)
        VALUES ($1, 'appointment_reminder', '预约提醒', '您有新的预约', 'unread')
        RETURNING id
        "#,
        user1_id
    )
    .fetch_one(&app.pool)
    .await
    .unwrap();

    // User1 can access their notification
    let (status, _) = app
        .get_with_auth(&format!("/api/v1/notifications/{}", notification_id), &token1)
        .await;
    assert_eq!(status, StatusCode::OK);

    // User2 cannot access user1's notification
    let (status, body) = app
        .get_with_auth(&format!("/api/v1/notifications/{}", notification_id), &token2)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["success"], false);

    // User2 cannot mark user1's notification as read
    let (status, body) = app
        .put_with_auth(
            &format!("/api/v1/notifications/{}/read", notification_id),
            json!({}),
            &token2,
        )
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["success"], false);

    // User2 cannot delete user1's notification
    let (status, body) = app
        .delete_with_auth(&format!("/api/v1/notifications/{}", notification_id), &token2)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(body["success"], false);
}