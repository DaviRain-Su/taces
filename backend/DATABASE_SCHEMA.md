# TCM Telemedicine Platform Database Schema

## Overview
This document describes the complete database schema for the TCM Telemedicine Platform. The database uses MySQL 8.0 with UTF-8mb4 character set for full Unicode support.

## Core Tables

### users
User account information for all system users (admins, doctors, patients).
```sql
CREATE TABLE users (
    id CHAR(36) PRIMARY KEY,
    account VARCHAR(50) UNIQUE NOT NULL,
    name VARCHAR(100) NOT NULL,
    password_hash VARCHAR(255) NOT NULL,
    gender ENUM('male', 'female', 'other') NOT NULL,
    phone VARCHAR(20) UNIQUE NOT NULL,
    email VARCHAR(100) UNIQUE,
    birthday DATE,
    role ENUM('admin', 'doctor', 'patient') NOT NULL DEFAULT 'patient',
    status ENUM('active', 'inactive', 'suspended') NOT NULL DEFAULT 'active',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);
```

### doctors
Extended information for users with doctor role.
```sql
CREATE TABLE doctors (
    id CHAR(36) PRIMARY KEY,
    user_id CHAR(36) UNIQUE NOT NULL REFERENCES users(id),
    certificate_type VARCHAR(50) NOT NULL,
    id_number VARCHAR(50) NOT NULL,
    hospital VARCHAR(200) NOT NULL,
    department VARCHAR(100),
    department_id CHAR(36) REFERENCES departments(id),
    title VARCHAR(100) NOT NULL,
    introduction TEXT,
    specialties JSON,
    experience TEXT,
    photos JSON,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);
```

### departments
Hospital departments/specialties.
```sql
CREATE TABLE departments (
    id CHAR(36) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    code VARCHAR(20) UNIQUE NOT NULL,
    contact_person VARCHAR(50),
    contact_phone VARCHAR(20),
    description TEXT,
    status ENUM('active', 'inactive') DEFAULT 'active',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);
```

## Appointment System

### appointments
Patient appointments with doctors.
```sql
CREATE TABLE appointments (
    id CHAR(36) PRIMARY KEY,
    patient_id CHAR(36) NOT NULL REFERENCES users(id),
    doctor_id CHAR(36) NOT NULL REFERENCES doctors(id),
    appointment_date TIMESTAMP NOT NULL,
    time_slot VARCHAR(20) NOT NULL,
    visit_type ENUM('online_video', 'offline') NOT NULL,
    symptoms TEXT NOT NULL,
    has_visited_before BOOLEAN DEFAULT FALSE,
    status ENUM('pending', 'confirmed', 'completed', 'cancelled') DEFAULT 'pending',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);
```

### prescriptions
Medical prescriptions issued by doctors.
```sql
CREATE TABLE prescriptions (
    id CHAR(36) PRIMARY KEY,
    code VARCHAR(20) UNIQUE NOT NULL,
    doctor_id CHAR(36) NOT NULL REFERENCES doctors(id),
    patient_id CHAR(36) NOT NULL REFERENCES users(id),
    patient_name VARCHAR(100) NOT NULL,
    diagnosis TEXT NOT NULL,
    medicines JSON NOT NULL,
    instructions TEXT NOT NULL,
    prescription_date TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

## Patient Management

### patient_profiles
Multiple patient profiles per user account (family members).
```sql
CREATE TABLE patient_profiles (
    id CHAR(36) PRIMARY KEY,
    user_id CHAR(36) NOT NULL REFERENCES users(id),
    name VARCHAR(100) NOT NULL,
    gender ENUM('male', 'female', 'other') NOT NULL,
    birthday DATE NOT NULL,
    phone VARCHAR(20),
    id_number VARCHAR(50),
    relationship VARCHAR(50) NOT NULL,
    medical_history TEXT,
    allergies TEXT,
    is_default BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);
```

### patient_groups
Doctor-created patient groupings.
```sql
CREATE TABLE patient_groups (
    id CHAR(36) PRIMARY KEY,
    doctor_id CHAR(36) NOT NULL REFERENCES doctors(id),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    patient_count INT DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);
```

### patient_group_members
Members of patient groups.
```sql
CREATE TABLE patient_group_members (
    id CHAR(36) PRIMARY KEY,
    group_id CHAR(36) NOT NULL REFERENCES patient_groups(id),
    patient_id CHAR(36) NOT NULL REFERENCES users(id),
    joined_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE KEY uk_group_patient (group_id, patient_id)
);
```

## Content Management

### articles
Health education articles.
```sql
CREATE TABLE articles (
    id CHAR(36) PRIMARY KEY,
    title VARCHAR(200) NOT NULL,
    content TEXT NOT NULL,
    author_id CHAR(36) NOT NULL REFERENCES users(id),
    category_id CHAR(36) REFERENCES content_categories(id),
    cover_image VARCHAR(500),
    summary TEXT,
    tags JSON,
    channels JSON,
    status ENUM('draft', 'published', 'archived') DEFAULT 'draft',
    view_count INT DEFAULT 0,
    published_at TIMESTAMP NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);
