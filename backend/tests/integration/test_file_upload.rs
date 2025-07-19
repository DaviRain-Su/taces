use crate::common::{create_test_app, create_test_user, TestApp};
use backend::models::file_upload::*;
use backend::models::UserRole;
use chrono::Utc;
use serial_test::serial;

#[tokio::test]
#[serial]
async fn test_create_upload_url() {
    let app = TestApp::new().await;
    
    // Create user
    let user = create_test_user(&app, "user@test.com", UserRole::Patient).await;
    let token = app.login_user(&user.email, "password123").await;
    
    // Request upload URL
    let create_dto = serde_json::json!({
        "file_name": "test-document.pdf",
        "file_type": "document",
        "file_size": 1048576,
        "mime_type": "application/pdf",
        "related_type": "prescription",
        "related_id": uuid::Uuid::new_v4()
    });
    
    let response = app.client
        .post(&format!("{}/api/v1/files/upload", app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&create_dto)
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 201);
    
    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["success"].as_bool().unwrap());
    assert!(body["data"]["upload_id"].as_str().is_some());
    assert!(body["data"]["upload_url"].as_str().is_some());
    assert_eq!(body["data"]["upload_method"].as_str().unwrap(), "PUT");
    assert!(body["data"]["upload_headers"].is_object());
    assert!(body["data"]["expires_at"].as_str().is_some());
}

#[tokio::test]
#[serial]
async fn test_complete_upload() {
    let app = TestApp::new().await;
    
    // Create user
    let user = create_test_user(&app, "user@test.com", UserRole::Patient).await;
    let token = app.login_user(&user.email, "password123").await;
    
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
        .bind(&user.id)
        .bind(&file_name)
        .bind(&file_path)
        .bind(524288i64)
        .bind(&Utc::now())
        .execute(&app.db)
        .await
        .unwrap();
    
    // Complete upload
    let complete_dto = serde_json::json!({
        "file_path": file_path,
        "file_url": format!("https://cdn.example.com/{}", file_path),
        "bucket_name": "tcm-files",
        "object_key": file_path,
        "etag": "5d41402abc4b2a76b9719d911017c592",
        "width": 800,
        "height": 600,
        "thumbnail_url": format!("https://cdn.example.com/{}_thumb.jpg", upload_id)
    });
    
    let response = app.client
        .put(&format!("{}/api/v1/files/upload/{}/complete", app.address, upload_id))
        .header("Authorization", format!("Bearer {}", token))
        .json(&complete_dto)
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["data"]["status"].as_str().unwrap(), "completed");
    assert_eq!(body["data"]["width"].as_i64().unwrap(), 800);
    assert_eq!(body["data"]["height"].as_i64().unwrap(), 600);
    assert!(body["data"]["thumbnail_url"].as_str().is_some());
}

