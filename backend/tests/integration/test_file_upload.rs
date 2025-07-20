use crate::common::TestApp;
use axum::http::StatusCode;
use backend::utils::test_helpers::create_test_user;
use chrono::Utc;
//use serial_test::serial;
use serde_json::json;

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
async fn test_create_upload_url() {
    let mut app = TestApp::new().await;

    // Create user
    let (user_id, email, password) = create_test_user(&app.pool, "patient").await;
    let token = get_auth_token(&mut app, &email, &password).await;

    // Request upload URL
    let create_dto = json!({
        "file_name": "test-document.pdf",
        "file_type": "document",
        "file_size": 1048576,
        "mime_type": "application/pdf",
        "related_type": "prescription",
        "related_id": uuid::Uuid::new_v4()
    });

    let (status, body) = app
        .post_with_auth("/api/v1/files/upload", create_dto, &token)
        .await;

    assert_eq!(status, StatusCode::CREATED);
    assert!(body["success"].as_bool().unwrap());
    assert!(body["data"]["upload_id"].as_str().is_some());
    assert!(body["data"]["upload_url"].as_str().is_some());
    assert_eq!(body["data"]["upload_method"].as_str().unwrap(), "PUT");
    assert!(body["data"]["upload_headers"].is_object());
    assert!(body["data"]["expires_at"].as_str().is_some());
}

#[tokio::test]
//#[serial]
async fn test_complete_upload() {
    let mut app = TestApp::new().await;

    // Create user
    let (user_id, email, password) = create_test_user(&app.pool, "patient").await;
    let token = get_auth_token(&mut app, &email, &password).await;

    // Create upload record
    let upload_id = uuid::Uuid::new_v4();
    let file_name = "test-image.jpg";
    let file_path = format!("image/2024/01/{}_1705766400.jpg", upload_id);

    let query = r#"
        INSERT INTO file_uploads (
            id, user_id, file_type, file_name, file_path,
            file_url, file_size, mime_type, status, uploaded_at
        ) VALUES (?, ?, 'image', ?, ?, '', ?, 'image/jpeg', 'uploading', ?)
    "#;

    sqlx::query(query)
        .bind(&upload_id)
        .bind(&user_id)
        .bind(&file_name)
        .bind(&file_path)
        .bind(524288i64)
        .bind(&Utc::now())
        .execute(&app.pool)
        .await
        .unwrap();

    // Complete upload
    let complete_dto = json!({
        "file_path": file_path,
        "file_url": format!("https://cdn.example.com/{}", file_path),
        "bucket_name": "tcm-files",
        "object_key": file_path,
        "etag": "5d41402abc4b2a76b9719d911017c592",
        "width": 800,
        "height": 600,
        "thumbnail_url": format!("https://cdn.example.com/{}_thumb.jpg", upload_id)
    });

    let (status, body) = app
        .put_with_auth(
            &format!("/api/v1/files/upload/{}/complete", upload_id),
            complete_dto,
            &token,
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["data"]["status"].as_str().unwrap(), "completed");
    assert_eq!(body["data"]["width"].as_i64().unwrap(), 800);
    assert_eq!(body["data"]["height"].as_i64().unwrap(), 600);
    assert!(body["data"]["thumbnail_url"].as_str().is_some());
}

#[tokio::test]
//#[serial]
async fn test_get_file() {
    let mut app = TestApp::new().await;

    // Create user
    let (user_id, email, password) = create_test_user(&app.pool, "patient").await;
    let token = get_auth_token(&mut app, &email, &password).await;

    // Create file record
    let file_id = uuid::Uuid::new_v4();
    let file_url = format!("https://cdn.example.com/image/2024/01/{}.jpg", file_id);

    let query = r#"
        INSERT INTO file_uploads (
            id, user_id, file_type, file_name, file_path, file_url,
            file_size, mime_type, is_public, status, uploaded_at
        ) VALUES (?, ?, 'image', 'test.jpg', 'image/2024/01/test.jpg', ?,
            524288, 'image/jpeg', false, 'completed', ?)
    "#;

    sqlx::query(query)
        .bind(&file_id)
        .bind(&user_id)
        .bind(&file_url)
        .bind(&Utc::now())
        .execute(&app.pool)
        .await
        .unwrap();

    // Get file
    let (status, body) = app
        .get_with_auth(&format!("/api/v1/files/{}", file_id), &token)
        .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["data"]["id"].as_str().unwrap(), file_id.to_string());
    assert_eq!(body["data"]["file_type"].as_str().unwrap(), "image");
    assert_eq!(body["data"]["file_url"].as_str().unwrap(), file_url);
}