```

### videos
Health education videos.
```sql
CREATE TABLE videos (
    id CHAR(36) PRIMARY KEY,
    title VARCHAR(200) NOT NULL,
    description TEXT,
    video_url VARCHAR(500) NOT NULL,
    thumbnail_url VARCHAR(500),
    duration INT,
    author_id CHAR(36) NOT NULL REFERENCES users(id),
    category_id CHAR(36) REFERENCES content_categories(id),
    tags JSON,
    status ENUM('draft', 'published', 'archived') DEFAULT 'draft',
    view_count INT DEFAULT 0,
    published_at TIMESTAMP NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);
```

### content_categories
Categories for organizing content.
```sql
CREATE TABLE content_categories (
    id CHAR(36) PRIMARY KEY,
    name VARCHAR(50) NOT NULL,
    description TEXT,
    parent_id CHAR(36) REFERENCES content_categories(id),
    sort_order INT DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);
```

### live_streams
Doctor-hosted live streaming sessions.
```sql
CREATE TABLE live_streams (
    id CHAR(36) PRIMARY KEY,
    title VARCHAR(200) NOT NULL,
    description TEXT,
    host_id CHAR(36) NOT NULL REFERENCES users(id),
    scheduled_time TIMESTAMP NOT NULL,
    stream_url VARCHAR(500),
    qr_code VARCHAR(500),
    status ENUM('scheduled', 'live', 'ended') DEFAULT 'scheduled',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);
```

## Community Features

### circles
Community circles/groups for patient discussions.
```sql
CREATE TABLE circles (
    id CHAR(36) PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    category VARCHAR(50),
    creator_id CHAR(36) NOT NULL REFERENCES users(id),
    member_count INT DEFAULT 0,
    post_count INT DEFAULT 0,
    is_active BOOLEAN DEFAULT TRUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);
```

### circle_members
Circle membership tracking.
```sql
CREATE TABLE circle_members (
    id CHAR(36) PRIMARY KEY,
    circle_id CHAR(36) NOT NULL REFERENCES circles(id),
    user_id CHAR(36) NOT NULL REFERENCES users(id),
    role ENUM('owner', 'moderator', 'member') DEFAULT 'member',
    joined_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE KEY uk_circle_user (circle_id, user_id)
);
```

### circle_posts
Posts within circles.
```sql
CREATE TABLE circle_posts (
    id CHAR(36) PRIMARY KEY,
    circle_id CHAR(36) NOT NULL REFERENCES circles(id),
    author_id CHAR(36) NOT NULL REFERENCES users(id),
    title VARCHAR(200) NOT NULL,
    content TEXT NOT NULL,
    images JSON,
    likes INT DEFAULT 0,
    comments INT DEFAULT 0,
    status ENUM('active', 'deleted') DEFAULT 'active',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);