#[tokio::test]
#[serial]
async fn test_get_file() {
    let app = TestApp::new().await;
    
    // Create user
    let user = create_test_user(&app, "user@test.com", UserRole::Patient).await;
    let token = app.login_user(&user.email, "password123").await;
    
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
        .bind(&user.id)
        .bind(&file_url)
        .bind(&Utc::now())
        .execute(&app.db)
        .await
        .unwrap();
    
    // Get file
    let response = app.client
        .get(&format!("{}/api/v1/files/{}", app.address, file_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["data"]["id"].as_str().unwrap(), file_id.to_string());
    assert_eq!(body["data"]["file_type"].as_str().unwrap(), "image");
    assert_eq!(body["data"]["file_url"].as_str().unwrap(), file_url);
}

#[tokio::test]
#[serial]
async fn test_list_files() {
    let app = TestApp::new().await;
    
    // Create user
    let user = create_test_user(&app, "user@test.com", UserRole::Patient).await;
    let token = app.login_user(&user.email, "password123").await;
    
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
            .bind(&user.id)
            .bind(&file_type)
            .bind(&format!("file{}.ext", i))
            .bind(&format!("{}/2024/01/{}.ext", file_type, file_id))
            .bind(&format!("https://cdn.example.com/{}/2024/01/{}.ext", file_type, file_id))
            .bind((i + 1) * 1048576i64)
            .bind(&now)
            .execute(&app.db)
            .await
            .unwrap();
    }
    
    // List files
    let response = app.client
        .get(&format!("{}/api/v1/files?page=1&page_size=10", app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["success"].as_bool().unwrap());
    
    let files = body["data"]["files"].as_array().unwrap();
    assert_eq!(files.len(), 3);
    assert_eq!(body["data"]["total"].as_i64().unwrap(), 3);
    assert_eq!(body["data"]["total_size"].as_i64().unwrap(), 6291456); // 6MB total
}

#[tokio::test]
#[serial]
async fn test_delete_file() {
    let app = TestApp::new().await;
    
    // Create user
    let user = create_test_user(&app, "user@test.com", UserRole::Patient).await;
    let token = app.login_user(&user.email, "password123").await;
    
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
        .bind(&user.id)
        .bind(&Utc::now())
        .execute(&app.db)
        .await
        .unwrap();
    
    // Delete file
    let response = app.client
        .delete(&format!("{}/api/v1/files/{}", app.address, file_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    // Verify file marked as deleted
    let check_query = r#"
        SELECT status, deleted_at FROM file_uploads WHERE id = ?
    "#;
    
    let row = sqlx::query(check_query)
        .bind(&file_id)
        .fetch_one(&app.db)
        .await
        .unwrap();
    
    let status: String = row.try_get("status").unwrap();
    assert_eq!(status, "deleted");
    
    let deleted_at: Option<chrono::DateTime<chrono::Utc>> = row.try_get("deleted_at").unwrap();
    assert!(deleted_at.is_some());
}

#[tokio::test]
#[serial]
async fn test_file_stats() {
    let app = TestApp::new().await;
    
    // Create user
    let user = create_test_user(&app, "user@test.com", UserRole::Patient).await;
    let token = app.login_user(&user.email, "password123").await;
    
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
                .bind(&user.id)
                .bind(&file_type)
                .bind(&format!("{}_{}.ext", file_type, i))
                .bind(&format!("{}/2024/01/{}.ext", file_type, file_id))
                .bind(&format!("https://cdn.example.com/{}/2024/01/{}.ext", file_type, file_id))
                .bind(&size)
                .bind(&Utc::now())
                .execute(&app.db)
                .await
                .unwrap();
        }
    }
    
    // Get stats
    let response = app.client
        .get(&format!("{}/api/v1/files/stats", app.address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["success"].as_bool().unwrap());
    
    let stats = &body["data"];
    assert_eq!(stats["total_files"].as_i64().unwrap(), 6);
    assert_eq!(stats["total_size"].as_i64().unwrap(), 14155776); // ~13.5MB
    
    let by_type = stats["by_type"].as_array().unwrap();
    assert!(!by_type.is_empty());
}

#[tokio::test]
#[serial]
async fn test_file_type_validation() {
    let app = TestApp::new().await;
    
    // Create user
    let user = create_test_user(&app, "user@test.com", UserRole::Patient).await;
    let token = app.login_user(&user.email, "password123").await;
    
    // Try to upload file exceeding size limit
    let create_dto = serde_json::json!({
        "file_name": "huge-image.jpg",
        "file_type": "image",
        "file_size": 20971520, // 20MB, exceeds 10MB limit
        "mime_type": "image/jpeg"
    });
    
    let response = app.client
        .post(&format!("{}/api/v1/files/upload", app.address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&create_dto)
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 400);
    
    let body: serde_json::Value = response.json().await.unwrap();
    assert!(!body["success"].as_bool().unwrap());
    assert!(body["message"].as_str().unwrap().contains("文件大小超过限制"));
}

#[tokio::test]
#[serial]
async fn test_authorization() {
    let app = TestApp::new().await;
    
    // Create two users
    let user1 = create_test_user(&app, "user1@test.com", UserRole::Patient).await;
    let user2 = create_test_user(&app, "user2@test.com", UserRole::Patient).await;
    let user2_token = app.login_user(&user2.email, "password123").await;
    
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
        .bind(&user1.id)
        .bind(&Utc::now())
        .execute(&app.db)
        .await
        .unwrap();
    
    // User2 tries to access user1's private file
    let response = app.client
        .get(&format!("{}/api/v1/files/{}", app.address, file_id))
        .header("Authorization", format!("Bearer {}", user2_token))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 403);
    
    // User2 tries to delete user1's file
    let response = app.client
        .delete(&format!("{}/api/v1/files/{}", app.address, file_id))
        .header("Authorization", format!("Bearer {}", user2_token))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 403);
}

#[tokio::test]
#[serial]
async fn test_admin_access() {
    let app = TestApp::new().await;
    
    // Create admin
    let admin = create_test_user(&app, "admin@test.com", UserRole::Admin).await;
    let admin_token = app.login_user(&admin.email, "password123").await;
    
    // Test config endpoints
    let endpoints = vec![
        ("/api/v1/files/config/upload", "GET"),
        ("/api/v1/files/config/image", "GET"),
        ("/api/v1/files/config/video", "GET"),
    ];
    
    for (endpoint, method) in endpoints {
        let response = match method {
            "GET" => {
                app.client
                    .get(&format!("{}{}", app.address, endpoint))
                    .header("Authorization", format!("Bearer {}", admin_token))
                    .send()
                    .await
                    .unwrap()
            }
            _ => unreachable!(),
        };
        
        assert_eq!(response.status(), 200);
    }
    
    // Test config update
    let update_dto = serde_json::json!({
        "config_value": "20971520",
        "description": "Maximum image file size (20MB)"
    });
    
    let response = app.client
        .put(&format!("{}/api/v1/files/config/file_upload/max_image_size", app.address))
        .header("Authorization", format!("Bearer {}", admin_token))
        .json(&update_dto)
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
}

#[tokio::test]
#[serial]
async fn test_public_file_access() {
    let app = TestApp::new().await;
    
    // Create user1 with public file
    let user1 = create_test_user(&app, "user1@test.com", UserRole::Patient).await;
    let user2 = create_test_user(&app, "user2@test.com", UserRole::Patient).await;
    let user2_token = app.login_user(&user2.email, "password123").await;
    
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
        .bind(&user1.id)
        .bind(&Utc::now())
        .execute(&app.db)
        .await
        .unwrap();
    
    // User2 can access public file
    let response = app.client
        .get(&format!("{}/api/v1/files/{}", app.address, file_id))
        .header("Authorization", format!("Bearer {}", user2_token))
        .send()
        .await
        .unwrap();
    
    assert_eq!(response.status(), 200);
    
    let body: serde_json::Value = response.json().await.unwrap();
    assert!(body["success"].as_bool().unwrap());
    assert!(body["data"]["is_public"].as_bool().unwrap());
}