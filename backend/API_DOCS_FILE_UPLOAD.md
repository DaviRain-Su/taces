# File Upload API Documentation

## Overview
The File Upload module provides a unified interface for uploading and managing files across the platform. It supports various file types (images, videos, documents, audio) and integrates with cloud storage services like Aliyun OSS and AWS S3.

## Table of Contents
- [Authentication](#authentication)
- [Upload Flow](#upload-flow)
- [File Management](#file-management)
- [Configuration Management](#configuration-management)
- [Statistics](#statistics)
- [Data Models](#data-models)
- [Error Codes](#error-codes)
- [Integration Guide](#integration-guide)

## Authentication
All endpoints require JWT authentication. Include the token in the Authorization header:
```
Authorization: Bearer <jwt_token>
```

## Upload Flow

The file upload process follows a two-step approach:
1. Request an upload URL from the server
2. Upload the file directly to the storage service
3. Confirm the upload completion

### Step 1: Create Upload URL
Requests a presigned upload URL from the server.

**Endpoint:** `POST /api/v1/files/upload`

**Access:** All authenticated users

**Request Body:**
```json
{
  "file_name": "report.pdf",
  "file_type": "document",
  "file_size": 2097152,
  "mime_type": "application/pdf",
  "related_type": "prescription",
  "related_id": "uuid"
}
```

**Response:**
```json
{
  "success": true,
  "message": "获取上传链接成功",
  "data": {
    "upload_id": "uuid",
    "upload_url": "https://oss.aliyuncs.com/bucket/document/2024/01/uuid_1705766400.pdf?signature=...",
    "upload_method": "PUT",
    "upload_headers": {
      "Content-Type": "application/pdf",
      "x-oss-object-acl": "private"
    },
    "expires_at": "2024-01-20T10:30:00Z"
  }
}
```

### Step 2: Upload File
Upload the file directly to the storage service using the provided URL.

```javascript
// Example using fetch API
const response = await fetch(uploadUrl, {
  method: uploadMethod,
  headers: uploadHeaders,
  body: fileData
});
```

### Step 3: Complete Upload
Confirm the upload completion and update file metadata.

**Endpoint:** `PUT /api/v1/files/upload/:id/complete`

**Access:** File owner only

**Request Body:**
```json
{
  "file_path": "document/2024/01/uuid_1705766400.pdf",
  "file_url": "https://cdn.example.com/document/2024/01/uuid_1705766400.pdf",
  "bucket_name": "tcm-files",
  "object_key": "document/2024/01/uuid_1705766400.pdf",
  "etag": "5d41402abc4b2a76b9719d911017c592",
  "width": null,
  "height": null,
  "thumbnail_url": null
}
```

**Response:**
```json
{
  "success": true,
  "message": "文件上传完成",
  "data": {
    "id": "uuid",
    "user_id": "uuid",
    "file_type": "document",
    "file_name": "report.pdf",
    "file_path": "document/2024/01/uuid_1705766400.pdf",
    "file_url": "https://cdn.example.com/document/2024/01/uuid_1705766400.pdf",
    "file_size": 2097152,
    "mime_type": "application/pdf",
    "status": "completed",
    "uploaded_at": "2024-01-20T10:00:00Z"
  }
}
```

## File Management

### Get File Details
Retrieves details of a specific file.

**Endpoint:** `GET /api/v1/files/:id`

**Access:** File owner, Admin, or public files

**Response:**
```json
{
  "success": true,
  "message": "获取文件信息成功",
  "data": {
    "id": "uuid",
    "user_id": "uuid",
    "file_type": "image",
    "file_name": "avatar.jpg",
    "file_url": "https://cdn.example.com/image/2024/01/uuid_1705766400.jpg",
    "file_size": 524288,
    "mime_type": "image/jpeg",
    "width": 800,
    "height": 600,
    "thumbnail_url": "https://cdn.example.com/thumbnail/2024/01/uuid_1705766400_thumb.jpg",
    "is_public": true,
    "status": "completed",
    "uploaded_at": "2024-01-20T09:00:00Z"
  }
}
```

### List Files
Lists files with filtering options.

**Endpoint:** `GET /api/v1/files`

**Access:** Own files (users), All files (Admin)

**Query Parameters:**
- `user_id` (optional): Filter by user (Admin only)
- `file_type` (optional): Filter by type (image, video, document, audio, other)
- `related_type` (optional): Filter by related entity type
- `related_id` (optional): Filter by related entity ID
- `status` (optional): Filter by status
- `is_public` (optional): Filter by public/private
- `page` (optional): Page number (default: 1)
- `page_size` (optional): Items per page (default: 20, max: 100)

**Response:**
```json
{
  "success": true,
  "message": "获取文件列表成功",
  "data": {
    "files": [
      {
        "id": "uuid",
        "file_type": "image",
        "file_name": "photo1.jpg",
        "file_url": "https://cdn.example.com/image/2024/01/uuid_1705766400.jpg",
        "file_size": 1048576,
        "status": "completed",
        "uploaded_at": "2024-01-20T08:00:00Z"
      }
    ],
    "total": 42,
    "page": 1,
    "page_size": 20,
    "total_size": 104857600
  }
}
```

### Delete File
Soft deletes a file (marks as deleted, actual deletion happens later).

**Endpoint:** `DELETE /api/v1/files/:id`

**Access:** File owner, Admin

**Response:**
```json
{
  "success": true,
  "message": "文件删除成功",
  "data": {}
}
```

### Get File Statistics
Retrieves file storage statistics.

**Endpoint:** `GET /api/v1/files/stats`

**Access:** Own stats (users), All stats (Admin)

**Response:**
```json
{
  "success": true,
  "message": "获取文件统计成功",
  "data": {
    "total_files": 150,
    "total_size": 1073741824,
    "by_type": [
      {
        "file_type": "image",
        "count": 80,
        "total_size": 419430400
      },
      {
        "file_type": "document",
        "count": 40,
        "total_size": 209715200
      },
      {
        "file_type": "video",
        "count": 30,
        "total_size": 444596224
      }
    ]
  }
}
```

## Configuration Management

### Get Upload Configuration
Retrieves general upload configuration.

**Endpoint:** `GET /api/v1/files/config/upload`

**Access:** Admin only

**Response:**
```json
{
  "success": true,
  "message": "获取上传配置成功",
  "data": {
    "max_file_size": 104857600,
    "allowed_mime_types": [
      "image/jpeg",
      "image/png",
      "image/gif",
      "video/mp4",
      "application/pdf"
    ],
    "storage_backend": "oss",
    "cdn_base_url": "https://cdn.example.com",
    "enable_compression": true,
    "enable_thumbnail": true
  }
}
```

### Get Image Configuration
Retrieves image-specific configuration.

**Endpoint:** `GET /api/v1/files/config/image`

**Access:** Admin only

**Response:**
```json
{
  "success": true,
  "message": "获取图片配置成功",
  "data": {
    "max_width": 4096,
    "max_height": 4096,
    "thumbnail_width": 200,
    "thumbnail_height": 200,
    "compression_quality": 85,
    "allowed_formats": ["jpg", "jpeg", "png", "gif", "webp"]
  }
}
```

### Get Video Configuration
Retrieves video-specific configuration.

**Endpoint:** `GET /api/v1/files/config/video`

**Access:** Admin only

**Response:**
```json
{
  "success": true,
  "message": "获取视频配置成功",
  "data": {
    "max_duration": 3600,
    "max_file_size": 104857600,
    "allowed_formats": ["mp4", "webm", "mov"],
    "enable_transcoding": false
  }
}
```

### Update System Configuration
Updates a specific configuration value.

**Endpoint:** `PUT /api/v1/files/config/:category/:key`

**Access:** Admin only

**Request Body:**
```json
{
  "config_value": "209715200",
  "description": "Maximum video file size (200MB)"
}
```

**Response:**
```json
{
  "success": true,
  "message": "配置更新成功",
  "data": {
    "id": "uuid",
    "category": "file_upload",
    "config_key": "max_video_size",
    "config_value": "209715200",
    "value_type": "number",
    "description": "Maximum video file size (200MB)",
    "updated_at": "2024-01-20T10:00:00Z"
  }
}
```

## Data Models

### FileType
- `image`: Image files (jpg, png, gif, webp)
- `video`: Video files (mp4, webm, mov)
- `document`: Documents (pdf, doc, docx)
- `audio`: Audio files (mp3, wav, ogg)
- `other`: Other file types

### UploadStatus
- `uploading`: File is being uploaded
- `completed`: Upload completed successfully
- `failed`: Upload failed
- `deleted`: File marked for deletion

### File Size Limits
- Images: 10MB
- Videos: 100MB
- Documents: 20MB
- Audio: 50MB
- Other: 10MB

## Error Codes
- `400`: Bad Request - Invalid parameters or file too large
- `401`: Unauthorized - Missing or invalid token
- `403`: Forbidden - Access denied to file
- `404`: Not Found - File not found
- `413`: Payload Too Large - File exceeds size limit
- `415`: Unsupported Media Type - File type not allowed
- `500`: Internal Server Error

## Integration Guide

### Client-Side Upload Example

```javascript
// Step 1: Request upload URL
async function requestUploadUrl(file) {
  const response = await fetch('/api/v1/files/upload', {
    method: 'POST',
    headers: {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      file_name: file.name,
      file_type: getFileType(file),
      file_size: file.size,
      mime_type: file.type
    })
  });
  
  if (!response.ok) {
    throw new Error('Failed to get upload URL');
  }
  
  return response.json();
}

// Step 2: Upload file to storage
async function uploadToStorage(file, uploadData) {
  const response = await fetch(uploadData.upload_url, {
    method: uploadData.upload_method,
    headers: uploadData.upload_headers,
    body: file
  });
  
  if (!response.ok) {
    throw new Error('Failed to upload file');
  }
  
  return {
    etag: response.headers.get('ETag'),
    url: uploadData.upload_url.split('?')[0]
  };
}

// Step 3: Complete upload
async function completeUpload(uploadId, uploadResult) {
  const response = await fetch(`/api/v1/files/upload/${uploadId}/complete`, {
    method: 'PUT',
    headers: {
      'Authorization': `Bearer ${token}`,
      'Content-Type': 'application/json'
    },
    body: JSON.stringify({
      file_url: uploadResult.url,
      etag: uploadResult.etag,
      // ... other metadata
    })
  });
  
  if (!response.ok) {
    throw new Error('Failed to complete upload');
  }
  
  return response.json();
}

// Main upload function
async function uploadFile(file) {
  try {
    // Step 1: Get upload URL
    const { data: uploadData } = await requestUploadUrl(file);
    
    // Step 2: Upload to storage
    const uploadResult = await uploadToStorage(file, uploadData);
    
    // Step 3: Complete upload
    const { data: fileData } = await completeUpload(
      uploadData.upload_id,
      uploadResult
    );
    
    return fileData;
  } catch (error) {
    console.error('Upload failed:', error);
    throw error;
  }
}
```

### Image Processing
For images, the system can automatically:
- Generate thumbnails
- Compress images while maintaining quality
- Extract metadata (width, height)
- Convert formats if needed

### Best Practices
1. Validate file type and size before uploading
2. Show upload progress to users
3. Handle upload failures gracefully
4. Implement retry logic for failed uploads
5. Clean up failed uploads
6. Use appropriate file types for content
7. Compress images client-side when possible

### Security Considerations
1. File type validation on both client and server
2. Virus scanning for uploaded files (future enhancement)
3. Presigned URLs expire after 30 minutes
4. Private files require authentication to access
5. File names are sanitized and randomized
6. Regular cleanup of orphaned files