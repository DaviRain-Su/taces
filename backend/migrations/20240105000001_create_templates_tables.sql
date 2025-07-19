-- 创建常用语表
CREATE TABLE common_phrases (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    doctor_id CHAR(36) NOT NULL,
    category VARCHAR(50) NOT NULL COMMENT '分类：diagnosis(诊断), advice(医嘱), symptom(症状描述)',
    content TEXT NOT NULL COMMENT '常用语内容',
    usage_count INT NOT NULL DEFAULT 0 COMMENT '使用次数',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (doctor_id) REFERENCES doctors(id) ON DELETE CASCADE,
    INDEX idx_doctor_id (doctor_id),
    INDEX idx_category (category),
    INDEX idx_usage_count (usage_count DESC)
);

-- 创建处方模板表
CREATE TABLE prescription_templates (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    doctor_id CHAR(36) NOT NULL,
    name VARCHAR(100) NOT NULL COMMENT '模板名称',
    description TEXT COMMENT '模板描述',
    diagnosis TEXT NOT NULL COMMENT '诊断',
    medicines JSON NOT NULL COMMENT '药品列表',
    instructions TEXT NOT NULL COMMENT '用药说明',
    usage_count INT NOT NULL DEFAULT 0 COMMENT '使用次数',
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (doctor_id) REFERENCES doctors(id) ON DELETE CASCADE,
    INDEX idx_doctor_id (doctor_id),
    INDEX idx_name (name),
    INDEX idx_usage_count (usage_count DESC)
);

-- 插入一些示例常用语
INSERT INTO common_phrases (doctor_id, category, content)
SELECT 
    d.id, 
    'diagnosis', 
    '风寒感冒，症见：恶寒重，发热轻，无汗，头痛，肢节酸疼，鼻塞声重，或鼻痒喷嚏'
FROM doctors d
WHERE EXISTS (SELECT 1 FROM users u WHERE u.id = d.user_id AND u.account = 'doctor_dong')
LIMIT 1;

INSERT INTO common_phrases (doctor_id, category, content)
SELECT 
    d.id, 
    'advice', 
    '注意休息，避免劳累；清淡饮食，多饮温开水；保持室内空气流通'
FROM doctors d
WHERE EXISTS (SELECT 1 FROM users u WHERE u.id = d.user_id AND u.account = 'doctor_dong')
LIMIT 1;

INSERT INTO common_phrases (doctor_id, category, content)
SELECT 
    d.id, 
    'symptom', 
    '患者自述：头痛、鼻塞、流清涕，咽部不适，全身酸痛，畏寒'
FROM doctors d
WHERE EXISTS (SELECT 1 FROM users u WHERE u.id = d.user_id AND u.account = 'doctor_dong')
LIMIT 1;

-- 插入一个示例处方模板
INSERT INTO prescription_templates (doctor_id, name, description, diagnosis, medicines, instructions)
SELECT 
    d.id,
    '感冒清热颗粒方',
    '用于风寒感冒引起的头痛发热、恶寒身痛、鼻流清涕、咳嗽咽干',
    '风寒感冒',
    '[{"name":"感冒清热颗粒","specification":"12g*10袋","dosage":"1袋","frequency":"一日3次","duration":"3天","usage":"开水冲服"},{"name":"板蓝根颗粒","specification":"10g*20袋","dosage":"1袋","frequency":"一日3次","duration":"3天","usage":"开水冲服"}]',
    '饭后服用，服药期间多饮水，注意休息，避免受凉'
FROM doctors d
WHERE EXISTS (SELECT 1 FROM users u WHERE u.id = d.user_id AND u.account = 'doctor_dong')
LIMIT 1;