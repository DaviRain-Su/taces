use backend::{
    config::storage, models::file_upload::FileType,
    services::file_storage_service::FileStorageService,
};
use std::env;

#[tokio::test]
async fn test_generate_file_path() {
    let path = FileStorageService::generate_file_path(&FileType::Image, "test.jpg");
    assert!(path.starts_with("images/"));
    assert!(path.ends_with(".jpg"));

    let path = FileStorageService::generate_file_path(&FileType::Document, "report.pdf");
    assert!(path.starts_with("documents/"));
    assert!(path.ends_with(".pdf"));

    let path = FileStorageService::generate_file_path(&FileType::Video, "consultation.mp4");
    assert!(path.starts_with("videos/"));
    assert!(path.ends_with(".mp4"));
}

#[tokio::test]
async fn test_validate_file_size() {
    // Image - max 10MB
    assert!(FileStorageService::validate_file_size(5 * 1024 * 1024, &FileType::Image).is_ok());
    assert!(FileStorageService::validate_file_size(15 * 1024 * 1024, &FileType::Image).is_err());

    // Document - max 50MB
    assert!(FileStorageService::validate_file_size(30 * 1024 * 1024, &FileType::Document).is_ok());
    assert!(FileStorageService::validate_file_size(60 * 1024 * 1024, &FileType::Document).is_err());

    // Video - max 500MB
    assert!(FileStorageService::validate_file_size(400 * 1024 * 1024, &FileType::Video).is_ok());
    assert!(FileStorageService::validate_file_size(600 * 1024 * 1024, &FileType::Video).is_err());
}

#[tokio::test]
async fn test_validate_file_extension() {
    // Image extensions
    assert!(FileStorageService::validate_file_extension("photo.jpg", &FileType::Image).is_ok());
    assert!(FileStorageService::validate_file_extension("photo.png", &FileType::Image).is_ok());
    assert!(FileStorageService::validate_file_extension("photo.gif", &FileType::Image).is_ok());
    assert!(FileStorageService::validate_file_extension("photo.exe", &FileType::Image).is_err());

    // Document extensions
    assert!(FileStorageService::validate_file_extension("report.pdf", &FileType::Document).is_ok());
    assert!(FileStorageService::validate_file_extension("data.xlsx", &FileType::Document).is_ok());
    assert!(FileStorageService::validate_file_extension("script.js", &FileType::Document).is_err());

    // Video extensions
    assert!(FileStorageService::validate_file_extension("video.mp4", &FileType::Video).is_ok());
    assert!(FileStorageService::validate_file_extension("video.avi", &FileType::Video).is_ok());
    assert!(FileStorageService::validate_file_extension("video.txt", &FileType::Video).is_err());

    // Other type allows any extension
    assert!(FileStorageService::validate_file_extension("anything.xyz", &FileType::Other).is_ok());
}

#[tokio::test]
async fn test_get_content_type() {
    // Images
    assert_eq!(
        FileStorageService::get_content_type("photo.jpg"),
        "image/jpeg"
    );
    assert_eq!(
        FileStorageService::get_content_type("photo.png"),
        "image/png"
    );
    assert_eq!(
        FileStorageService::get_content_type("photo.gif"),
        "image/gif"
    );

    // Documents
    assert_eq!(
        FileStorageService::get_content_type("report.pdf"),
        "application/pdf"
    );
    assert_eq!(
        FileStorageService::get_content_type("data.xlsx"),
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
    );

    // Videos
    assert_eq!(
        FileStorageService::get_content_type("video.mp4"),
        "video/mp4"
    );
    assert_eq!(
        FileStorageService::get_content_type("video.avi"),
        "video/x-msvideo"
    );

    // Audio
    assert_eq!(
        FileStorageService::get_content_type("audio.mp3"),
        "audio/mpeg"
    );
    assert_eq!(
        FileStorageService::get_content_type("audio.wav"),
        "audio/wav"
    );

    // Unknown
    assert_eq!(
        FileStorageService::get_content_type("file.xyz"),
        "application/octet-stream"
    );
}

#[tokio::test]
async fn test_upload_to_local() {
    let file_path = "test/local_upload_test.txt";
    let data = b"Test file content".to_vec();

    let result = FileStorageService::upload_to_local(file_path, data.clone()).await;
    assert!(result.is_ok());

    let url = result.unwrap();
    assert_eq!(url, format!("/uploads/{}", file_path));

    // Verify file exists
    let full_path = format!("uploads/{}", file_path);
    assert!(tokio::fs::metadata(&full_path).await.is_ok());

    // Clean up
    let _ = tokio::fs::remove_file(&full_path).await;
}

#[tokio::test]
async fn test_delete_from_local() {
    let file_path = "test/local_delete_test.txt";
    let data = b"Test file content".to_vec();

    // First upload
    let _ = FileStorageService::upload_to_local(file_path, data).await;

    // Then delete
    let result = FileStorageService::delete_from_local(file_path).await;
    assert!(result.is_ok());

    // Verify file doesn't exist
    let full_path = format!("uploads/{}", file_path);
    assert!(tokio::fs::metadata(&full_path).await.is_err());
}

#[tokio::test]
async fn test_s3_client_creation() {
    // Temporarily set environment variables
    env::set_var("STORAGE_ACCESS_KEY_ID", "test_key");
    env::set_var("STORAGE_SECRET_ACCESS_KEY", "test_secret");

    let client = storage::create_s3_client_optional().await;

    // Client should be created if credentials are provided
    assert!(client.is_some());

    // Clean up
    env::remove_var("STORAGE_ACCESS_KEY_ID");
    env::remove_var("STORAGE_SECRET_ACCESS_KEY");
}

#[tokio::test]
#[ignore] // Only run when S3 is configured
async fn test_s3_upload_and_delete() {
    let s3_client = storage::create_s3_client().await.unwrap();

    let file_path = "test/s3_test.txt";
    let data = b"Test S3 content".to_vec();
    let content_type = "text/plain";

    // Upload
    let url = FileStorageService::upload_to_cloud(&s3_client, file_path, data, content_type)
        .await
        .unwrap();

    assert!(!url.is_empty());

    // Delete
    let result = FileStorageService::delete_from_cloud(&s3_client, file_path).await;
    assert!(result.is_ok());
}

#[tokio::test]
#[ignore] // Only run when S3 is configured
async fn test_presigned_urls() {
    let s3_client = storage::create_s3_client().await.unwrap();

    let file_path = "test/presigned_test.jpg";
    let content_type = "image/jpeg";

    // Generate upload URL
    let upload_url = FileStorageService::generate_presigned_upload_url(
        &s3_client,
        file_path,
        content_type,
        300, // 5 minutes
    )
    .await
    .unwrap();

    assert!(upload_url.contains(file_path));
    assert!(upload_url.contains("X-Amz-Signature"));

    // Generate download URL
    let download_url =
        FileStorageService::generate_presigned_download_url(&s3_client, file_path, 300)
            .await
            .unwrap();

    assert!(download_url.contains(file_path));
    assert!(download_url.contains("X-Amz-Signature"));
}
