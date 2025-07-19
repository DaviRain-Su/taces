-- 创建支付订单表
CREATE TABLE payment_orders (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    order_no VARCHAR(50) UNIQUE NOT NULL COMMENT '订单号',
    user_id CHAR(36) NOT NULL COMMENT '用户ID',
    appointment_id CHAR(36) COMMENT '关联的预约ID',
    order_type ENUM('appointment', 'consultation', 'prescription', 'other') NOT NULL COMMENT '订单类型',
    amount DECIMAL(10, 2) NOT NULL COMMENT '订单金额',
    currency VARCHAR(10) NOT NULL DEFAULT 'CNY' COMMENT '货币类型',
    status ENUM('pending', 'paid', 'cancelled', 'refunded', 'partial_refunded', 'expired') NOT NULL DEFAULT 'pending' COMMENT '订单状态',
    payment_method ENUM('wechat', 'alipay', 'bank_card', 'balance') COMMENT '支付方式',
    payment_time TIMESTAMP NULL COMMENT '支付时间',
    expire_time TIMESTAMP NOT NULL COMMENT '订单过期时间',
    description VARCHAR(500) COMMENT '订单描述',
    metadata JSON COMMENT '额外数据',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    -- 索引
    INDEX idx_payment_orders_user_id (user_id),
    INDEX idx_payment_orders_appointment_id (appointment_id),
    INDEX idx_payment_orders_order_no (order_no),
    INDEX idx_payment_orders_status (status),
    INDEX idx_payment_orders_created_at (created_at DESC),
    
    -- 外键
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (appointment_id) REFERENCES appointments(id)
) COMMENT='支付订单表';

-- 创建支付交易记录表
CREATE TABLE payment_transactions (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    transaction_no VARCHAR(50) UNIQUE NOT NULL COMMENT '交易流水号',
    order_id CHAR(36) NOT NULL COMMENT '订单ID',
    payment_method ENUM('wechat', 'alipay', 'bank_card', 'balance') NOT NULL COMMENT '支付方式',
    transaction_type ENUM('payment', 'refund') NOT NULL COMMENT '交易类型',
    amount DECIMAL(10, 2) NOT NULL COMMENT '交易金额',
    status ENUM('pending', 'success', 'failed') NOT NULL DEFAULT 'pending' COMMENT '交易状态',
    
    -- 第三方支付信息
    external_transaction_id VARCHAR(100) COMMENT '第三方交易ID',
    prepay_id VARCHAR(100) COMMENT '预支付ID（微信）',
    trade_no VARCHAR(100) COMMENT '支付宝交易号',
    
    -- 请求和响应数据
    request_data JSON COMMENT '请求数据',
    response_data JSON COMMENT '响应数据',
    callback_data JSON COMMENT '回调数据',
    
    -- 错误信息
    error_code VARCHAR(50) COMMENT '错误代码',
    error_message VARCHAR(500) COMMENT '错误信息',
    
    -- 时间戳
    initiated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP COMMENT '发起时间',
    completed_at TIMESTAMP NULL COMMENT '完成时间',
    
    -- 索引
    INDEX idx_payment_transactions_order_id (order_id),
    INDEX idx_payment_transactions_transaction_no (transaction_no),
    INDEX idx_payment_transactions_external_id (external_transaction_id),
    INDEX idx_payment_transactions_status (status),
    
    -- 外键
    FOREIGN KEY (order_id) REFERENCES payment_orders(id)
) COMMENT='支付交易记录表';

-- 创建退款记录表
CREATE TABLE refund_records (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    refund_no VARCHAR(50) UNIQUE NOT NULL COMMENT '退款单号',
    order_id CHAR(36) NOT NULL COMMENT '原订单ID',
    transaction_id CHAR(36) NOT NULL COMMENT '原交易ID',
    user_id CHAR(36) NOT NULL COMMENT '用户ID',
    refund_amount DECIMAL(10, 2) NOT NULL COMMENT '退款金额',
    refund_reason VARCHAR(500) NOT NULL COMMENT '退款原因',
    status ENUM('pending', 'processing', 'success', 'failed', 'cancelled') NOT NULL DEFAULT 'pending' COMMENT '退款状态',
    
    -- 审核信息
    reviewed_by CHAR(36) COMMENT '审核人ID',
    reviewed_at TIMESTAMP NULL COMMENT '审核时间',
    review_notes VARCHAR(500) COMMENT '审核备注',
    
    -- 第三方退款信息
    external_refund_id VARCHAR(100) COMMENT '第三方退款ID',
    refund_response JSON COMMENT '退款响应数据',
    
    -- 时间戳
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    completed_at TIMESTAMP NULL COMMENT '退款完成时间',
    
    -- 索引
    INDEX idx_refund_records_order_id (order_id),
    INDEX idx_refund_records_user_id (user_id),
    INDEX idx_refund_records_refund_no (refund_no),
    INDEX idx_refund_records_status (status),
    
    -- 外键
    FOREIGN KEY (order_id) REFERENCES payment_orders(id),
    FOREIGN KEY (transaction_id) REFERENCES payment_transactions(id),
    FOREIGN KEY (user_id) REFERENCES users(id),
    FOREIGN KEY (reviewed_by) REFERENCES users(id)
) COMMENT='退款记录表';

