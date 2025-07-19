-- Create patient_profiles table
CREATE TABLE patient_profiles (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    user_id CHAR(36) NOT NULL,
    name VARCHAR(50) NOT NULL,
    id_number VARCHAR(18) NOT NULL,
    phone VARCHAR(20) NOT NULL,
    gender ENUM('男', '女') NOT NULL,
    birthday DATE,
    relationship ENUM('self', 'family', 'friend', 'other') NOT NULL,
    is_default BOOLEAN NOT NULL DEFAULT FALSE,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_user_id (user_id),
    INDEX idx_user_default (user_id, is_default)
);