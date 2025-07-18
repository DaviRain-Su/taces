-- Create UUID function for MySQL
DELIMITER $$
CREATE FUNCTION IF NOT EXISTS UUID_V4() RETURNS CHAR(36)
DETERMINISTIC
BEGIN
    RETURN LOWER(CONCAT(
        HEX(RANDOM_BYTES(4)),
        '-', HEX(RANDOM_BYTES(2)),
        '-4', SUBSTR(HEX(RANDOM_BYTES(2)), 2, 3),
        '-', HEX(FLOOR(ASCII(RANDOM_BYTES(1)) / 64) + 8), SUBSTR(HEX(RANDOM_BYTES(2)), 2, 3),
        '-', HEX(RANDOM_BYTES(6))
    ));
END$$
DELIMITER ;

-- Create users table
CREATE TABLE users (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    account VARCHAR(50) UNIQUE NOT NULL,
    name VARCHAR(50) NOT NULL,
    password VARCHAR(255) NOT NULL,
    gender VARCHAR(10) NOT NULL,
    phone VARCHAR(20) NOT NULL,
    email VARCHAR(100),
    birthday DATETIME,
    role ENUM('admin', 'doctor', 'patient') NOT NULL,
    status ENUM('active', 'inactive') NOT NULL DEFAULT 'active',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    INDEX idx_account (account),
    INDEX idx_phone (phone)
);

-- Create doctors table
CREATE TABLE doctors (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    user_id CHAR(36) NOT NULL,
    certificate_type VARCHAR(50) NOT NULL,
    id_number VARCHAR(18) NOT NULL,
    hospital VARCHAR(100) NOT NULL,
    department VARCHAR(50) NOT NULL,
    title VARCHAR(50) NOT NULL,
    introduction TEXT,
    specialties JSON NOT NULL DEFAULT ('[]'),
    experience TEXT,
    avatar VARCHAR(255),
    license_photo VARCHAR(255),
    id_card_front VARCHAR(255),
    id_card_back VARCHAR(255),
    title_cert VARCHAR(255),
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE,
    INDEX idx_user_id (user_id)
);

-- Create appointments table
CREATE TABLE appointments (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    patient_id CHAR(36) NOT NULL,
    doctor_id CHAR(36) NOT NULL,
    appointment_date DATETIME NOT NULL,
    time_slot VARCHAR(20) NOT NULL,
    visit_type ENUM('online_video', 'offline') NOT NULL,
    symptoms VARCHAR(100) NOT NULL,
    has_visited_before BOOLEAN NOT NULL DEFAULT FALSE,
    status ENUM('pending', 'confirmed', 'completed', 'cancelled') NOT NULL DEFAULT 'pending',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (patient_id) REFERENCES users(id),
    FOREIGN KEY (doctor_id) REFERENCES doctors(id),
    INDEX idx_patient_id (patient_id),
    INDEX idx_doctor_id (doctor_id),
    INDEX idx_appointment_date (appointment_date)
);

-- Create prescriptions table
CREATE TABLE prescriptions (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    code VARCHAR(50) UNIQUE NOT NULL,
    doctor_id CHAR(36) NOT NULL,
    patient_id CHAR(36) NOT NULL,
    patient_name VARCHAR(50) NOT NULL,
    diagnosis TEXT NOT NULL,
    medicines JSON NOT NULL,
    instructions TEXT NOT NULL,
    prescription_date DATETIME NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY (doctor_id) REFERENCES doctors(id),
    FOREIGN KEY (patient_id) REFERENCES users(id),
    INDEX idx_prescription_patient_id (patient_id),
    INDEX idx_prescription_doctor_id (doctor_id)
);

-- Create live_streams table
CREATE TABLE live_streams (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    title VARCHAR(200) NOT NULL,
    host_id CHAR(36) NOT NULL,
    host_name VARCHAR(50) NOT NULL,
    scheduled_time DATETIME NOT NULL,
    stream_url VARCHAR(255),
    qr_code VARCHAR(255),
    status ENUM('scheduled', 'live', 'ended') NOT NULL DEFAULT 'scheduled',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (host_id) REFERENCES users(id),
    INDEX idx_host_id (host_id)
);

-- Create circle_posts table
CREATE TABLE circle_posts (
    id CHAR(36) PRIMARY KEY DEFAULT (UUID()),
    author_id CHAR(36) NOT NULL,
    circle_id CHAR(36) NOT NULL,
    title VARCHAR(200) NOT NULL,
    content TEXT NOT NULL,
    images JSON NOT NULL DEFAULT ('[]'),
    likes BIGINT NOT NULL DEFAULT 0,
    comments BIGINT NOT NULL DEFAULT 0,
    status ENUM('active', 'deleted') NOT NULL DEFAULT 'active',
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
    FOREIGN KEY (author_id) REFERENCES users(id),
    INDEX idx_author_id (author_id)
);