-- 创建支付配置表
CREATE TABLE payment_configs (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    payment_method ENUM('wechat', 'alipay') NOT NULL COMMENT '支付方式',
    config_key VARCHAR(50) NOT NULL COMMENT '配置键',
    config_value TEXT NOT NULL COMMENT '配置值',
    is_encrypted BOOLEAN DEFAULT FALSE COMMENT '是否加密存储',
    description VARCHAR(200) COMMENT '配置描述',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    -- 唯一约束
    UNIQUE KEY uk_payment_method_key (payment_method, config_key)
) COMMENT='支付配置表';

-- 创建价格配置表
CREATE TABLE price_configs (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    service_type VARCHAR(50) NOT NULL COMMENT '服务类型',
    service_name VARCHAR(100) NOT NULL COMMENT '服务名称',
    price DECIMAL(10, 2) NOT NULL COMMENT '价格',
    discount_price DECIMAL(10, 2) COMMENT '折扣价',
    is_active BOOLEAN DEFAULT TRUE COMMENT '是否启用',
    effective_date DATE COMMENT '生效日期',
    expiry_date DATE COMMENT '失效日期',
    description TEXT COMMENT '描述',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    -- 索引
    INDEX idx_price_configs_service_type (service_type),
    INDEX idx_price_configs_is_active (is_active)
) COMMENT='价格配置表';

-- 创建用户余额表
CREATE TABLE user_balances (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    user_id CHAR(36) UNIQUE NOT NULL COMMENT '用户ID',
    balance DECIMAL(10, 2) NOT NULL DEFAULT 0.00 COMMENT '余额',
    frozen_balance DECIMAL(10, 2) NOT NULL DEFAULT 0.00 COMMENT '冻结余额',
    total_income DECIMAL(10, 2) NOT NULL DEFAULT 0.00 COMMENT '累计收入',
    total_expense DECIMAL(10, 2) NOT NULL DEFAULT 0.00 COMMENT '累计支出',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    
    -- 外键
    FOREIGN KEY (user_id) REFERENCES users(id)
) COMMENT='用户余额表';

-- 创建余额变动记录表
CREATE TABLE balance_transactions (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    user_id CHAR(36) NOT NULL COMMENT '用户ID',
    transaction_type ENUM('income', 'expense', 'freeze', 'unfreeze') NOT NULL COMMENT '交易类型',
    amount DECIMAL(10, 2) NOT NULL COMMENT '金额',
    balance_before DECIMAL(10, 2) NOT NULL COMMENT '变动前余额',
    balance_after DECIMAL(10, 2) NOT NULL COMMENT '变动后余额',
    related_type VARCHAR(50) COMMENT '关联类型',
    related_id CHAR(36) COMMENT '关联ID',
    description VARCHAR(200) NOT NULL COMMENT '描述',
    created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    
    -- 索引
    INDEX idx_balance_transactions_user_id (user_id),
    INDEX idx_balance_transactions_type (transaction_type),
    INDEX idx_balance_transactions_created_at (created_at DESC),
    
    -- 外键
    FOREIGN KEY (user_id) REFERENCES users(id)
) COMMENT='余额变动记录表';

-- 插入默认价格配置
INSERT INTO price_configs (service_type, service_name, price, description) VALUES
('appointment_offline', '线下问诊', 50.00, '线下面对面问诊服务'),
('appointment_online', '视频问诊', 30.00, '在线视频问诊服务'),
('prescription', '处方费', 10.00, '电子处方服务费'),
('consultation', '图文咨询', 20.00, '图文咨询服务费');

-- 插入支付配置模板（实际值需要后续配置）
INSERT INTO payment_configs (payment_method, config_key, config_value, is_encrypted, description) VALUES
-- 微信支付配置
('wechat', 'app_id', '', FALSE, '微信公众号/小程序AppID'),
('wechat', 'mch_id', '', FALSE, '微信商户号'),
('wechat', 'api_key', '', TRUE, '微信支付API密钥'),
('wechat', 'cert_path', '', FALSE, '微信支付证书路径'),
('wechat', 'notify_url', '', FALSE, '微信支付回调地址'),
-- 支付宝配置
('alipay', 'app_id', '', FALSE, '支付宝应用ID'),
('alipay', 'private_key', '', TRUE, '支付宝应用私钥'),
('alipay', 'public_key', '', TRUE, '支付宝公钥'),
('alipay', 'notify_url', '', FALSE, '支付宝回调地址'),
('alipay', 'return_url', '', FALSE, '支付宝返回地址');