#[tokio::test]
//#[serial]
async fn test_list_files() {
    let mut app = TestApp::new().await;

    // Create user
    let (user_id, email, password) = create_test_user(&app.pool, "patient").await;
    let token = get_auth_token(&mut app, &email, &password).await;

    // Create multiple files
    let now = Utc::now();
    for i in 0..3 {
        let file_id = uuid::Uuid::new_v4();
        let file_type = match i {
            0 => "image",
            1 => "document",
            _ => "video",
        };

        let query = r#"
            INSERT INTO file_uploads (
                id, user_id, file_type, file_name, file_path, file_url,
                file_size, status, uploaded_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, 'completed', ?)
        "#;

        sqlx::query(query)
            .bind(&file_id)
            .bind(&user_id)
            .bind(&file_type)
            .bind(&format!("file{}.ext", i))
            .bind(&format!("{}/2024/01/{}.ext", file_type, file_id))
            .bind(&format!(
                "https://cdn.example.com/{}/2024/01/{}.ext",
                file_type, file_id
            ))
            .bind((i + 1) * 1048576i64)
            .bind(&now)
            .execute(&app.pool)
            .await
            .unwrap();
    }

    // List files
    let (status, body) = app
        .get_with_auth("/api/v1/files?page=1&page_size=10", &token)
        .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());

    let files = body["data"]["files"].as_array().unwrap();
    assert_eq!(files.len(), 3);
    assert_eq!(body["data"]["total"].as_i64().unwrap(), 3);
    assert_eq!(body["data"]["total_size"].as_i64().unwrap(), 6291456); // 6MB total
}

#[tokio::test]
//#[serial]
async fn test_delete_file() {
    let mut app = TestApp::new().await;

    // Create user
    let (user_id, email, password) = create_test_user(&app.pool, "patient").await;
    let token = get_auth_token(&mut app, &email, &password).await;

    // Create file
    let file_id = uuid::Uuid::new_v4();

    let query = r#"
        INSERT INTO file_uploads (
            id, user_id, file_type, file_name, file_path, file_url,
            file_size, status, uploaded_at
        ) VALUES (?, ?, 'document', 'test.pdf', 'document/test.pdf', 'https://cdn.example.com/test.pdf',
            1048576, 'completed', ?)
    "#;

    sqlx::query(query)
        .bind(&file_id)
        .bind(&user_id)
        .bind(&Utc::now())
        .execute(&app.pool)
        .await
        .unwrap();

    // Delete file
    let (status, _) = app
        .delete_with_auth(&format!("/api/v1/files/{}", file_id), &token)
        .await;

    assert_eq!(status, StatusCode::OK);

    // Verify file marked as deleted
    let check_query = r#"
        SELECT status, deleted_at FROM file_uploads WHERE id = ?
    "#;

    #[derive(sqlx::FromRow)]
    struct FileStatus {
        status: String,
        deleted_at: Option<chrono::DateTime<chrono::Utc>>,
    }

    let file_status: FileStatus = sqlx::query_as(check_query)
        .bind(&file_id)
        .fetch_one(&app.pool)
        .await
        .unwrap();

    assert_eq!(file_status.status, "deleted");
    assert!(file_status.deleted_at.is_some());
}

#[tokio::test]
//#[serial]
async fn test_file_stats() {
    let mut app = TestApp::new().await;

    // Create user
    let (user_id, email, password) = create_test_user(&app.pool, "patient").await;
    let token = get_auth_token(&mut app, &email, &password).await;

    // Create files of different types
    let file_configs = vec![
        ("image", 2, 1048576i64),
        ("video", 1, 10485760i64),
        ("document", 3, 524288i64),
    ];

    for (file_type, count, size) in file_configs {
        for i in 0..count {
            let file_id = uuid::Uuid::new_v4();

            let query = r#"
                INSERT INTO file_uploads (
                    id, user_id, file_type, file_name, file_path, file_url,
                    file_size, status, uploaded_at
                ) VALUES (?, ?, ?, ?, ?, ?, ?, 'completed', ?)
            "#;

            sqlx::query(query)
                .bind(&file_id)
                .bind(&user_id)
                .bind(&file_type)
                .bind(&format!("{}_{}.ext", file_type, i))
                .bind(&format!("{}/2024/01/{}.ext", file_type, file_id))
                .bind(&format!(
                    "https://cdn.example.com/{}/2024/01/{}.ext",
                    file_type, file_id
                ))
                .bind(&size)
                .bind(&Utc::now())
                .execute(&app.pool)
                .await
                .unwrap();
        }
    }

    // Get stats
    let (status, body) = app.get_with_auth("/api/v1/files/stats", &token).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());

    let stats = &body["data"];
    assert_eq!(stats["total_files"].as_i64().unwrap(), 6);
    assert_eq!(stats["total_size"].as_i64().unwrap(), 14155776); // ~13.5MB

    let by_type = stats["by_type"].as_array().unwrap();
    assert!(!by_type.is_empty());
}

