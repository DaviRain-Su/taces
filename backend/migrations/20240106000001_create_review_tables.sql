-- 创建患者评价表
CREATE TABLE patient_reviews (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    appointment_id CHAR(36) NOT NULL COMMENT '关联的预约ID',
    doctor_id CHAR(36) NOT NULL COMMENT '被评价的医生ID',
    patient_id CHAR(36) NOT NULL COMMENT '评价的患者ID',
    rating INT NOT NULL COMMENT '评分（1-5星）',
    attitude_rating INT NOT NULL COMMENT '服务态度评分（1-5星）',
    professionalism_rating INT NOT NULL COMMENT '专业水平评分（1-5星）',
    efficiency_rating INT NOT NULL COMMENT '诊疗效率评分（1-5星）',
    comment TEXT COMMENT '评价内容',
    reply TEXT COMMENT '医生回复',
    reply_at DATETIME COMMENT '回复时间',
    is_anonymous BOOLEAN NOT NULL DEFAULT FALSE COMMENT '是否匿名',
    is_visible BOOLEAN NOT NULL DEFAULT TRUE COMMENT '是否显示',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (appointment_id) REFERENCES appointments(id),
    FOREIGN KEY (doctor_id) REFERENCES doctors(id),
    FOREIGN KEY (patient_id) REFERENCES users(id),
    UNIQUE KEY unique_appointment_review (appointment_id),
    INDEX idx_doctor_id (doctor_id),
    INDEX idx_patient_id (patient_id),
    INDEX idx_rating (rating),
    INDEX idx_created_at (created_at DESC),
    INDEX idx_visible (is_visible)
);

-- 创建评价标签表
CREATE TABLE review_tags (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    name VARCHAR(50) NOT NULL UNIQUE COMMENT '标签名称',
    category ENUM('positive', 'negative') NOT NULL COMMENT '标签类型：正面/负面',
    usage_count INT NOT NULL DEFAULT 0 COMMENT '使用次数',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    INDEX idx_category (category),
    INDEX idx_usage_count (usage_count DESC)
);

-- 创建评价-标签关联表
CREATE TABLE review_tag_relations (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    review_id CHAR(36) NOT NULL,
    tag_id CHAR(36) NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (review_id) REFERENCES patient_reviews(id) ON DELETE CASCADE,
    FOREIGN KEY (tag_id) REFERENCES review_tags(id),
    UNIQUE KEY unique_review_tag (review_id, tag_id),
    INDEX idx_review_id (review_id),
    INDEX idx_tag_id (tag_id)
);

-- 插入默认评价标签
INSERT INTO review_tags (name, category) VALUES
-- 正面标签
('医术精湛', 'positive'),
('态度友好', 'positive'),
('耐心细致', 'positive'),
('经验丰富', 'positive'),
('诊断准确', 'positive'),
('疗效显著', 'positive'),
('解释清楚', 'positive'),
('服务周到', 'positive'),
('医德高尚', 'positive'),
('值得信赖', 'positive'),
-- 负面标签
('态度冷淡', 'negative'),
('等待时间长', 'negative'),
('解释不清', 'negative'),
('效果一般', 'negative'),
('费用偏高', 'negative');

-- 更新医生表，添加评价统计字段
ALTER TABLE doctors
ADD COLUMN total_reviews INT NOT NULL DEFAULT 0 COMMENT '总评价数',
ADD COLUMN average_rating DECIMAL(3,2) NOT NULL DEFAULT 0.00 COMMENT '平均评分',
ADD COLUMN average_attitude DECIMAL(3,2) NOT NULL DEFAULT 0.00 COMMENT '平均态度评分',
ADD COLUMN average_professionalism DECIMAL(3,2) NOT NULL DEFAULT 0.00 COMMENT '平均专业评分',
ADD COLUMN average_efficiency DECIMAL(3,2) NOT NULL DEFAULT 0.00 COMMENT '平均效率评分',
ADD INDEX idx_average_rating (average_rating DESC);

-- 创建评价统计视图
CREATE VIEW doctor_review_statistics AS
SELECT 
    d.id AS doctor_id,
    d.user_id,
    COUNT(pr.id) AS total_reviews,
    COALESCE(AVG(pr.rating), 0) AS average_rating,
    COALESCE(AVG(pr.attitude_rating), 0) AS average_attitude,
    COALESCE(AVG(pr.professionalism_rating), 0) AS average_professionalism,
    COALESCE(AVG(pr.efficiency_rating), 0) AS average_efficiency,
    SUM(CASE WHEN pr.rating = 5 THEN 1 ELSE 0 END) AS five_star_count,
    SUM(CASE WHEN pr.rating = 4 THEN 1 ELSE 0 END) AS four_star_count,
    SUM(CASE WHEN pr.rating = 3 THEN 1 ELSE 0 END) AS three_star_count,
    SUM(CASE WHEN pr.rating = 2 THEN 1 ELSE 0 END) AS two_star_count,
    SUM(CASE WHEN pr.rating = 1 THEN 1 ELSE 0 END) AS one_star_count
FROM doctors d
LEFT JOIN patient_reviews pr ON d.id = pr.doctor_id AND pr.is_visible = TRUE
GROUP BY d.id;