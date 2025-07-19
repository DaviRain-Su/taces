-- 创建圈子表
CREATE TABLE circles (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    avatar VARCHAR(255),
    category VARCHAR(50) NOT NULL,
    creator_id CHAR(36) NOT NULL,
    member_count INT NOT NULL DEFAULT 0,
    post_count INT NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (creator_id) REFERENCES users(id),
    INDEX idx_category (category),
    INDEX idx_creator (creator_id),
    INDEX idx_active (is_active)
);

-- 创建圈子成员表
CREATE TABLE circle_members (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    circle_id CHAR(36) NOT NULL,
    user_id CHAR(36) NOT NULL,
    role ENUM('owner', 'admin', 'member') NOT NULL DEFAULT 'member',
    joined_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (circle_id) REFERENCES circles(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE KEY unique_circle_member (circle_id, user_id),
    INDEX idx_circle_id (circle_id),
    INDEX idx_user_id (user_id),
    INDEX idx_role (role)
);

-- 修改 circle_posts 表以添加外键约束
ALTER TABLE circle_posts
ADD CONSTRAINT fk_circle_posts_circle
FOREIGN KEY (circle_id) REFERENCES circles(id) ON DELETE CASCADE;

-- 创建帖子点赞表
CREATE TABLE post_likes (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    post_id CHAR(36) NOT NULL,
    user_id CHAR(36) NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (post_id) REFERENCES circle_posts(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    UNIQUE KEY unique_post_like (post_id, user_id),
    INDEX idx_post_id (post_id),
    INDEX idx_user_id (user_id)
);

-- 创建帖子评论表
CREATE TABLE post_comments (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    post_id CHAR(36) NOT NULL,
    user_id CHAR(36) NOT NULL,
    content TEXT NOT NULL,
    is_deleted BOOLEAN NOT NULL DEFAULT FALSE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (post_id) REFERENCES circle_posts(id) ON DELETE CASCADE,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_post_id (post_id),
    INDEX idx_user_id (user_id),
    INDEX idx_created_at (created_at)
);

-- 创建敏感词表
CREATE TABLE sensitive_words (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    word VARCHAR(100) NOT NULL UNIQUE,
    category VARCHAR(50),
    severity ENUM('low', 'medium', 'high') NOT NULL DEFAULT 'medium',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_word (word),
    INDEX idx_active (is_active)
);

-- 插入默认敏感词
INSERT INTO sensitive_words (word, category, severity) VALUES
('违法', '法律', 'high'),
('赌博', '违规', 'high'),
('诈骗', '违规', 'high'),
('传销', '违规', 'high'),
('非法集资', '违规', 'high');

-- 插入默认圈子分类（使用管理员用户作为创建者）
INSERT INTO circles (id, name, description, category, creator_id, member_count) 
SELECT 
    '00000000-0000-0000-0000-000000000001', 
    '董老师养生堂', 
    '董老师分享中医养生知识', 
    '中医养生', 
    id, 
    1
FROM users WHERE account = 'admin' LIMIT 1;

INSERT INTO circles (id, name, description, category, creator_id, member_count) 
SELECT 
    '00000000-0000-0000-0000-000000000002', 
    '中医爱好者', 
    '中医文化交流社区', 
    '中医文化', 
    id, 
    1
FROM users WHERE account = 'admin' LIMIT 1;

INSERT INTO circles (id, name, description, category, creator_id, member_count) 
SELECT 
    '00000000-0000-0000-0000-000000000003', 
    '慢性病调理', 
    '慢性病患者互助交流', 
    '健康管理', 
    id, 
    1
FROM users WHERE account = 'admin' LIMIT 1;