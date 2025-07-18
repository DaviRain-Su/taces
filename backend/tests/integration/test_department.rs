use crate::common::TestApp;
use backend::{
    utils::test_helpers::create_test_user,
    models::user::LoginDto,
};
use axum::http::StatusCode;
use serde_json::json;

async fn get_auth_token(app: &mut TestApp, account: &str, password: &str) -> String {
    let login_dto = LoginDto {
        account: account.to_string(),
        password: password.to_string(),
    };
    
    let (status, body) = app.post("/api/v1/auth/login", login_dto).await;
    assert_eq!(status, StatusCode::OK, "Login failed: {:?}", body);
    body["data"]["token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_department_crud() {
    let mut app = TestApp::new().await;
    
    // Create admin user and get token
    let (_admin_id, admin_account, admin_password) = create_test_user(&app.pool, "admin").await;
    let admin_token = get_auth_token(&mut app, &admin_account, &admin_password).await;

    // Create department
    let create_dto = json!({
        "name": "中医科",
        "code": "ZY001",
        "contact_person": "张医生",
        "contact_phone": "13800138000",
        "description": "中医内科诊疗"
    });

    let (status, body) = app.post_with_auth("/api/v1/departments", create_dto, &admin_token).await;
    println!("Create department response: status={:?}, body={:?}", status, body);
    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());
    let department_id = body["data"]["id"].as_str().unwrap();

    // List departments (public endpoint)
    let (status, body) = app.get("/api/v1/departments").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());
    assert!(body["data"].as_array().unwrap().len() > 0);

    // Get department by ID (public endpoint)
    let (status, body) = app.get(&format!("/api/v1/departments/{}", department_id)).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["data"]["code"].as_str().unwrap(), "ZY001");

    // Get department by code (public endpoint)
    let (status, body) = app.get("/api/v1/departments/code/ZY001").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["data"]["name"].as_str().unwrap(), "中医科");

    // Update department
    let update_dto = json!({
        "name": "中医内科",
        "status": "inactive"
    });

    let (status, body) = app.put_with_auth(
        &format!("/api/v1/departments/{}", department_id),
        update_dto,
        &admin_token
    ).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["data"]["name"].as_str().unwrap(), "中医内科");
    assert_eq!(body["data"]["status"].as_str().unwrap(), "inactive");

    // Delete department
    let (status, body) = app.delete_with_auth(
        &format!("/api/v1/departments/{}", department_id),
        &admin_token
    ).await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());

    // Verify deletion
    let (status, _body) = app.get(&format!("/api/v1/departments/{}", department_id)).await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_department_permissions() {
    let mut app = TestApp::new().await;
    
    // Create users and get tokens
    let (_doctor_id, doctor_account, doctor_password) = create_test_user(&app.pool, "doctor").await;
    let (_patient_id, patient_account, patient_password) = create_test_user(&app.pool, "patient").await;
    
    let doctor_token = get_auth_token(&mut app, &doctor_account, &doctor_password).await;
    let patient_token = get_auth_token(&mut app, &patient_account, &patient_password).await;

    let create_dto = json!({
        "name": "针灸推拿科",
        "code": "ZJTN001",
        "contact_person": "李医生",
        "contact_phone": "13900139000",
        "description": "针灸推拿治疗"
    });

    // Doctor cannot create department
    let (status, _body) = app.post_with_auth("/api/v1/departments", &create_dto, &doctor_token).await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Patient cannot create department
    let (status, _body) = app.post_with_auth("/api/v1/departments", &create_dto, &patient_token).await;
    assert_eq!(status, StatusCode::FORBIDDEN);

    // Unauthenticated user cannot create department
    let (status, _body) = app.post("/api/v1/departments", &create_dto).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_department_duplicate_code() {
    let mut app = TestApp::new().await;
    
    // Create admin user and get token
    let (_admin_id, admin_account, admin_password) = create_test_user(&app.pool, "admin").await;
    let admin_token = get_auth_token(&mut app, &admin_account, &admin_password).await;

    let create_dto = json!({
        "name": "康复科",
        "code": "KF001",
        "contact_person": "王医生",
        "contact_phone": "13700137000",
        "description": "康复治疗"
    });

    // Create first department
    let (status, _body) = app.post_with_auth("/api/v1/departments", &create_dto, &admin_token).await;
    assert_eq!(status, StatusCode::OK);

    // Try to create department with same code
    let duplicate_dto = json!({
        "name": "另一个康复科",
        "code": "KF001",
        "contact_person": "赵医生",
        "contact_phone": "13600136000",
        "description": "另一个康复治疗"
    });

    let (status, body) = app.post_with_auth("/api/v1/departments", &duplicate_dto, &admin_token).await;
    assert_eq!(status, StatusCode::CONFLICT);
    assert!(!body["success"].as_bool().unwrap());
    assert_eq!(body["message"].as_str().unwrap(), "Department code already exists");
}

#[tokio::test]
async fn test_department_search_and_filter() {
    let mut app = TestApp::new().await;
    
    // Create admin user and get token
    let (_admin_id, admin_account, admin_password) = create_test_user(&app.pool, "admin").await;
    let admin_token = get_auth_token(&mut app, &admin_account, &admin_password).await;

    // Create multiple departments
    let departments = vec![
        ("中医内科", "ZYNK001"),
        ("中医外科", "ZYWK001"),
        ("针灸科", "ZJ001"),
    ];

    for (name, code) in &departments {
        let create_dto = json!({
            "name": name,
            "code": code,
            "contact_person": "测试医生",
            "contact_phone": "13800138000",
            "description": "测试科室"
        });

        app.post_with_auth("/api/v1/departments", create_dto, &admin_token).await;
    }

    // Search by name
    let (status, body) = app.get("/api/v1/departments?search=中医").await;
    assert_eq!(status, StatusCode::OK);
    let data = body["data"].as_array().unwrap();
    assert!(data.len() >= 2); // Should find at least "中医内科" and "中医外科"

    // Filter by status
    let (status, body) = app.get("/api/v1/departments?status=active").await;
    assert_eq!(status, StatusCode::OK);
    let data = body["data"].as_array().unwrap();
    assert!(data.len() >= 3); // At least the three departments we created

    // Pagination
    let (status, body) = app.get("/api/v1/departments?page=1&per_page=1").await;
    assert_eq!(status, StatusCode::OK);
    let data = body["data"].as_array().unwrap();
    assert_eq!(data.len(), 1); // Should return only 1 department per page
}