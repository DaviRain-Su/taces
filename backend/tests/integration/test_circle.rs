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
async fn test_create_circle() {
    let mut app = TestApp::new().await;

    // Create a test user
    let (user_id, account, password) = create_test_user(&app.pool, "patient").await;
    let token = get_auth_token(&mut app, &account, &password).await;

    // Create a circle
    let create_dto = json!({
        "name": "Test Circle",
        "description": "A test circle for unit testing",
        "category": "测试分类"
    });

    let (status, body) = app
        .post_with_auth("/api/v1/circles", create_dto, &token)
        .await;

    if status != StatusCode::OK {
        println!("Create circle failed with status {}: {:?}", status, body);
    }
    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(
        body["message"].as_str().unwrap(),
        "Circle created successfully"
    );

    let circle = &body["data"];
    assert_eq!(circle["name"].as_str().unwrap(), "Test Circle");
    assert_eq!(
        circle["description"].as_str().unwrap(),
        "A test circle for unit testing"
    );
    assert_eq!(circle["category"].as_str().unwrap(), "测试分类");
    assert_eq!(circle["member_count"].as_i64().unwrap(), 1);
    assert_eq!(circle["creator_id"].as_str().unwrap(), user_id.to_string());
}

#[tokio::test]
async fn test_get_circles() {
    let mut app = TestApp::new().await;

    // Create users
    let (_user1_id, account1, password1) = create_test_user(&app.pool, "patient").await;
    let token1 = get_auth_token(&mut app, &account1, &password1).await;

    let (_user2_id, account2, password2) = create_test_user(&app.pool, "patient").await;
    let token2 = get_auth_token(&mut app, &account2, &password2).await;

    // User1 creates two circles
    let create_dto1 = json!({
        "name": "中医养生",
        "description": "讨论中医养生知识",
        "category": "健康"
    });
    app.post_with_auth("/api/v1/circles", create_dto1, &token1)
        .await;

    let create_dto2 = json!({
        "name": "慢病调理",
        "description": "慢性病患者交流",
        "category": "健康"
    });
    app.post_with_auth("/api/v1/circles", create_dto2, &token1)
        .await;

    // Get all circles
    let (status, body) = app.get_with_auth("/api/v1/circles", &token2).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());

    let circles = body["data"]["circles"].as_array().unwrap();
    assert!(circles.len() >= 2);

    // Search by category
    let (status, body) = app
        .get_with_auth("/api/v1/circles?category=健康", &token2)
        .await;
    assert_eq!(status, StatusCode::OK);

    let circles = body["data"]["circles"].as_array().unwrap();
    assert!(circles
        .iter()
        .all(|c| c["category"].as_str().unwrap() == "健康"));
}

#[tokio::test]
async fn test_join_and_leave_circle() {
    let mut app = TestApp::new().await;

    // Create two users
    let (_user1_id, account1, password1) = create_test_user(&app.pool, "patient").await;
    let token1 = get_auth_token(&mut app, &account1, &password1).await;

    let (_user2_id, account2, password2) = create_test_user(&app.pool, "patient").await;
    let token2 = get_auth_token(&mut app, &account2, &password2).await;

    // User1 creates a circle
    let create_dto = json!({
        "name": "Join Test Circle",
        "description": "Testing join functionality",
        "category": "测试"
    });

    let (status, body) = app
        .post_with_auth("/api/v1/circles", create_dto, &token1)
        .await;
    assert_eq!(status, StatusCode::OK);

    let circle_id = body["data"]["id"].as_str().unwrap();

    // User2 joins the circle
    let (status, body) = app
        .post_with_auth(
            &format!("/api/v1/circles/{}/join", circle_id),
            json!({}),
            &token2,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["message"].as_str().unwrap(),
        "Joined circle successfully"
    );

    // Get circle info to verify member count
    let (status, body) = app
        .get_with_auth(&format!("/api/v1/circles/{}", circle_id), &token2)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["circle"]["member_count"].as_i64().unwrap(), 2);
    assert!(body["data"]["is_joined"].as_bool().unwrap());
    assert_eq!(body["data"]["member_role"].as_str().unwrap(), "member");

    // User2 leaves the circle
    let (status, body) = app
        .post_with_auth(
            &format!("/api/v1/circles/{}/leave", circle_id),
            json!({}),
            &token2,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(
        body["message"].as_str().unwrap(),
        "Left circle successfully"
    );

    // Get circle info to verify member count decreased
    let (status, body) = app
        .get_with_auth(&format!("/api/v1/circles/{}", circle_id), &token2)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["circle"]["member_count"].as_i64().unwrap(), 1);
    assert!(!body["data"]["is_joined"].as_bool().unwrap());
}

