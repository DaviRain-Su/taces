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
async fn test_post_crud() {
    let mut app = TestApp::new().await;

    // Create users
    let (_user1_id, account1, password1) = create_test_user(&app.pool, "patient").await;
    let token1 = get_auth_token(&mut app, &account1, &password1).await;

    let (_user2_id, account2, password2) = create_test_user(&app.pool, "patient").await;
    let token2 = get_auth_token(&mut app, &account2, &password2).await;

    // User1 creates a circle
    let create_circle_dto = json!({
        "name": "Test Circle for Posts",
        "description": "Testing posts functionality",
        "category": "测试"
    });

    let (status, body) = app
        .post_with_auth("/api/v1/circles", create_circle_dto, &token1)
        .await;
    assert_eq!(status, StatusCode::OK);
    let circle_id = body["data"]["id"].as_str().unwrap();

    // User2 joins the circle
    app.post_with_auth(
        &format!("/api/v1/circles/{}/join", circle_id),
        json!({}),
        &token2,
    )
    .await;

    // User2 creates a post
    let create_post_dto = json!({
        "circle_id": circle_id,
        "title": "My First Post",
        "content": "This is my first post in the circle!",
        "images": ["image1.jpg", "image2.jpg"]
    });

    let (status, body) = app
        .post_with_auth("/api/v1/posts", create_post_dto, &token2)
        .await;
    if status != StatusCode::OK {
        println!("Create post failed with status {}: {:?}", status, body);
    }
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["title"].as_str().unwrap(), "My First Post");
    let post_id = body["data"]["id"].as_str().unwrap();

    // Get the post
    let (status, body) = app
        .get_with_auth(&format!("/api/v1/posts/{}", post_id), &token1)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["title"].as_str().unwrap(), "My First Post");
    assert_eq!(
        body["data"]["author_name"].as_str().unwrap(),
        "Test patient User"
    );
    assert!(!body["data"]["is_liked"].as_bool().unwrap());

    // Update the post
    let update_dto = json!({
        "title": "Updated Title",
        "content": "Updated content"
    });

    let (status, body) = app
        .put_with_auth(&format!("/api/v1/posts/{}", post_id), update_dto, &token2)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["title"].as_str().unwrap(), "Updated Title");

    // Non-author cannot update
    let (status, _) = app
        .put_with_auth(
            &format!("/api/v1/posts/{}", post_id),
            json!({"title": "Hacked"}),
            &token1,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Get circle posts
    let (status, body) = app
        .get_with_auth(&format!("/api/v1/circles/{}/posts", circle_id), &token1)
        .await;
    assert_eq!(status, StatusCode::OK);
    let posts = body["data"]["posts"].as_array().unwrap();
    assert_eq!(posts.len(), 1);
}

#[tokio::test]
async fn test_post_likes() {
    let mut app = TestApp::new().await;

    // Create users and circle
    let (_user1_id, account1, password1) = create_test_user(&app.pool, "patient").await;
    let token1 = get_auth_token(&mut app, &account1, &password1).await;

    let (_user2_id, account2, password2) = create_test_user(&app.pool, "patient").await;
    let token2 = get_auth_token(&mut app, &account2, &password2).await;

    // Create circle and both users join
    let (_, body) = app
        .post_with_auth(
            "/api/v1/circles",
            json!({
                "name": "Like Test Circle",
                "description": "Testing likes",
                "category": "测试"
            }),
            &token1,
        )
        .await;
    let circle_id = body["data"]["id"].as_str().unwrap();

    app.post_with_auth(
        &format!("/api/v1/circles/{}/join", circle_id),
        json!({}),
        &token2,
    )
    .await;

    // Create a post
    let (_, body) = app
        .post_with_auth(
            "/api/v1/posts",
            json!({
                "circle_id": circle_id,
                "title": "Test Post for Likes",
                "content": "Please like this post!",
                "images": []
            }),
            &token1,
        )
        .await;
    let post_id = body["data"]["id"].as_str().unwrap();

    // User2 likes the post
    let (status, body) = app
        .post_with_auth(
            &format!("/api/v1/posts/{}/like", post_id),
            json!({}),
            &token2,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["data"]["liked"].as_bool().unwrap());

    // Check post likes count
    let (status, body) = app
        .get_with_auth(&format!("/api/v1/posts/{}", post_id), &token2)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["likes"].as_i64().unwrap(), 1);
    assert!(body["data"]["is_liked"].as_bool().unwrap());

    // User2 unlikes the post
    let (status, body) = app
        .post_with_auth(
            &format!("/api/v1/posts/{}/like", post_id),
            json!({}),
            &token2,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(!body["data"]["liked"].as_bool().unwrap());

    // Check likes count decreased
    let (_, body) = app
        .get_with_auth(&format!("/api/v1/posts/{}", post_id), &token2)
        .await;
    assert_eq!(body["data"]["likes"].as_i64().unwrap(), 0);
    assert!(!body["data"]["is_liked"].as_bool().unwrap());
}

