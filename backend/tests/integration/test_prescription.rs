use crate::common::TestApp;
use axum::http::StatusCode;
use backend::{
    models::{prescription::*, user::LoginDto},
    utils::test_helpers::{create_test_doctor, create_test_user},
};

async fn get_auth_token(app: &mut TestApp, account: &str, password: &str) -> String {
    let login_dto = LoginDto {
        account: account.to_string(),
        password: password.to_string(),
    };

    let (_, body) = app.post("/api/v1/auth/login", login_dto).await;
    body["data"]["token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_create_prescription() {
    let mut app = TestApp::new().await;

    // Create doctor and patient
    let (doctor_user_id, doctor_account, doctor_password) =
        create_test_user(&app.pool, "doctor").await;
    let (doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;
    let (patient_user_id, _, _) = create_test_user(&app.pool, "patient").await;

    let doctor_token = get_auth_token(&mut app, &doctor_account, &doctor_password).await;

    // Create prescription
    let prescription_dto = CreatePrescriptionDto {
        doctor_id,
        patient_id: patient_user_id,
        patient_name: "测试患者".to_string(),
        diagnosis: "风寒感冒".to_string(),
        medicines: vec![
            Medicine {
                name: "板蓝根颗粒".to_string(),
                dosage: "10g".to_string(),
                frequency: "每日3次".to_string(),
                duration: "3天".to_string(),
                notes: Some("开水冲服".to_string()),
            },
            Medicine {
                name: "感冒清热颗粒".to_string(),
                dosage: "12g".to_string(),
                frequency: "每日2次".to_string(),
                duration: "3天".to_string(),
                notes: Some("饭后服用".to_string()),
            },
        ],
        instructions: "多喝温水，注意休息，避免受凉".to_string(),
    };

    let (status, body) = app
        .post_with_auth("/api/v1/prescriptions", prescription_dto, &doctor_token)
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["doctor_id"], doctor_id.to_string());
    assert_eq!(body["data"]["patient_id"], patient_user_id.to_string());
    assert_eq!(body["data"]["diagnosis"], "风寒感冒");
    assert!(body["data"]["code"].as_str().unwrap().starts_with("RX"));
    assert_eq!(body["data"]["medicines"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_list_prescriptions() {
    let mut app = TestApp::new().await;

    // Create admin user
    let (_, admin_account, admin_password) = create_test_user(&app.pool, "admin").await;
    let admin_token = get_auth_token(&mut app, &admin_account, &admin_password).await;

    // Create doctor and patient
    let (doctor_user_id, _, _) = create_test_user(&app.pool, "doctor").await;
    let (doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;
    let (patient_user_id, _, _) = create_test_user(&app.pool, "patient").await;

    // Create multiple prescriptions
    for i in 0..3 {
        let prescription_dto = CreatePrescriptionDto {
            doctor_id,
            patient_id: patient_user_id,
            patient_name: format!("患者{}", i + 1),
            diagnosis: format!("诊断{}", i + 1),
            medicines: vec![Medicine {
                name: format!("药品{}", i + 1),
                dosage: "10g".to_string(),
                frequency: "每日3次".to_string(),
                duration: "3天".to_string(),
                notes: None,
            }],
            instructions: "".to_string(),
        };

        let _ = app
            .post_with_auth("/api/v1/prescriptions", prescription_dto, &admin_token)
            .await;
    }

    // List prescriptions (admin only)
    let (status, body) = app
        .get_with_auth("/api/v1/prescriptions?page=1&page_size=10", &admin_token)
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert!(body["data"]["items"].is_array());
    assert!(body["data"]["items"].as_array().unwrap().len() >= 3);
}

#[tokio::test]
async fn test_get_prescription_by_id() {
    let mut app = TestApp::new().await;

    // Create doctor and patient
    let (doctor_user_id, doctor_account, doctor_password) =
        create_test_user(&app.pool, "doctor").await;
    let (doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;
    let (patient_user_id, patient_account, patient_password) =
        create_test_user(&app.pool, "patient").await;

    let doctor_token = get_auth_token(&mut app, &doctor_account, &doctor_password).await;
    let patient_token = get_auth_token(&mut app, &patient_account, &patient_password).await;

    // Create prescription
    let prescription_dto = CreatePrescriptionDto {
        doctor_id,
        patient_id: patient_user_id,
        patient_name: "测试患者".to_string(),
        diagnosis: "测试诊断".to_string(),
        medicines: vec![Medicine {
            name: "测试药品".to_string(),
            dosage: "10g".to_string(),
            frequency: "每日3次".to_string(),
            duration: "3天".to_string(),
            notes: None,
        }],
        instructions: "".to_string(),
    };

    let (_, create_body) = app
        .post_with_auth("/api/v1/prescriptions", prescription_dto, &doctor_token)
        .await;

    let prescription_id = create_body["data"]["id"].as_str().unwrap();

    // Get prescription by ID (doctor can access)
    let (status, body) = app
        .get_with_auth(
            &format!("/api/v1/prescriptions/{}", prescription_id),
            &doctor_token,
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["id"], prescription_id);

    // Get prescription by ID (patient can access their own)
    let (status, body) = app
        .get_with_auth(
            &format!("/api/v1/prescriptions/{}", prescription_id),
            &patient_token,
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["id"], prescription_id);
}

#[tokio::test]
async fn test_get_prescription_by_code() {
    let mut app = TestApp::new().await;

    // Create doctor and patient
    let (doctor_user_id, doctor_account, doctor_password) =
        create_test_user(&app.pool, "doctor").await;
    let (doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;
    let (patient_user_id, patient_account, patient_password) =
        create_test_user(&app.pool, "patient").await;

    let doctor_token = get_auth_token(&mut app, &doctor_account, &doctor_password).await;
    let patient_token = get_auth_token(&mut app, &patient_account, &patient_password).await;

    // Create prescription
    let prescription_dto = CreatePrescriptionDto {
        doctor_id,
        patient_id: patient_user_id,
        patient_name: "测试患者".to_string(),
        diagnosis: "测试诊断".to_string(),
        medicines: vec![Medicine {
            name: "测试药品".to_string(),
            dosage: "10g".to_string(),
            frequency: "每日3次".to_string(),
            duration: "3天".to_string(),
            notes: None,
        }],
        instructions: "".to_string(),
    };

    let (_, create_body) = app
        .post_with_auth("/api/v1/prescriptions", prescription_dto, &doctor_token)
        .await;

    let prescription_code = create_body["data"]["code"].as_str().unwrap();

    // Get prescription by code (patient can access)
    let (status, body) = app
        .get_with_auth(
            &format!("/api/v1/prescriptions/code/{}", prescription_code),
            &patient_token,
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["code"], prescription_code);
}

#[tokio::test]
async fn test_get_doctor_prescriptions() {
    let mut app = TestApp::new().await;

    // Create doctor and patient
    let (doctor_user_id, doctor_account, doctor_password) =
        create_test_user(&app.pool, "doctor").await;
    let (doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;
    let (patient_user_id, _, _) = create_test_user(&app.pool, "patient").await;

    let doctor_token = get_auth_token(&mut app, &doctor_account, &doctor_password).await;

    // Create multiple prescriptions for the doctor
    for i in 0..3 {
        let prescription_dto = CreatePrescriptionDto {
            doctor_id,
            patient_id: patient_user_id,
            patient_name: format!("患者{}", i + 1),
            diagnosis: format!("诊断{}", i + 1),
            medicines: vec![Medicine {
                name: format!("药品{}", i + 1),
                dosage: "10g".to_string(),
                frequency: "每日3次".to_string(),
                duration: "3天".to_string(),
                notes: None,
            }],
            instructions: "".to_string(),
        };

        let _ = app
            .post_with_auth("/api/v1/prescriptions", prescription_dto, &doctor_token)
            .await;
    }

    // Get doctor's prescriptions
    let (status, body) = app
        .get_with_auth(
            &format!(
                "/api/v1/prescriptions/doctor/{}?page=1&page_size=10",
                doctor_id
            ),
            &doctor_token,
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert!(body["data"]["items"].is_array());
    assert_eq!(body["data"]["items"].as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn test_get_patient_prescriptions() {
    let mut app = TestApp::new().await;

    // Create doctor and patient
    let (doctor_user_id, _, _) = create_test_user(&app.pool, "doctor").await;
    let (doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;
    let (patient_user_id, patient_account, patient_password) =
        create_test_user(&app.pool, "patient").await;

    let patient_token = get_auth_token(&mut app, &patient_account, &patient_password).await;

    // Create prescriptions for the patient
    for i in 0..2 {
        let prescription_dto = CreatePrescriptionDto {
            doctor_id,
            patient_id: patient_user_id,
            patient_name: "测试患者".to_string(),
            diagnosis: format!("诊断{}", i + 1),
            medicines: vec![Medicine {
                name: format!("药品{}", i + 1),
                dosage: "10g".to_string(),
                frequency: "每日3次".to_string(),
                duration: "3天".to_string(),
                notes: None,
            }],
            instructions: "".to_string(),
        };

        let _ = app
            .post_with_auth("/api/v1/prescriptions", prescription_dto, &patient_token)
            .await;
    }

    // Get patient's prescriptions
    let (status, body) = app
        .get_with_auth(
            &format!(
                "/api/v1/prescriptions/patient/{}?page=1&page_size=10",
                patient_user_id
            ),
            &patient_token,
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert!(body["data"]["items"].is_array());
    assert_eq!(body["data"]["items"].as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn test_prescription_authorization() {
    let mut app = TestApp::new().await;

    // Create two doctors and a patient
    let (doctor1_user_id, doctor1_account, doctor1_password) =
        create_test_user(&app.pool, "doctor").await;
    let (doctor1_id, _) = create_test_doctor(&app.pool, doctor1_user_id).await;
    let (doctor2_user_id, doctor2_account, doctor2_password) =
        create_test_user(&app.pool, "doctor").await;
    let (_doctor2_id, _) = create_test_doctor(&app.pool, doctor2_user_id).await;
    let (patient_user_id, _, _) = create_test_user(&app.pool, "patient").await;

    let doctor1_token = get_auth_token(&mut app, &doctor1_account, &doctor1_password).await;
    let doctor2_token = get_auth_token(&mut app, &doctor2_account, &doctor2_password).await;

    // Doctor 1 creates a prescription
    let prescription_dto = CreatePrescriptionDto {
        doctor_id: doctor1_id,
        patient_id: patient_user_id,
        patient_name: "测试患者".to_string(),
        diagnosis: "测试诊断".to_string(),
        medicines: vec![Medicine {
            name: "测试药品".to_string(),
            dosage: "10g".to_string(),
            frequency: "每日3次".to_string(),
            duration: "3天".to_string(),
            notes: None,
        }],
        instructions: "".to_string(),
    };

    let (_, create_body) = app
        .post_with_auth("/api/v1/prescriptions", prescription_dto, &doctor1_token)
        .await;

    let prescription_id = create_body["data"]["id"].as_str().unwrap();

    // Doctor 2 tries to access doctor 1's prescription (should succeed - doctors can view all prescriptions)
    let (status, body) = app
        .get_with_auth(
            &format!("/api/v1/prescriptions/{}", prescription_id),
            &doctor2_token,
        )
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
}

#[tokio::test]
async fn test_patient_cannot_create_prescription() {
    let mut app = TestApp::new().await;

    // Create doctor and patient
    let (doctor_user_id, _, _) = create_test_user(&app.pool, "doctor").await;
    let (doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;
    let (patient_user_id, patient_account, patient_password) =
        create_test_user(&app.pool, "patient").await;

    let patient_token = get_auth_token(&mut app, &patient_account, &patient_password).await;

    // Patient tries to create a prescription (should fail)
    let prescription_dto = CreatePrescriptionDto {
        doctor_id,
        patient_id: patient_user_id,
        patient_name: "测试患者".to_string(),
        diagnosis: "测试诊断".to_string(),
        medicines: vec![Medicine {
            name: "测试药品".to_string(),
            dosage: "10g".to_string(),
            frequency: "每日3次".to_string(),
            duration: "3天".to_string(),
            notes: None,
        }],
        instructions: "".to_string(),
    };

    let (status, body) = app
        .post_with_auth("/api/v1/prescriptions", prescription_dto, &patient_token)
        .await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["success"], false);
}

#[tokio::test]
async fn test_prescription_with_multiple_medicines() {
    let mut app = TestApp::new().await;

    // Create doctor and patient
    let (doctor_user_id, doctor_account, doctor_password) =
        create_test_user(&app.pool, "doctor").await;
    let (doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;
    let (patient_user_id, _, _) = create_test_user(&app.pool, "patient").await;

    let doctor_token = get_auth_token(&mut app, &doctor_account, &doctor_password).await;

    // Create prescription with multiple medicines
    let prescription_dto = CreatePrescriptionDto {
        doctor_id,
        patient_id: patient_user_id,
        patient_name: "复方测试患者".to_string(),
        diagnosis: "肝肾阴虚，虚火上炎".to_string(),
        medicines: vec![
            Medicine {
                name: "六味地黄丸".to_string(),
                dosage: "9g".to_string(),
                frequency: "每日2次".to_string(),
                duration: "30天".to_string(),
                notes: Some("空腹温水送服".to_string()),
            },
            Medicine {
                name: "知柏地黄丸".to_string(),
                dosage: "9g".to_string(),
                frequency: "每日2次".to_string(),
                duration: "30天".to_string(),
                notes: Some("饭后温水送服".to_string()),
            },
            Medicine {
                name: "龙胆泻肝丸".to_string(),
                dosage: "6g".to_string(),
                frequency: "每日3次".to_string(),
                duration: "7天".to_string(),
                notes: Some("饭后服用".to_string()),
            },
            Medicine {
                name: "黄连上清片".to_string(),
                dosage: "4片".to_string(),
                frequency: "每日3次".to_string(),
                duration: "5天".to_string(),
                notes: Some("含服或吞服".to_string()),
            },
        ],
        instructions: "忌辛辣油腻，保持心情舒畅，规律作息".to_string(),
    };

    let (status, body) = app
        .post_with_auth("/api/v1/prescriptions", prescription_dto, &doctor_token)
        .await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["success"], true);
    assert_eq!(body["data"]["medicines"].as_array().unwrap().len(), 4);
    assert_eq!(body["data"]["diagnosis"], "肝肾阴虚，虚火上炎");
}

#[tokio::test]
async fn test_prescription_code_uniqueness() {
    let mut app = TestApp::new().await;

    // Create doctor and patient
    let (doctor_user_id, doctor_account, doctor_password) =
        create_test_user(&app.pool, "doctor").await;
    let (doctor_id, _) = create_test_doctor(&app.pool, doctor_user_id).await;
    let (patient_user_id, _, _) = create_test_user(&app.pool, "patient").await;

    let doctor_token = get_auth_token(&mut app, &doctor_account, &doctor_password).await;

    // Create multiple prescriptions
    let mut prescription_codes = Vec::new();
    for i in 0..5 {
        let prescription_dto = CreatePrescriptionDto {
            doctor_id,
            patient_id: patient_user_id,
            patient_name: format!("患者{}", i + 1),
            diagnosis: "测试诊断".to_string(),
            medicines: vec![Medicine {
                name: "测试药品".to_string(),
                dosage: "10g".to_string(),
                frequency: "每日3次".to_string(),
                duration: "3天".to_string(),
                notes: None,
            }],
            instructions: "".to_string(),
        };

        let (_, body) = app
            .post_with_auth("/api/v1/prescriptions", prescription_dto, &doctor_token)
            .await;

        let code = body["data"]["code"].as_str().unwrap().to_string();
        assert!(
            !prescription_codes.contains(&code),
            "Prescription code should be unique"
        );
        prescription_codes.push(code);
    }

    assert_eq!(prescription_codes.len(), 5);
}
