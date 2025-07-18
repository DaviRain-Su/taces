use sqlx::{MySql, MySqlPool, Pool};
use uuid::Uuid;
use crate::models::user::*;
use crate::utils::password::hash_password;

pub async fn create_test_pool() -> Pool<MySql> {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "mysql://tcm_user:tcm_pass123@localhost:3307/tcm_telemedicine_test".to_string());
    
    MySqlPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

pub async fn setup_test_db(pool: &Pool<MySql>) {
    // Clean up existing data
    sqlx::query("SET FOREIGN_KEY_CHECKS = 0")
        .execute(pool)
        .await
        .unwrap();
    
    let tables = vec!["prescriptions", "appointments", "doctors", "users"];
    for table in tables {
        sqlx::query(&format!("TRUNCATE TABLE {}", table))
            .execute(pool)
            .await
            .unwrap();
    }
    
    sqlx::query("SET FOREIGN_KEY_CHECKS = 1")
        .execute(pool)
        .await
        .unwrap();
}

pub async fn create_test_user(pool: &Pool<MySql>, role: &str) -> (Uuid, String, String) {
    let user_id = Uuid::new_v4();
    let account = format!("test_{}_{}", role, user_id.to_string().split('-').next().unwrap());
    let password = "test123456";
    let hashed_password = hash_password(password).unwrap();
    
    sqlx::query(r#"
        INSERT INTO users (id, account, name, password, gender, phone, email, role, status)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'active')
    "#)
        .bind(user_id.to_string())
        .bind(&account)
        .bind(format!("Test {} User", role))
        .bind(&hashed_password)
        .bind("男")
        .bind(format!("139{:08}", rand::random::<u32>() % 100000000))
        .bind(format!("{}@test.com", account))
        .bind(role)
        .execute(pool)
        .await
        .unwrap();
    
    (user_id, account, password.to_string())
}

pub async fn create_test_doctor(pool: &Pool<MySql>, user_id: Uuid) -> Uuid {
    let doctor_id = Uuid::new_v4();
    
    sqlx::query(r#"
        INSERT INTO doctors (id, user_id, certificate_type, id_number, hospital, department, 
                           title, introduction, specialties)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
    "#)
        .bind(doctor_id.to_string())
        .bind(user_id.to_string())
        .bind("医师资格证")
        .bind("110101199001011234")
        .bind("测试医院")
        .bind("中医科")
        .bind("主治医师")
        .bind("测试医生简介")
        .bind(r#"["中医内科", "针灸"]"#)
        .execute(pool)
        .await
        .unwrap();
    
    doctor_id
}