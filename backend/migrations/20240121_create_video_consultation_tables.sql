-- 创建视频问诊会话表
CREATE TABLE video_consultations (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    appointment_id CHAR(36) NOT NULL COMMENT '关联的预约ID',
    doctor_id CHAR(36) NOT NULL COMMENT '医生ID',
    patient_id CHAR(36) NOT NULL COMMENT '患者ID',
    room_id VARCHAR(100) UNIQUE NOT NULL COMMENT '房间ID',
    status ENUM('waiting', 'in_progress', 'completed', 'cancelled', 'no_show') NOT NULL DEFAULT 'waiting' COMMENT '会话状态',
    scheduled_start_time TIMESTAMP NOT NULL COMMENT '预定开始时间',
    actual_start_time TIMESTAMP NULL COMMENT '实际开始时间',
    end_time TIMESTAMP NULL COMMENT '结束时间',
    duration INT COMMENT '通话时长（秒）',
    
    -- WebRTC 相关
    doctor_token TEXT COMMENT '医生端访问令牌',
    patient_token TEXT COMMENT '患者端访问令牌',
    ice_servers JSON COMMENT 'ICE服务器配置',
    
    -- 诊断信息
    chief_complaint TEXT COMMENT '主诉',
    diagnosis TEXT COMMENT '诊断',
    treatment_plan TEXT COMMENT '治疗方案',
    notes TEXT COMMENT '医生笔记',
    
    -- 质量和评价
    connection_quality ENUM('excellent', 'good', 'fair', 'poor') COMMENT '连接质量',
    patient_rating INT COMMENT '患者评分 1-5',
    patient_feedback TEXT COMMENT '患者反馈',
    
    -- 元数据
    metadata JSON COMMENT '额外数据',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    -- 索引
    INDEX idx_video_consultations_appointment_id (appointment_id),
    INDEX idx_video_consultations_doctor_id (doctor_id),
    INDEX idx_video_consultations_patient_id (patient_id),
    INDEX idx_video_consultations_room_id (room_id),
    INDEX idx_video_consultations_status (status),
    INDEX idx_video_consultations_scheduled_time (scheduled_start_time),
    
    -- 外键
    FOREIGN KEY (appointment_id) REFERENCES appointments(id),
    FOREIGN KEY (doctor_id) REFERENCES doctors(id),
    FOREIGN KEY (patient_id) REFERENCES users(id)
) COMMENT='视频问诊会话表';

-- 创建视频问诊录制记录表
CREATE TABLE video_recordings (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    consultation_id CHAR(36) NOT NULL COMMENT '问诊会话ID',
    recording_url VARCHAR(500) COMMENT '录制文件URL',
    thumbnail_url VARCHAR(500) COMMENT '缩略图URL',
    file_size BIGINT COMMENT '文件大小（字节）',
    duration INT COMMENT '录制时长（秒）',
    format VARCHAR(20) COMMENT '文件格式',
    status ENUM('recording', 'processing', 'completed', 'failed') NOT NULL DEFAULT 'recording' COMMENT '录制状态',
    error_message TEXT COMMENT '错误信息',
    started_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '开始录制时间',
    completed_at TIMESTAMP NULL COMMENT '完成时间',
    expires_at TIMESTAMP NULL COMMENT '过期时间',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- 索引
    INDEX idx_video_recordings_consultation_id (consultation_id),
    INDEX idx_video_recordings_status (status),
    
    -- 外键
    FOREIGN KEY (consultation_id) REFERENCES video_consultations(id)
) COMMENT='视频问诊录制记录表';

-- 创建WebRTC信令消息表（用于消息中转）
CREATE TABLE webrtc_signals (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    room_id VARCHAR(100) NOT NULL COMMENT '房间ID',
    from_user_id CHAR(36) NOT NULL COMMENT '发送者ID',
    to_user_id CHAR(36) NOT NULL COMMENT '接收者ID',
    signal_type ENUM('offer', 'answer', 'ice_candidate', 'join', 'leave', 'error') NOT NULL COMMENT '信令类型',
    payload JSON NOT NULL COMMENT '信令数据',
    delivered BOOLEAN DEFAULT FALSE COMMENT '是否已投递',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- 索引
    INDEX idx_webrtc_signals_room_id (room_id),
    INDEX idx_webrtc_signals_to_user (to_user_id, delivered),
    INDEX idx_webrtc_signals_created_at (created_at)
) COMMENT='WebRTC信令消息表';

-- 创建视频通话事件日志表
CREATE TABLE video_call_events (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    consultation_id CHAR(36) NOT NULL COMMENT '问诊会话ID',
    user_id CHAR(36) NOT NULL COMMENT '用户ID',
    event_type ENUM(
        'joined', 'left', 'reconnected', 'disconnected',
        'camera_on', 'camera_off', 'mic_on', 'mic_off',
        'screen_share_start', 'screen_share_end',
        'recording_start', 'recording_end',
        'network_poor', 'network_recovered'
    ) NOT NULL COMMENT '事件类型',
    event_data JSON COMMENT '事件数据',
    ip_address VARCHAR(45) COMMENT 'IP地址',
    user_agent TEXT COMMENT '用户代理',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- 索引
    INDEX idx_video_call_events_consultation_id (consultation_id),
    INDEX idx_video_call_events_user_id (user_id),
    INDEX idx_video_call_events_type (event_type),
    INDEX idx_video_call_events_created_at (created_at),
    
    -- 外键
    FOREIGN KEY (consultation_id) REFERENCES video_consultations(id),
    FOREIGN KEY (user_id) REFERENCES users(id)
) COMMENT='视频通话事件日志表';