#[tokio::test]
async fn test_post_comments() {
    let mut app = TestApp::new().await;

    // Create users and circle
    let (_user1_id, account1, password1) = create_test_user(&app.pool, "patient").await;
    let token1 = get_auth_token(&mut app, &account1, &password1).await;

    let (_user2_id, account2, password2) = create_test_user(&app.pool, "patient").await;
    let token2 = get_auth_token(&mut app, &account2, &password2).await;

    // Create circle and both users join
    let (_, body) = app
        .post_with_auth(
            "/api/v1/circles",
            json!({
                "name": "Comment Test Circle",
                "description": "Testing comments",
                "category": "测试"
            }),
            &token1,
        )
        .await;
    let circle_id = body["data"]["id"].as_str().unwrap();

    app.post_with_auth(
        &format!("/api/v1/circles/{}/join", circle_id),
        json!({}),
        &token2,
    )
    .await;

    // Create a post
    let (_, body) = app
        .post_with_auth(
            "/api/v1/posts",
            json!({
                "circle_id": circle_id,
                "title": "Test Post for Comments",
                "content": "Please comment on this post!",
                "images": []
            }),
            &token1,
        )
        .await;
    let post_id = body["data"]["id"].as_str().unwrap();

    // User2 comments on the post
    let (status, body) = app
        .post_with_auth(
            &format!("/api/v1/posts/{}/comments", post_id),
            json!({
                "content": "This is a great post!"
            }),
            &token2,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    let comment_id = body["data"]["id"].as_str().unwrap();

    // Get comments
    let (status, body) = app
        .get_with_auth(&format!("/api/v1/posts/{}/comments", post_id), &token1)
        .await;
    assert_eq!(status, StatusCode::OK);
    let comments = body["data"]["comments"].as_array().unwrap();
    assert_eq!(comments.len(), 1);
    assert_eq!(
        comments[0]["content"].as_str().unwrap(),
        "This is a great post!"
    );
    assert_eq!(
        comments[0]["user_name"].as_str().unwrap(),
        "Test patient User"
    );

    // Check post comment count
    let (_, body) = app
        .get_with_auth(&format!("/api/v1/posts/{}", post_id), &token1)
        .await;
    assert_eq!(body["data"]["comments"].as_i64().unwrap(), 1);

    // User2 can delete own comment
    let (status, _) = app
        .delete_with_auth(&format!("/api/v1/comments/{}", comment_id), &token2)
        .await;
    assert_eq!(status, StatusCode::OK);

    // Check comment count decreased
    let (_, body) = app
        .get_with_auth(&format!("/api/v1/posts/{}", post_id), &token1)
        .await;
    assert_eq!(body["data"]["comments"].as_i64().unwrap(), 0);
}

#[tokio::test]
async fn test_sensitive_words() {
    let mut app = TestApp::new().await;

    // Create user and circle
    let (_user_id, account, password) = create_test_user(&app.pool, "patient").await;
    let token = get_auth_token(&mut app, &account, &password).await;

    let (_, body) = app
        .post_with_auth(
            "/api/v1/circles",
            json!({
                "name": "Sensitive Test Circle",
                "description": "Testing sensitive words",
                "category": "测试"
            }),
            &token,
        )
        .await;
    let circle_id = body["data"]["id"].as_str().unwrap();

    // Try to create post with sensitive word
    let (status, body) = app
        .post_with_auth(
            "/api/v1/posts",
            json!({
                "circle_id": circle_id,
                "title": "Normal Title",
                "content": "This contains 赌博 which is sensitive",
                "images": []
            }),
            &token,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("sensitive words"));

    // Try to create comment with sensitive word
    let (_, body) = app
        .post_with_auth(
            "/api/v1/posts",
            json!({
                "circle_id": circle_id,
                "title": "Clean Post",
                "content": "This is a clean post",
                "images": []
            }),
            &token,
        )
        .await;
    let post_id = body["data"]["id"].as_str().unwrap();

    let (status, body) = app
        .post_with_auth(
            &format!("/api/v1/posts/{}/comments", post_id),
            json!({
                "content": "This contains 诈骗 word"
            }),
            &token,
        )
        .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("sensitive words"));
}

#[tokio::test]
async fn test_non_member_cannot_post() {
    let mut app = TestApp::new().await;

    // Create users
    let (_user1_id, account1, password1) = create_test_user(&app.pool, "patient").await;
    let token1 = get_auth_token(&mut app, &account1, &password1).await;

    let (_user2_id, _account2, password2) = create_test_user(&app.pool, "patient").await;
    let token2 = get_auth_token(&mut app, &_account2, &password2).await;

    // User1 creates a circle
    let (_, body) = app
        .post_with_auth(
            "/api/v1/circles",
            json!({
                "name": "Members Only Circle",
                "description": "Only members can post",
                "category": "测试"
            }),
            &token1,
        )
        .await;
    let circle_id = body["data"]["id"].as_str().unwrap();

    // User2 (non-member) tries to post
    let (status, body) = app
        .post_with_auth(
            "/api/v1/posts",
            json!({
                "circle_id": circle_id,
                "title": "Unauthorized Post",
                "content": "I'm not a member",
                "images": []
            }),
            &token2,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("must be a member"));
}