#[tokio::test]
async fn test_circle_member_management() {
    let mut app = TestApp::new().await;

    // Create three users
    let (_owner_id, owner_account, owner_password) = create_test_user(&app.pool, "patient").await;
    let owner_token = get_auth_token(&mut app, &owner_account, &owner_password).await;

    let (user2_id, account2, password2) = create_test_user(&app.pool, "patient").await;
    let member_token = get_auth_token(&mut app, &account2, &password2).await;

    let (user3_id, account3, password3) = create_test_user(&app.pool, "patient").await;
    let member3_token = get_auth_token(&mut app, &account3, &password3).await;

    // Owner creates a circle
    let create_dto = json!({
        "name": "Member Management Test",
        "description": "Testing member management",
        "category": "测试"
    });

    let (status, body) = app
        .post_with_auth("/api/v1/circles", create_dto, &owner_token)
        .await;
    assert_eq!(status, StatusCode::OK);

    let circle_id = body["data"]["id"].as_str().unwrap();

    // Members join
    app.post_with_auth(
        &format!("/api/v1/circles/{}/join", circle_id),
        json!({}),
        &member_token,
    )
    .await;
    app.post_with_auth(
        &format!("/api/v1/circles/{}/join", circle_id),
        json!({}),
        &member3_token,
    )
    .await;

    // Owner promotes member2 to admin
    let (status, _body) = app
        .put_with_auth(
            &format!("/api/v1/circles/{}/members/{}/role", circle_id, user2_id),
            json!({ "role": "admin" }),
            &owner_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Get members list
    let (status, body) = app
        .get_with_auth(
            &format!("/api/v1/circles/{}/members", circle_id),
            &owner_token,
        )
        .await;
    if status != StatusCode::OK {
        println!("Get members failed with status {}: {:?}", status, body);
    }
    assert_eq!(status, StatusCode::OK);

    let members = body["data"]["members"].as_array().unwrap();
    assert_eq!(members.len(), 3);

    // Check member2's role is admin
    let member2_info = members
        .iter()
        .find(|m| m["user_id"].as_str().unwrap() == user2_id.to_string())
        .unwrap();
    assert_eq!(member2_info["role"].as_str().unwrap(), "admin");

    // Owner removes member3
    let (status, _body) = app
        .delete_with_auth(
            &format!("/api/v1/circles/{}/members/{}", circle_id, user3_id),
            &owner_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Verify member was removed
    let (status, body) = app
        .get_with_auth(
            &format!("/api/v1/circles/{}/members", circle_id),
            &owner_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    let members = body["data"]["members"].as_array().unwrap();
    assert_eq!(members.len(), 2);
}

#[tokio::test]
async fn test_circle_permissions() {
    let mut app = TestApp::new().await;

    // Create two users
    let (_owner_id, owner_account, owner_password) = create_test_user(&app.pool, "patient").await;
    let owner_token = get_auth_token(&mut app, &owner_account, &owner_password).await;

    let (_other_id, other_account, other_password) = create_test_user(&app.pool, "patient").await;
    let other_token = get_auth_token(&mut app, &other_account, &other_password).await;

    // Owner creates a circle
    let create_dto = json!({
        "name": "Permission Test",
        "description": "Testing permissions",
        "category": "测试"
    });

    let (status, body) = app
        .post_with_auth("/api/v1/circles", create_dto, &owner_token)
        .await;
    assert_eq!(status, StatusCode::OK);

    let circle_id = body["data"]["id"].as_str().unwrap();

    // Non-owner tries to update circle (should fail)
    let (status, body) = app
        .put_with_auth(
            &format!("/api/v1/circles/{}", circle_id),
            json!({ "name": "Hacked Name" }),
            &other_token,
        )
        .await;
    println!("Update response - status: {}, body: {:?}", status, body);
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Non-owner tries to delete circle (should fail)
    let (status, body) = app
        .delete_with_auth(&format!("/api/v1/circles/{}", circle_id), &other_token)
        .await;
    println!("Delete response - status: {}, body: {:?}", status, body);
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Owner can update
    let (status, body) = app
        .put_with_auth(
            &format!("/api/v1/circles/{}", circle_id),
            json!({ "name": "Updated Name" }),
            &owner_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["name"].as_str().unwrap(), "Updated Name");
}
