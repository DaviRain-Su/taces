-- 创建通知表
CREATE TABLE notifications (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    user_id CHAR(36) NOT NULL,
    type ENUM(
        'appointment_reminder',      -- 预约提醒
        'appointment_confirmed',     -- 预约确认
        'appointment_cancelled',     -- 预约取消
        'prescription_ready',        -- 处方已开具
        'doctor_reply',             -- 医生回复
        'system_announcement',      -- 系统公告
        'review_reply',             -- 评价回复
        'live_stream_reminder',     -- 直播提醒
        'group_message'             -- 群发消息
    ) NOT NULL,
    title VARCHAR(200) NOT NULL,
    content TEXT NOT NULL,
    related_id CHAR(36),  -- 关联的业务ID（如预约ID、处方ID等）
    status ENUM('unread', 'read', 'deleted') NOT NULL DEFAULT 'unread',
    metadata JSON DEFAULT ('{}'),  -- 额外的元数据
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    read_at TIMESTAMP NULL,
    
    -- 索引
    INDEX idx_notifications_user_id (user_id),
    INDEX idx_notifications_status (status),
    INDEX idx_notifications_created_at (created_at DESC),
    INDEX idx_notifications_user_status (user_id, status),
    
    -- 外键
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- 创建通知设置表（用户可以配置接收哪些类型的通知）
CREATE TABLE notification_settings (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    user_id CHAR(36) NOT NULL,
    notification_type ENUM(
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
    enabled BOOLEAN NOT NULL DEFAULT true,
    email_enabled BOOLEAN NOT NULL DEFAULT false,
    sms_enabled BOOLEAN NOT NULL DEFAULT false,
    push_enabled BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    -- 唯一约束
    UNIQUE KEY uk_user_notification_type (user_id, notification_type),
    
    -- 索引
    INDEX idx_notification_settings_user_id (user_id),
    
    -- 外键
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);

-- 创建短信发送记录表
CREATE TABLE sms_logs (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    user_id CHAR(36),
    phone VARCHAR(20) NOT NULL,
    template_code VARCHAR(50) NOT NULL,
    template_params JSON DEFAULT ('{}'),
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- pending, sent, failed
    error_message TEXT,
    provider VARCHAR(50), -- aliyun, tencent
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    sent_at TIMESTAMP NULL,
    
    -- 索引
    INDEX idx_sms_logs_user_id (user_id),
    INDEX idx_sms_logs_phone (phone),
    INDEX idx_sms_logs_created_at (created_at DESC),
    
    -- 外键
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
);

-- 创建邮件发送记录表
CREATE TABLE email_logs (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    user_id CHAR(36),
    email VARCHAR(255) NOT NULL,
    subject VARCHAR(255) NOT NULL,
    body TEXT NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'pending', -- pending, sent, failed
    error_message TEXT,
    provider VARCHAR(50), -- smtp, sendgrid, etc
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    sent_at TIMESTAMP NULL,
    
    -- 索引
    INDEX idx_email_logs_user_id (user_id),
    INDEX idx_email_logs_email (email),
    INDEX idx_email_logs_created_at (created_at DESC),
    
    -- 外键
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE SET NULL
);

-- 创建推送token表（用于移动端推送）
CREATE TABLE push_tokens (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    user_id CHAR(36) NOT NULL,
    device_type VARCHAR(20) NOT NULL, -- ios, android, web
    token TEXT NOT NULL,
    device_info JSON DEFAULT ('{}'),
    active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    -- 索引
    INDEX idx_push_tokens_user_id (user_id),
    INDEX idx_push_tokens_token (token(255)),
    
    -- 外键
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE
);