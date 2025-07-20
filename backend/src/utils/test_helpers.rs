use crate::utils::password::hash_password;
use sqlx::{MySql, MySqlPool, Pool};
use uuid::Uuid;

pub async fn create_test_pool() -> Pool<MySql> {
    let database_url = std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
        "mysql://tcm_user:tcm_pass123@localhost:3307/tcm_telemedicine_test".to_string()
    });

    MySqlPool::connect(&database_url)
        .await
        .expect("Failed to connect to test database")
}

pub async fn setup_test_db(pool: &Pool<MySql>) {
    // Clean up existing data
    sqlx::query("DELETE FROM prescriptions")
        .execute(pool)
        .await
        .unwrap();
    // Delete from tables that reference video_consultations first
    sqlx::query("DELETE FROM video_call_events")
        .execute(pool)
        .await
        .unwrap_or_else(|_| Default::default()); // Ignore error if table doesn't exist
    sqlx::query("DELETE FROM video_recordings")
        .execute(pool)
        .await
        .unwrap_or_else(|_| Default::default()); // Ignore error if table doesn't exist
    sqlx::query("DELETE FROM webrtc_signals")
        .execute(pool)
        .await
        .unwrap_or_else(|_| Default::default()); // Ignore error if table doesn't exist
    sqlx::query("DELETE FROM file_uploads")
        .execute(pool)
        .await
        .unwrap_or_else(|_| Default::default()); // Ignore error if table doesn't exist
    sqlx::query("DELETE FROM video_consultations")
        .execute(pool)
        .await
        .unwrap_or_else(|_| Default::default()); // Ignore error if table doesn't exist
    sqlx::query("DELETE FROM review_replies")
        .execute(pool)
        .await
        .unwrap_or_else(|_| Default::default()); // Ignore error if table doesn't exist
    sqlx::query("DELETE FROM patient_reviews")
        .execute(pool)
        .await
        .unwrap_or_else(|_| Default::default()); // Ignore error if table doesn't exist
    sqlx::query("DELETE FROM appointments")
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM videos")
        .execute(pool)
        .await
        .unwrap_or_else(|_| Default::default()); // Ignore error if table doesn't exist
    sqlx::query("DELETE FROM articles")
        .execute(pool)
        .await
        .unwrap_or_else(|_| Default::default()); // Ignore error if table doesn't exist
    sqlx::query("DELETE FROM circle_posts")
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM post_comments")
        .execute(pool)
        .await
        .unwrap_or_else(|_| Default::default()); // Ignore error if table doesn't exist
    sqlx::query("DELETE FROM post_likes")
        .execute(pool)
        .await
        .unwrap_or_else(|_| Default::default()); // Ignore error if table doesn't exist
    sqlx::query("DELETE FROM circle_members")
        .execute(pool)
        .await
        .unwrap_or_else(|_| Default::default()); // Ignore error if table doesn't exist
    sqlx::query("DELETE FROM circles")
        .execute(pool)
        .await
        .unwrap_or_else(|_| Default::default()); // Ignore error if table doesn't exist
    sqlx::query("DELETE FROM prescription_templates")
        .execute(pool)
        .await
        .unwrap_or_else(|_| Default::default()); // Ignore error if table doesn't exist
    sqlx::query("DELETE FROM common_phrases")
        .execute(pool)
        .await
        .unwrap_or_else(|_| Default::default()); // Ignore error if table doesn't exist
    sqlx::query("DELETE FROM live_streams")
        .execute(pool)
        .await
        .unwrap_or_else(|_| Default::default()); // Ignore error if table doesn't exist
    sqlx::query("DELETE FROM patient_profiles")
        .execute(pool)
        .await
        .unwrap_or_else(|_| Default::default()); // Ignore error if table doesn't exist
    sqlx::query("DELETE FROM patient_group_members")
        .execute(pool)
        .await
        .unwrap_or_else(|_| Default::default()); // Ignore error if table doesn't exist
    sqlx::query("DELETE FROM patient_groups")
        .execute(pool)
        .await
        .unwrap_or_else(|_| Default::default()); // Ignore error if table doesn't exist
    sqlx::query("DELETE FROM video_consultation_templates")
        .execute(pool)
        .await
        .unwrap_or_else(|_| Default::default()); // Ignore error if table doesn't exist
    sqlx::query("DELETE FROM doctors")
        .execute(pool)
        .await
        .unwrap();
    sqlx::query("DELETE FROM departments")
        .execute(pool)
        .await
        .unwrap_or_else(|_| Default::default()); // Ignore error if table doesn't exist
    sqlx::query("DELETE FROM balance_transactions")
        .execute(pool)
        .await
        .unwrap_or_else(|_| Default::default()); // Ignore error if table doesn't exist
    sqlx::query("DELETE FROM refund_records")
        .execute(pool)
        .await
        .unwrap_or_else(|_| Default::default()); // Ignore error if table doesn't exist
    sqlx::query("DELETE FROM payment_transactions")
        .execute(pool)
        .await
        .unwrap_or_else(|_| Default::default()); // Ignore error if table doesn't exist
    sqlx::query("DELETE FROM payment_orders")
        .execute(pool)
        .await
        .unwrap_or_else(|_| Default::default()); // Ignore error if table doesn't exist
    sqlx::query("DELETE FROM user_balances")
        .execute(pool)
        .await
        .unwrap_or_else(|_| Default::default()); // Ignore error if table doesn't exist
    sqlx::query("DELETE FROM users")
        .execute(pool)
        .await
        .unwrap();
}

pub async fn create_test_user(pool: &Pool<MySql>, role: &str) -> (Uuid, String, String) {
    let user_id = Uuid::new_v4();
    let account = format!(
        "test_{}_{}",
        role,
        user_id.to_string().split('-').next().unwrap()
    );
    let password = "test123456";
    let hashed_password = hash_password(password).unwrap();

    sqlx::query(
        r#"
        INSERT INTO users (id, account, name, password, gender, phone, email, role, status)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, 'active')
    "#,
    )
    .bind(user_id.to_string())
    .bind(&account)
    .bind(format!("Test {} User", role))
    .bind(&hashed_password)
    .bind("男")
    .bind(format!(
        "139{:08}",
        10000000 + (user_id.as_u128() as u32 % 90000000)
    ))
    .bind(format!("{}@test.com", account))
    .bind(role)
    .execute(pool)
    .await
    .unwrap();

    (user_id, account, password.to_string())
}

pub async fn create_test_doctor(pool: &Pool<MySql>, user_id: Uuid) -> (Uuid, String) {
    let doctor_id = Uuid::new_v4();
    let department = "中医科";

    sqlx::query(
        r#"
        INSERT INTO doctors (id, user_id, certificate_type, id_number, hospital, department, 
                           title, introduction, specialties)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
    "#,
    )
    .bind(doctor_id.to_string())
    .bind(user_id.to_string())
    .bind("医师资格证")
    .bind("110101199001011234")
    .bind("测试医院")
    .bind(department)
    .bind("主治医师")
    .bind("测试医生简介")
    .bind(r#"["中医内科", "针灸"]"#)
    .execute(pool)
    .await
    .unwrap();

    (doctor_id, department.to_string())
}
