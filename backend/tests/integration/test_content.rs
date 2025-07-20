use crate::common::TestApp;
use axum::http::StatusCode;
use backend::{
    models::user::LoginDto,
    utils::test_helpers::{create_test_doctor, create_test_user},
};
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
async fn test_article_crud() {
    let mut app = TestApp::new().await;

    // Create users
    let (_admin_id, admin_account, admin_password) = create_test_user(&app.pool, "admin").await;
    let (doctor_id, doctor_account, doctor_password) = create_test_user(&app.pool, "doctor").await;
    let (_doctor_record_id, _) = create_test_doctor(&app.pool, doctor_id).await;

    let _admin_token = get_auth_token(&mut app, &admin_account, &admin_password).await;
    let doctor_token = get_auth_token(&mut app, &doctor_account, &doctor_password).await;

    // Create article as doctor
    let create_dto = json!({
        "title": "中医养生之道",
        "summary": "介绍中医养生的基本理念和方法",
        "content": "中医养生强调天人合一，顺应自然...",
        "category": "健康科普",
        "tags": ["中医", "养生", "健康"],
        "publish_channels": ["官网新闻", "手机端"]
    });

    let (status, body) = app
        .post_with_auth("/api/v1/content/articles", create_dto, &doctor_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());
    let article_id = body["data"]["id"].as_str().unwrap();

    // List articles
    let (status, body) = app.get("/api/v1/content/articles").await;
    assert_eq!(status, StatusCode::OK);
    let articles = body["data"].as_array().unwrap();
    assert!(!articles.is_empty());

    // Get article by ID
    let (status, body) = app
        .get(&format!("/api/v1/content/articles/{}", article_id))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["title"], "中医养生之道");
    assert_eq!(body["data"]["status"], "draft");

    // Update article
    let update_dto = json!({
        "title": "中医养生之道（更新版）",
        "content": "更新后的内容..."
    });

    let (status, body) = app
        .put_with_auth(
            &format!("/api/v1/content/articles/{}", article_id),
            update_dto,
            &doctor_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["title"], "中医养生之道（更新版）");

    // Publish article
    let publish_dto = json!({
        "publish_channels": ["官网新闻", "手机端", "健康科普"]
    });

    let (status, body) = app
        .post_with_auth(
            &format!("/api/v1/content/articles/{}/publish", article_id),
            publish_dto,
            &doctor_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["status"], "published");

    // Unpublish article
    let (status, body) = app
        .post_with_auth(
            &format!("/api/v1/content/articles/{}/unpublish", article_id),
            json!({}),
            &doctor_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["status"], "offline");

    // Delete article
    let (status, _body) = app
        .delete_with_auth(
            &format!("/api/v1/content/articles/{}", article_id),
            &doctor_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Verify article is deleted
    let (status, _) = app
        .get(&format!("/api/v1/content/articles/{}", article_id))
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_video_crud() {
    let mut app = TestApp::new().await;

    // Create doctor
    let (doctor_id, doctor_account, doctor_password) = create_test_user(&app.pool, "doctor").await;
    let (_doctor_record_id, _) = create_test_doctor(&app.pool, doctor_id).await;
    let doctor_token = get_auth_token(&mut app, &doctor_account, &doctor_password).await;

    // Create video
    let create_dto = json!({
        "title": "中医推拿手法教学",
        "video_url": "https://example.com/videos/tuina.mp4",
        "duration": 1800,
        "file_size": 104857600,
        "description": "详细讲解中医推拿的基本手法",
        "category": "专家讲座",
        "tags": ["推拿", "中医", "教学"]
    });

    let (status, body) = app
        .post_with_auth("/api/v1/content/videos", create_dto, &doctor_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());
    let video_id = body["data"]["id"].as_str().unwrap();

    // List videos
    let (status, body) = app.get("/api/v1/content/videos").await;
    assert_eq!(status, StatusCode::OK);
    let videos = body["data"].as_array().unwrap();
    assert!(!videos.is_empty());

    // Get video by ID
    let (status, body) = app
        .get(&format!("/api/v1/content/videos/{}", video_id))
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["title"], "中医推拿手法教学");
    assert_eq!(body["data"]["duration"], 1800);

    // Update video
    let update_dto = json!({
        "title": "中医推拿手法教学（高清版）",
        "description": "更详细的推拿手法讲解"
    });

    let (status, body) = app
        .put_with_auth(
            &format!("/api/v1/content/videos/{}", video_id),
            update_dto,
            &doctor_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["title"], "中医推拿手法教学（高清版）");

    // Publish video
    let publish_dto = json!({
        "publish_channels": ["官网", "手机端"]
    });

    let (status, body) = app
        .post_with_auth(
            &format!("/api/v1/content/videos/{}/publish", video_id),
            publish_dto,
            &doctor_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["status"], "published");

    // Delete video
    let (status, _body) = app
        .delete_with_auth(
            &format!("/api/v1/content/videos/{}", video_id),
            &doctor_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_content_permissions() {
    let mut app = TestApp::new().await;

    // Create users
    let (_admin_id, admin_account, admin_password) = create_test_user(&app.pool, "admin").await;
    let (doctor1_id, doctor1_account, doctor1_password) =
        create_test_user(&app.pool, "doctor").await;
    let (doctor2_id, doctor2_account, doctor2_password) =
        create_test_user(&app.pool, "doctor").await;
    let (_patient_id, patient_account, patient_password) =
        create_test_user(&app.pool, "patient").await;

    let (_doctor1_record_id, _) = create_test_doctor(&app.pool, doctor1_id).await;
    let (_doctor2_record_id, _) = create_test_doctor(&app.pool, doctor2_id).await;

    let admin_token = get_auth_token(&mut app, &admin_account, &admin_password).await;
    let doctor1_token = get_auth_token(&mut app, &doctor1_account, &doctor1_password).await;
    let doctor2_token = get_auth_token(&mut app, &doctor2_account, &doctor2_password).await;
    let patient_token = get_auth_token(&mut app, &patient_account, &patient_password).await;

    // Patient cannot create article
    let create_dto = json!({
        "title": "测试文章",
        "content": "测试内容",
        "category": "健康科普"
    });

    let (status, _) = app
        .post_with_auth(
            "/api/v1/content/articles",
            create_dto.clone(),
            &patient_token,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Doctor1 creates article
    let (status, body) = app
        .post_with_auth("/api/v1/content/articles", create_dto, &doctor1_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    let article_id = body["data"]["id"].as_str().unwrap();

    // Doctor2 cannot update doctor1's article
    let update_dto = json!({
        "title": "尝试修改别人的文章"
    });

    let (status, _) = app
        .put_with_auth(
            &format!("/api/v1/content/articles/{}", article_id),
            update_dto.clone(),
            &doctor2_token,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Admin can update any article
    let (status, body) = app
        .put_with_auth(
            &format!("/api/v1/content/articles/{}", article_id),
            update_dto,
            &admin_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["title"], "尝试修改别人的文章");

    // Doctor2 cannot delete doctor1's article
    let (status, _) = app
        .delete_with_auth(
            &format!("/api/v1/content/articles/{}", article_id),
            &doctor2_token,
        )
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Admin can delete any article
    let (status, _) = app
        .delete_with_auth(
            &format!("/api/v1/content/articles/{}", article_id),
            &admin_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_content_search_and_filter() {
    let mut app = TestApp::new().await;

    // Create doctor
    let (doctor_id, doctor_account, doctor_password) = create_test_user(&app.pool, "doctor").await;
    let (_doctor_record_id, _) = create_test_doctor(&app.pool, doctor_id).await;
    let doctor_token = get_auth_token(&mut app, &doctor_account, &doctor_password).await;

    // Create multiple articles with different categories and statuses
    let articles = vec![
        json!({
            "title": "春季养生指南",
            "content": "春季养生的重要性...",
            "category": "健康科普",
            "summary": "春季如何调理身体"
        }),
        json!({
            "title": "夏季防暑妙招",
            "content": "夏季防暑的方法...",
            "category": "健康科普",
            "summary": "夏季防暑降温技巧"
        }),
        json!({
            "title": "中医诊疗新进展",
            "content": "最新的中医诊疗技术...",
            "category": "官网新闻",
            "summary": "介绍中医最新发展"
        }),
    ];

    let mut article_ids = Vec::new();
    for article in articles {
        let (status, body) = app
            .post_with_auth("/api/v1/content/articles", article, &doctor_token)
            .await;
        assert_eq!(status, StatusCode::OK);
        article_ids.push(body["data"]["id"].as_str().unwrap().to_string());
    }

    // Publish first article
    let (status, _) = app
        .post_with_auth(
            &format!("/api/v1/content/articles/{}/publish", article_ids[0]),
            json!({"publish_channels": ["官网"]}),
            &doctor_token,
        )
        .await;
    assert_eq!(status, StatusCode::OK);

    // Search by keyword
    let (status, body) = app.get("/api/v1/content/articles?search=养生").await;
    assert_eq!(status, StatusCode::OK);
    let results = body["data"].as_array().unwrap();
    assert_eq!(results.len(), 1); // Should find "春季养生指南"

    // Filter by category
    let (status, body) = app.get("/api/v1/content/articles?category=健康科普").await;
    assert_eq!(status, StatusCode::OK);
    let results = body["data"].as_array().unwrap();
    assert_eq!(results.len(), 2);

    // Filter by status
    let (status, body) = app.get("/api/v1/content/articles?status=published").await;
    assert_eq!(status, StatusCode::OK);
    let results = body["data"].as_array().unwrap();
    assert_eq!(results.len(), 1);
}

#[tokio::test]
async fn test_category_management() {
    let mut app = TestApp::new().await;

    // Create admin
    let (_admin_id, admin_account, admin_password) = create_test_user(&app.pool, "admin").await;
    let admin_token = get_auth_token(&mut app, &admin_account, &admin_password).await;

    // List categories
    let (status, body) = app.get("/api/v1/content/categories").await;
    assert_eq!(status, StatusCode::OK);
    let categories = body["data"].as_array().unwrap();
    assert!(!categories.is_empty()); // Should have default categories

    // Create new category with unique name
    let unique_name = format!(
        "测试分类_{}",
        uuid::Uuid::new_v4().to_string().split('-').next().unwrap()
    );
    let create_dto = json!({
        "name": &unique_name,
        "type": "article",
        "sort_order": 10
    });

    let (status, body) = app
        .post_with_auth("/api/v1/content/categories", create_dto, &admin_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["name"], unique_name);

    // Try to create duplicate category
    let duplicate_dto = json!({
        "name": &unique_name,
        "type": "article"
    });

    let (status, _) = app
        .post_with_auth("/api/v1/content/categories", duplicate_dto, &admin_token)
        .await;
    assert_eq!(status, StatusCode::CONFLICT);

    // Filter categories by type
    let (status, body) = app
        .get("/api/v1/content/categories?content_type=video")
        .await;
    assert_eq!(status, StatusCode::OK);
    let video_categories = body["data"].as_array().unwrap();
    assert!(video_categories
        .iter()
        .all(|c| c["type"] == "video" || c["type"] == "both"));
}

#[tokio::test]
async fn test_view_count_increment() {
    let mut app = TestApp::new().await;

    // Create doctor and article
    let (doctor_id, doctor_account, doctor_password) = create_test_user(&app.pool, "doctor").await;
    let (_doctor_record_id, _) = create_test_doctor(&app.pool, doctor_id).await;
    let doctor_token = get_auth_token(&mut app, &doctor_account, &doctor_password).await;

    let create_dto = json!({
        "title": "测试浏览量",
        "content": "测试内容",
        "category": "健康科普"
    });

    let (status, body) = app
        .post_with_auth("/api/v1/content/articles", create_dto, &doctor_token)
        .await;
    assert_eq!(status, StatusCode::OK);
    let article_id = body["data"]["id"].as_str().unwrap();

    // Initial view count should be 0
    assert_eq!(body["data"]["view_count"], 0);

    // View article multiple times
    for i in 1..=3 {
        let (status, body) = app
            .get(&format!("/api/v1/content/articles/{}", article_id))
            .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["data"]["view_count"], i);
    }
}
