use backend::{
    config::{Config, database},
    models::{user::*, doctor::*, appointment::*, prescription::*},
    utils::password::hash_password,
};
use chrono::{Utc, Duration};
use uuid::Uuid;
use dotenv::dotenv;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();
    
    println!("Starting database seeding...");
    
    let config = Config::from_env()?;
    let pool = database::create_pool().await?;
    
    // Clear existing data
    println!("Clearing existing data...");
    sqlx::query("SET FOREIGN_KEY_CHECKS = 0").execute(&pool).await?;
    sqlx::query("TRUNCATE TABLE prescriptions").execute(&pool).await?;
    sqlx::query("TRUNCATE TABLE appointments").execute(&pool).await?;
    sqlx::query("TRUNCATE TABLE doctors").execute(&pool).await?;
    sqlx::query("TRUNCATE TABLE users").execute(&pool).await?;
    sqlx::query("SET FOREIGN_KEY_CHECKS = 1").execute(&pool).await?;
    
    // Create admin user
    let admin_id = Uuid::new_v4();
    let admin_password = hash_password("admin123")?;
    println!("Creating admin user...");
    sqlx::query(r#"
        INSERT INTO users (id, account, name, password, gender, phone, email, role, status)
        VALUES (?, 'admin', '系统管理员', ?, '男', '13800000000', 'admin@tcm.com', 'admin', 'active')
    "#)
        .bind(admin_id.to_string())
        .bind(&admin_password)
        .execute(&pool)
        .await?;
    
    // Create doctor users
    let doctor1_id = Uuid::new_v4();
    let doctor1_password = hash_password("doctor123")?;
    println!("Creating doctor users...");
    sqlx::query(r#"
        INSERT INTO users (id, account, name, password, gender, phone, email, role, status)
        VALUES (?, 'doctor_dong', '董老师', ?, '男', '13900000001', 'dong@tcm.com', 'doctor', 'active')
    "#)
        .bind(doctor1_id.to_string())
        .bind(&doctor1_password)
        .execute(&pool)
        .await?;
    
    let doctor2_id = Uuid::new_v4();
    let doctor2_password = hash_password("doctor123")?;
    sqlx::query(r#"
        INSERT INTO users (id, account, name, password, gender, phone, email, role, status)
        VALUES (?, 'doctor_wang', '王医生', ?, '女', '13900000002', 'wang@tcm.com', 'doctor', 'active')
    "#)
        .bind(doctor2_id.to_string())
        .bind(&doctor2_password)
        .execute(&pool)
        .await?;
    
    // Create doctor profiles
    let doctor1_profile_id = Uuid::new_v4();
    println!("Creating doctor profiles...");
    sqlx::query(r#"
        INSERT INTO doctors (id, user_id, certificate_type, id_number, hospital, department, 
                           title, introduction, specialties, experience)
        VALUES (?, ?, '医师资格证', '110101199001011234', '香河香草中医诊所', '中医内科', 
                '主任医师', '30余年中医临床经验，擅长治疗各类慢性病', '["慢性病调理", "中医内科", "针灸推拿"]', 
                '从医30余年，治愈患者数万例')
    "#)
        .bind(doctor1_profile_id.to_string())
        .bind(doctor1_id.to_string())
        .execute(&pool)
        .await?;
    
    let doctor2_profile_id = Uuid::new_v4();
    sqlx::query(r#"
        INSERT INTO doctors (id, user_id, certificate_type, id_number, hospital, department, 
                           title, introduction, specialties, experience)
        VALUES (?, ?, '医师资格证', '110101199001012345', '香河香草中医诊所', '针灸推拿科', 
                '副主任医师', '专注针灸推拿治疗，对颈肩腰腿痛有独到见解', '["针灸", "推拿", "康复理疗"]', 
                '从医15年，精通各类针灸手法')
    "#)
        .bind(doctor2_profile_id.to_string())
        .bind(doctor2_id.to_string())
        .execute(&pool)
        .await?;
    
    // Create patient users
    let mut patient_ids = Vec::new();
    println!("Creating patient users...");
    for i in 1..=10 {
        let patient_id = Uuid::new_v4();
        let patient_password = hash_password("patient123")?;
        sqlx::query(r#"
            INSERT INTO users (id, account, name, password, gender, phone, email, role, status)
            VALUES (?, ?, ?, ?, ?, ?, ?, 'patient', 'active')
        "#)
            .bind(patient_id.to_string())
            .bind(format!("patient{}", i))
            .bind(format!("患者{}", i))
            .bind(&patient_password)
            .bind(if i % 2 == 0 { "女" } else { "男" })
            .bind(format!("138{:08}", i))
            .bind(format!("patient{}@example.com", i))
            .execute(&pool)
            .await?;
        
        patient_ids.push(patient_id);
    }
    
    // Create appointments
    println!("Creating appointments...");
    let now = Utc::now();
    let appointment_statuses = ["pending", "confirmed", "completed", "cancelled"];
    let visit_types = ["online_video", "offline"];
    let time_slots = ["09:00", "09:30", "10:00", "10:30", "14:00", "14:30", "15:00", "15:30"];
    
    for i in 0..20 {
        let appointment_id = Uuid::new_v4();
        let patient_idx = i % patient_ids.len();
        let doctor_profile_id = if i % 2 == 0 { doctor1_profile_id } else { doctor2_profile_id };
        let appointment_date = now + Duration::days((i / 4) as i64);
        let status = appointment_statuses[i % appointment_statuses.len()];
        let visit_type = visit_types[i % visit_types.len()];
        let time_slot = time_slots[i % time_slots.len()];
        
        sqlx::query(r#"
            INSERT INTO appointments (id, patient_id, doctor_id, appointment_date, time_slot, 
                                    visit_type, symptoms, has_visited_before, status)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#)
            .bind(appointment_id.to_string())
            .bind(patient_ids[patient_idx].to_string())
            .bind(doctor_profile_id.to_string())
            .bind(appointment_date)
            .bind(time_slot)
            .bind(visit_type)
            .bind(format!("症状描述{}: 头痛、失眠、食欲不振", i + 1))
            .bind(i % 3 == 0)
            .bind(status)
            .execute(&pool)
            .await?;
    }
    
    // Create prescriptions for completed appointments
    println!("Creating prescriptions...");
    let completed_appointments = sqlx::query(r#"
        SELECT a.id, a.patient_id, a.doctor_id, u.name as patient_name
        FROM appointments a
        JOIN users u ON a.patient_id = u.id
        WHERE a.status = 'completed'
    "#)
        .fetch_all(&pool)
        .await?;
    
    for (i, row) in completed_appointments.iter().enumerate() {
        let prescription_id = Uuid::new_v4();
        let code = format!("RX2024{:04}", i + 1);
        let patient_id: String = sqlx::Row::get(row, "patient_id");
        let doctor_id: String = sqlx::Row::get(row, "doctor_id");
        let patient_name: String = sqlx::Row::get(row, "patient_name");
        
        let medicines = r#"[
            {"name": "当归", "dosage": "10g", "frequency": "每日3次", "duration": "7天"},
            {"name": "黄芪", "dosage": "15g", "frequency": "每日3次", "duration": "7天"},
            {"name": "党参", "dosage": "10g", "frequency": "每日2次", "duration": "14天"}
        ]"#;
        
        sqlx::query(r#"
            INSERT INTO prescriptions (id, code, doctor_id, patient_id, patient_name, 
                                     diagnosis, medicines, instructions, prescription_date)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#)
            .bind(prescription_id.to_string())
            .bind(&code)
            .bind(&doctor_id)
            .bind(&patient_id)
            .bind(&patient_name)
            .bind(format!("中医诊断{}: 气血不足，脾胃虚弱", i + 1))
            .bind(medicines)
            .bind("饭后温水送服，忌辛辣生冷")
            .bind(now - Duration::days(i as i64))
            .execute(&pool)
            .await?;
    }
    
    println!("Database seeding completed successfully!");
    println!("\nTest accounts:");
    println!("  Admin: admin / admin123");
    println!("  Doctor: doctor_dong / doctor123");
    println!("  Doctor: doctor_wang / doctor123");
    println!("  Patients: patient1-10 / patient123");
    
    Ok(())
}