```

### post_likes
Like tracking for posts.
```sql
CREATE TABLE post_likes (
    id CHAR(36) PRIMARY KEY,
    post_id CHAR(36) NOT NULL REFERENCES circle_posts(id),
    user_id CHAR(36) NOT NULL REFERENCES users(id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE KEY uk_post_user (post_id, user_id)
);
```

### post_comments
Comments on circle posts.
```sql
CREATE TABLE post_comments (
    id CHAR(36) PRIMARY KEY,
    post_id CHAR(36) NOT NULL REFERENCES circle_posts(id),
    author_id CHAR(36) NOT NULL REFERENCES users(id),
    content TEXT NOT NULL,
    status ENUM('active', 'deleted') DEFAULT 'active',
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);
```

## Review System

### reviews
Patient reviews for doctors.
```sql
CREATE TABLE reviews (
    id CHAR(36) PRIMARY KEY,
    appointment_id CHAR(36) UNIQUE NOT NULL REFERENCES appointments(id),
    patient_id CHAR(36) NOT NULL REFERENCES users(id),
    doctor_id CHAR(36) NOT NULL REFERENCES doctors(id),
    rating INT NOT NULL CHECK (rating >= 1 AND rating <= 5),
    comment TEXT,
    is_anonymous BOOLEAN DEFAULT FALSE,
    is_visible BOOLEAN DEFAULT TRUE,
    doctor_reply TEXT,
    doctor_reply_at TIMESTAMP NULL,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);
```

### review_tags
Tags associated with reviews.
```sql
CREATE TABLE review_tags (
    id CHAR(36) PRIMARY KEY,
    name VARCHAR(50) UNIQUE NOT NULL,
    usage_count INT DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

### review_tag_mappings
Many-to-many relationship between reviews and tags.
```sql
CREATE TABLE review_tag_mappings (
    id CHAR(36) PRIMARY KEY,
    review_id CHAR(36) NOT NULL REFERENCES reviews(id),
    tag_id CHAR(36) NOT NULL REFERENCES review_tags(id),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    UNIQUE KEY uk_review_tag (review_id, tag_id)
);
```

## Template System

### common_phrases
Reusable phrases for doctors.
```sql
CREATE TABLE common_phrases (
    id CHAR(36) PRIMARY KEY,
    doctor_id CHAR(36) NOT NULL REFERENCES doctors(id),
    category VARCHAR(50) NOT NULL,
    content TEXT NOT NULL,
    usage_count INT DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);
```

### prescription_templates
Reusable prescription templates.
```sql
CREATE TABLE prescription_templates (
    id CHAR(36) PRIMARY KEY,
    doctor_id CHAR(36) NOT NULL REFERENCES doctors(id),
    name VARCHAR(100) NOT NULL,
    diagnosis TEXT NOT NULL,
    medicines JSON NOT NULL,
    instructions TEXT NOT NULL,
    usage_count INT DEFAULT 0,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);
```

## Notification System

### notifications
User notifications.
```sql
CREATE TABLE notifications (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    user_id CHAR(36) NOT NULL REFERENCES users(id),
    type ENUM(
        'appointment_reminder',
        'appointment_confirmed',
        'appointment_cancelled',
        'prescription_ready',
        'doctor_reply',
        'system_announcement',
        'review_reply',
        'live_stream_reminder',
        'group_message'
    ) NOT NULL,
    title VARCHAR(200) NOT NULL,
    content TEXT NOT NULL,
    related_id CHAR(36),
    status ENUM('unread', 'read', 'deleted') NOT NULL DEFAULT 'unread',
    metadata JSON DEFAULT ('{}'),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    read_at TIMESTAMP NULL
);
```

### notification_settings
User preferences for notifications.
```sql
CREATE TABLE notification_settings (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    user_id CHAR(36) NOT NULL REFERENCES users(id),
    notification_type ENUM(...) NOT NULL,
    enabled BOOLEAN NOT NULL DEFAULT true,
    email_enabled BOOLEAN NOT NULL DEFAULT false,
    sms_enabled BOOLEAN NOT NULL DEFAULT false,
    push_enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    UNIQUE KEY uk_user_notification_type (user_id, notification_type)
);
```

### sms_logs
SMS sending history.
```sql
CREATE TABLE sms_logs (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    user_id CHAR(36) REFERENCES users(id),
    phone VARCHAR(20) NOT NULL,
    template_code VARCHAR(50) NOT NULL,
    template_params JSON DEFAULT ('{}'),
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    error_message TEXT,
    provider VARCHAR(50),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    sent_at TIMESTAMP NULL
);
```

### email_logs
Email sending history.
```sql
CREATE TABLE email_logs (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    user_id CHAR(36) REFERENCES users(id),
    email VARCHAR(255) NOT NULL,
    subject VARCHAR(255) NOT NULL,
    body TEXT NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending',
    error_message TEXT,
    provider VARCHAR(50),
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    sent_at TIMESTAMP NULL
);
```

### push_tokens
Mobile push notification tokens.
```sql
CREATE TABLE push_tokens (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    user_id CHAR(36) NOT NULL REFERENCES users(id),
    device_type VARCHAR(20) NOT NULL,
    token TEXT NOT NULL,
    device_info JSON DEFAULT ('{}'),
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);
```

## Indexes

Key indexes for performance optimization:

```sql
-- User queries
CREATE INDEX idx_users_role ON users(role);
CREATE INDEX idx_users_status ON users(status);

-- Appointment queries
CREATE INDEX idx_appointments_patient_id ON appointments(patient_id);
CREATE INDEX idx_appointments_doctor_id ON appointments(doctor_id);
CREATE INDEX idx_appointments_date ON appointments(appointment_date);
CREATE INDEX idx_appointments_status ON appointments(status);

-- Content queries
CREATE INDEX idx_articles_author_id ON articles(author_id);
CREATE INDEX idx_articles_status ON articles(status);
CREATE INDEX idx_videos_author_id ON videos(author_id);
CREATE INDEX idx_videos_status ON videos(status);

-- Review queries
CREATE INDEX idx_reviews_doctor_id ON reviews(doctor_id);
CREATE INDEX idx_reviews_patient_id ON reviews(patient_id);
CREATE INDEX idx_reviews_rating ON reviews(rating);

-- Notification queries
CREATE INDEX idx_notifications_user_id ON notifications(user_id);
CREATE INDEX idx_notifications_status ON notifications(status);
CREATE INDEX idx_notifications_created_at ON notifications(created_at DESC);
CREATE INDEX idx_notifications_user_status ON notifications(user_id, status);
```

## Migration Notes

1. All tables use `CHAR(36)` for UUID primary keys
2. Timestamps use `TIMESTAMP` type with automatic update on modification
3. JSON columns store structured data like arrays and objects
4. Proper foreign key constraints ensure referential integrity
5. Indexes are created on frequently queried columns
6. ENUM types are used for fixed value sets
7. UTF8MB4 character set supports full Unicode including emojis