-- Create articles table
CREATE TABLE articles (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    title VARCHAR(200) NOT NULL COMMENT '文章标题',
    cover_image VARCHAR(500) COMMENT '封面图片URL',
    summary VARCHAR(500) COMMENT '文章摘要',
    content TEXT NOT NULL COMMENT '文章内容',
    author_id CHAR(36) NOT NULL COMMENT '作者ID',
    author_name VARCHAR(100) NOT NULL COMMENT '作者姓名',
    author_type ENUM('admin', 'doctor') NOT NULL DEFAULT 'doctor' COMMENT '作者类型',
    category VARCHAR(50) NOT NULL COMMENT '文章分类',
    tags JSON COMMENT '标签列表',
    view_count INT DEFAULT 0 COMMENT '浏览次数',
    like_count INT DEFAULT 0 COMMENT '点赞次数',
    status ENUM('draft', 'published', 'offline') NOT NULL DEFAULT 'draft' COMMENT '状态',
    publish_channels JSON COMMENT '发布渠道',
    published_at DATETIME COMMENT '发布时间',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    INDEX idx_author_id (author_id),
    INDEX idx_category (category),
    INDEX idx_status (status),
    INDEX idx_published_at (published_at),
    FOREIGN KEY (author_id) REFERENCES users(id) ON DELETE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='文章表';

-- Create videos table
CREATE TABLE videos (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    title VARCHAR(200) NOT NULL COMMENT '视频标题',
    cover_image VARCHAR(500) COMMENT '封面图片URL',
    video_url VARCHAR(500) NOT NULL COMMENT '视频URL',
    duration INT COMMENT '视频时长(秒)',
    file_size BIGINT COMMENT '文件大小(字节)',
    description TEXT COMMENT '视频描述',
    author_id CHAR(36) NOT NULL COMMENT '作者ID',
    author_name VARCHAR(100) NOT NULL COMMENT '作者姓名',
    author_type ENUM('admin', 'doctor') NOT NULL DEFAULT 'doctor' COMMENT '作者类型',
    category VARCHAR(50) NOT NULL COMMENT '视频分类',
    tags JSON COMMENT '标签列表',
    view_count INT DEFAULT 0 COMMENT '观看次数',
    like_count INT DEFAULT 0 COMMENT '点赞次数',
    status ENUM('draft', 'processing', 'published', 'offline') NOT NULL DEFAULT 'draft' COMMENT '状态',
    publish_channels JSON COMMENT '发布渠道',
    published_at DATETIME COMMENT '发布时间',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    INDEX idx_author_id (author_id),
    INDEX idx_category (category),
    INDEX idx_status (status),
    INDEX idx_published_at (published_at),
    FOREIGN KEY (author_id) REFERENCES users(id) ON DELETE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='视频表';

-- Create content_categories table
CREATE TABLE content_categories (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    name VARCHAR(50) NOT NULL COMMENT '分类名称',
    type ENUM('article', 'video', 'both') NOT NULL DEFAULT 'both' COMMENT '适用类型',
    sort_order INT DEFAULT 0 COMMENT '排序顺序',
    is_active BOOLEAN DEFAULT TRUE COMMENT '是否启用',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    UNIQUE KEY uk_name_type (name, type)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci COMMENT='内容分类表';

-- Insert default categories
INSERT INTO content_categories (name, type, sort_order) VALUES
('医院介绍', 'article', 1),
('官网新闻', 'article', 2),
('健康科普', 'both', 3),
('活动动态', 'both', 4),
('中医养生', 'both', 5),
('专家讲座', 'video', 6),
('患者教育', 'video', 7);