-- 创建视频问诊模板表（医生可以保存常用的诊断模板）
CREATE TABLE video_consultation_templates (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    doctor_id CHAR(36) NOT NULL COMMENT '医生ID',
    name VARCHAR(100) NOT NULL COMMENT '模板名称',
    chief_complaint TEXT COMMENT '主诉模板',
    diagnosis TEXT COMMENT '诊断模板',
    treatment_plan TEXT COMMENT '治疗方案模板',
    notes TEXT COMMENT '备注模板',
    usage_count INT DEFAULT 0 COMMENT '使用次数',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    -- 索引
    INDEX idx_consultation_templates_doctor_id (doctor_id),
    
    -- 外键
    FOREIGN KEY (doctor_id) REFERENCES doctors(id)
) COMMENT='视频问诊模板表';

-- 创建文件上传记录表
CREATE TABLE file_uploads (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    user_id CHAR(36) NOT NULL COMMENT '上传用户ID',
    file_type ENUM('image', 'video', 'document', 'audio', 'other') NOT NULL COMMENT '文件类型',
    file_name VARCHAR(255) NOT NULL COMMENT '原始文件名',
    file_path VARCHAR(500) NOT NULL COMMENT '存储路径',
    file_url VARCHAR(500) NOT NULL COMMENT '访问URL',
    file_size BIGINT NOT NULL COMMENT '文件大小（字节）',
    mime_type VARCHAR(100) COMMENT 'MIME类型',
    
    -- 关联信息
    related_type VARCHAR(50) COMMENT '关联类型（如：consultation, prescription等）',
    related_id CHAR(36) COMMENT '关联ID',
    
    -- 图片专用字段
    width INT COMMENT '宽度（像素）',
    height INT COMMENT '高度（像素）',
    thumbnail_url VARCHAR(500) COMMENT '缩略图URL',
    
    -- 安全和状态
    is_public BOOLEAN DEFAULT FALSE COMMENT '是否公开访问',
    status ENUM('uploading', 'completed', 'failed', 'deleted') NOT NULL DEFAULT 'uploading' COMMENT '状态',
    error_message TEXT COMMENT '错误信息',
    
    -- OSS相关
    bucket_name VARCHAR(100) COMMENT 'OSS桶名',
    object_key VARCHAR(500) COMMENT 'OSS对象键',
    etag VARCHAR(100) COMMENT 'ETag',
    
    -- 时间戳
    uploaded_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at TIMESTAMP NULL COMMENT '过期时间',
    deleted_at TIMESTAMP NULL COMMENT '删除时间',
    
    -- 索引
    INDEX idx_file_uploads_user_id (user_id),
    INDEX idx_file_uploads_related (related_type, related_id),
    INDEX idx_file_uploads_status (status),
    INDEX idx_file_uploads_uploaded_at (uploaded_at DESC),
    
    -- 外键
    FOREIGN KEY (user_id) REFERENCES users(id)
) COMMENT='文件上传记录表';

-- 创建系统配置表（用于存储视频通话配置等）
CREATE TABLE system_configs (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    category VARCHAR(50) NOT NULL COMMENT '配置分类',
    config_key VARCHAR(100) NOT NULL COMMENT '配置键',
    config_value TEXT NOT NULL COMMENT '配置值',
    value_type ENUM('string', 'number', 'boolean', 'json') NOT NULL DEFAULT 'string' COMMENT '值类型',
    description TEXT COMMENT '配置描述',
    is_encrypted BOOLEAN DEFAULT FALSE COMMENT '是否加密',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    -- 唯一约束
    UNIQUE KEY uk_category_key (category, config_key),
    
    -- 索引
    INDEX idx_system_configs_category (category)
) COMMENT='系统配置表';

-- 插入默认视频通话配置
INSERT INTO system_configs (category, config_key, config_value, value_type, description) VALUES
('video_call', 'max_duration', '3600', 'number', '最大通话时长（秒）'),
('video_call', 'recording_enabled', 'true', 'boolean', '是否启用录制'),
('video_call', 'ice_servers', '[{"urls": ["stun:stun.l.google.com:19302"]}]', 'json', 'ICE服务器配置'),
('video_call', 'video_codec', 'VP8', 'string', '视频编解码器'),
('video_call', 'audio_codec', 'opus', 'string', '音频编解码器'),
('video_call', 'max_video_bitrate', '1000000', 'number', '最大视频码率（bps）'),
('video_call', 'max_audio_bitrate', '64000', 'number', '最大音频码率（bps）'),
('file_upload', 'max_image_size', '10485760', 'number', '最大图片大小（10MB）'),
('file_upload', 'max_video_size', '104857600', 'number', '最大视频大小（100MB）'),
('file_upload', 'allowed_image_types', '["jpg","jpeg","png","gif","webp"]', 'json', '允许的图片类型'),
('file_upload', 'allowed_video_types', '["mp4","webm","mov"]', 'json', '允许的视频类型'),
('file_upload', 'image_compression_quality', '85', 'number', '图片压缩质量（0-100）');

-- 定期清理过期的WebRTC信令（可通过定时任务执行）
-- DELETE FROM webrtc_signals WHERE created_at < DATE_SUB(NOW(), INTERVAL 1 HOUR);