#[tokio::test]
//#[serial]
async fn test_file_type_validation() {
    let mut app = TestApp::new().await;

    // Create user
    let (user_id, email, password) = create_test_user(&app.pool, "patient").await;
    let token = get_auth_token(&mut app, &email, &password).await;

    // Try to upload file exceeding size limit
    let create_dto = json!({
        "file_name": "huge-image.jpg",
        "file_type": "image",
        "file_size": 20971520, // 20MB, exceeds 10MB limit
        "mime_type": "image/jpeg"
    });

    let (status, body) = app
        .post_with_auth("/api/v1/files/upload", create_dto, &token)
        .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(!body["success"].as_bool().unwrap());
    assert!(body["message"]
        .as_str()
        .unwrap()
        .contains("文件大小超过限制"));
}

#[tokio::test]
//#[serial]
async fn test_authorization() {
    let mut app = TestApp::new().await;

    // Create two users
    let (user1_id, _, _) = create_test_user(&app.pool, "patient").await;
    let (user2_id, email2, password2) = create_test_user(&app.pool, "patient").await;
    let user2_token = get_auth_token(&mut app, &email2, &password2).await;

    // Create private file for user1
    let file_id = uuid::Uuid::new_v4();

    let query = r#"
        INSERT INTO file_uploads (
            id, user_id, file_type, file_name, file_path, file_url,
            file_size, is_public, status, uploaded_at
        ) VALUES (?, ?, 'document', 'private.pdf', 'document/private.pdf', 'https://cdn.example.com/private.pdf',
            1048576, false, 'completed', ?)
    "#;

    sqlx::query(query)
        .bind(&file_id)
        .bind(&user1_id)
        .bind(&Utc::now())
        .execute(&app.pool)
        .await
        .unwrap();

    // User2 tries to access user1's private file
    let (status, _) = app
        .get_with_auth(&format!("/api/v1/files/{}", file_id), &user2_token)
        .await;

    assert_eq!(status, StatusCode::FORBIDDEN);

    // User2 tries to delete user1's file
    let (status, _) = app
        .delete_with_auth(&format!("/api/v1/files/{}", file_id), &user2_token)
        .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
//#[serial]
async fn test_admin_access() {
    let mut app = TestApp::new().await;

    // Create admin
    let (admin_id, admin_email, admin_password) = create_test_user(&app.pool, "admin").await;
    let admin_token = get_auth_token(&mut app, &admin_email, &admin_password).await;

    // Test config endpoints
    let endpoints = vec![
        ("/api/v1/files/config/upload", "GET"),
        ("/api/v1/files/config/image", "GET"),
        ("/api/v1/files/config/video", "GET"),
    ];

    for (endpoint, _method) in endpoints {
        let (status, _) = app.get_with_auth(endpoint, &admin_token).await;
        assert_eq!(status, StatusCode::OK);
    }

    // Test config update
    let update_dto = json!({
        "config_value": "20971520",
        "description": "Maximum image file size (20MB)"
    });

    let (status, _) = app
        .put_with_auth(
            "/api/v1/files/config/file_upload/max_image_size",
            update_dto,
            &admin_token,
        )
        .await;

    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
//#[serial]
async fn test_public_file_access() {
    let mut app = TestApp::new().await;

    // Create user1 with public file
    let (user1_id, _, _) = create_test_user(&app.pool, "patient").await;
    let (user2_id, email2, password2) = create_test_user(&app.pool, "patient").await;
    let user2_token = get_auth_token(&mut app, &email2, &password2).await;

    // Create public file
    let file_id = uuid::Uuid::new_v4();

    let query = r#"
        INSERT INTO file_uploads (
            id, user_id, file_type, file_name, file_path, file_url,
            file_size, is_public, status, uploaded_at
        ) VALUES (?, ?, 'image', 'public.jpg', 'image/public.jpg', 'https://cdn.example.com/public.jpg',
            524288, true, 'completed', ?)
    "#;

    sqlx::query(query)
        .bind(&file_id)
        .bind(&user1_id)
        .bind(&Utc::now())
        .execute(&app.pool)
        .await
        .unwrap();

    // User2 can access public file
    let (status, body) = app
        .get_with_auth(&format!("/api/v1/files/{}", file_id), &user2_token)
        .await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());
    assert!(body["data"]["is_public"].as_bool().unwrap());
}
