use crate::common::TestApp;
use axum::http::StatusCode;
use backend::{
    models::user::LoginDto,
    utils::test_helpers::{create_test_user, create_test_doctor},
};
use serde_json::json;

async fn get_doctor_auth_token(app: &mut TestApp) -> (String, String) {
    // 创建医生用户
    let (user_id, account, password) = create_test_user(&app.pool, "doctor").await;
    let (_doctor_id, _department) = create_test_doctor(&app.pool, user_id).await;
    
    // 登录获取token
    let login_dto = LoginDto {
        account: account.clone(),
        password,
    };
    
    let (status, body) = app.post("/api/v1/auth/login", login_dto).await;
    assert_eq!(status, StatusCode::OK);
    
    let token = body["data"]["token"].as_str().unwrap().to_string();
    (token, account)
}

#[tokio::test]
async fn test_common_phrase_crud() {
    let mut app = TestApp::new().await;
    
    // 获取医生token
    let (token, _) = get_doctor_auth_token(&mut app).await;
    
    // 1. 创建常用语
    let create_dto = json!({
        "category": "diagnosis",
        "content": "风寒感冒，症见恶寒重、发热轻"
    });
    
    let (status, body) = app
        .post_with_auth("/api/v1/templates/common-phrases", create_dto, &token)
        .await;
    if status != StatusCode::OK {
        println!("Create common phrase failed: {:?}", body);
    }
    assert_eq!(status, StatusCode::OK);
    let phrase_id = body["data"]["id"].as_str().unwrap();
    assert_eq!(body["data"]["content"].as_str().unwrap(), "风寒感冒，症见恶寒重、发热轻");
    assert_eq!(body["data"]["usage_count"].as_i64().unwrap(), 0);
    
    // 2. 获取常用语列表
    let (status, body) = app
        .get_with_auth("/api/v1/templates/common-phrases", &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    let phrases = body["data"]["phrases"].as_array().unwrap();
    assert!(phrases.len() >= 1); // 包含刚创建的和种子数据
    
    // 3. 根据分类筛选
    let (status, body) = app
        .get_with_auth("/api/v1/templates/common-phrases?category=diagnosis", &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    let phrases = body["data"]["phrases"].as_array().unwrap();
    assert!(phrases.iter().all(|p| p["category"].as_str().unwrap() == "diagnosis"));
    
    // 4. 获取单个常用语
    let (status, body) = app
        .get_with_auth(&format!("/api/v1/templates/common-phrases/{}", phrase_id), &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["id"].as_str().unwrap(), phrase_id);
    
    // 5. 更新常用语
    let update_dto = json!({
        "content": "风寒感冒，症见恶寒重、发热轻、无汗、头痛"
    });
    
    let (status, body) = app
        .put_with_auth(&format!("/api/v1/templates/common-phrases/{}", phrase_id), update_dto, &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["content"].as_str().unwrap(), "风寒感冒，症见恶寒重、发热轻、无汗、头痛");
    
    // 6. 使用常用语（增加使用次数）
    let (status, _) = app
        .post_with_auth(&format!("/api/v1/templates/common-phrases/{}/use", phrase_id), json!({}), &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    
    // 验证使用次数增加
    let (_, body) = app
        .get_with_auth(&format!("/api/v1/templates/common-phrases/{}", phrase_id), &token)
        .await;
    assert_eq!(body["data"]["usage_count"].as_i64().unwrap(), 1);
    
    // 7. 删除常用语
    let (status, _) = app
        .delete_with_auth(&format!("/api/v1/templates/common-phrases/{}", phrase_id), &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    
    // 验证删除后无法获取
    let (status, _) = app
        .get_with_auth(&format!("/api/v1/templates/common-phrases/{}", phrase_id), &token)
        .await;
    assert_eq!(status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_prescription_template_crud() {
    let mut app = TestApp::new().await;
    
    // 获取医生token
    let (token, _) = get_doctor_auth_token(&mut app).await;
    
    // 1. 创建处方模板
    let create_dto = json!({
        "name": "感冒方剂",
        "description": "用于治疗风寒感冒",
        "diagnosis": "风寒感冒",
        "medicines": [
            {
                "name": "感冒清热颗粒",
                "specification": "12g*10袋",
                "dosage": "1袋",
                "frequency": "一日3次",
                "duration": "3天",
                "usage": "开水冲服"
            }
        ],
        "instructions": "饭后服用，多饮水"
    });
    
    let (status, body) = app
        .post_with_auth("/api/v1/templates/prescription-templates", create_dto, &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    let template_id = body["data"]["id"].as_str().unwrap();
    assert_eq!(body["data"]["name"].as_str().unwrap(), "感冒方剂");
    assert_eq!(body["data"]["medicines"].as_array().unwrap().len(), 1);
    
    // 2. 获取处方模板列表
    let (status, body) = app
        .get_with_auth("/api/v1/templates/prescription-templates", &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    let templates = body["data"]["templates"].as_array().unwrap();
    assert!(templates.len() >= 1);
    
    // 3. 搜索处方模板
    let (status, body) = app
        .get_with_auth("/api/v1/templates/prescription-templates?search=感冒", &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    let templates = body["data"]["templates"].as_array().unwrap();
    assert!(templates.iter().any(|t| 
        t["name"].as_str().unwrap().contains("感冒") || 
        t["diagnosis"].as_str().unwrap().contains("感冒")
    ));
    
    // 4. 获取单个处方模板
    let (status, body) = app
        .get_with_auth(&format!("/api/v1/templates/prescription-templates/{}", template_id), &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["id"].as_str().unwrap(), template_id);
    
    // 5. 更新处方模板
    let update_dto = json!({
        "name": "风寒感冒方剂",
        "medicines": [
            {
                "name": "感冒清热颗粒",
                "specification": "12g*10袋",
                "dosage": "1袋",
                "frequency": "一日3次",
                "duration": "3天",
                "usage": "开水冲服"
            },
            {
                "name": "板蓝根颗粒",
                "specification": "10g*20袋",
                "dosage": "1袋",
                "frequency": "一日3次",
                "duration": "3天",
                "usage": "开水冲服"
            }
        ]
    });
    
    let (status, body) = app
        .put_with_auth(&format!("/api/v1/templates/prescription-templates/{}", template_id), update_dto, &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["data"]["name"].as_str().unwrap(), "风寒感冒方剂");
    assert_eq!(body["data"]["medicines"].as_array().unwrap().len(), 2);
    
    // 6. 使用处方模板
    let (status, _) = app
        .post_with_auth(&format!("/api/v1/templates/prescription-templates/{}/use", template_id), json!({}), &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    
    // 验证使用次数增加
    let (_, body) = app
        .get_with_auth(&format!("/api/v1/templates/prescription-templates/{}", template_id), &token)
        .await;
    assert_eq!(body["data"]["usage_count"].as_i64().unwrap(), 1);
    
    // 7. 删除处方模板
    let (status, _) = app
        .delete_with_auth(&format!("/api/v1/templates/prescription-templates/{}", template_id), &token)
        .await;
    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_common_phrase_categories() {
    let mut app = TestApp::new().await;
    
    let (token, _) = get_doctor_auth_token(&mut app).await;
    
    // 测试所有分类
    let categories = vec!["diagnosis", "advice", "symptom"];
    
    for category in categories {
        let create_dto = json!({
            "category": category,
            "content": format!("测试{}", category)
        });
        
        let (status, body) = app
            .post_with_auth("/api/v1/templates/common-phrases", create_dto, &token)
            .await;
        assert_eq!(status, StatusCode::OK);
        assert_eq!(body["data"]["category"].as_str().unwrap(), category);
    }
    
    // 测试无效分类
    let invalid_dto = json!({
        "category": "invalid",
        "content": "测试内容"
    });
    
    let (status, _) = app
        .post_with_auth("/api/v1/templates/common-phrases", invalid_dto, &token)
        .await;
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
}

#[tokio::test]
async fn test_template_permissions() {
    let mut app = TestApp::new().await;
    
    // 创建两个医生
    let (token1, _) = get_doctor_auth_token(&mut app).await;
    let (token2, _) = get_doctor_auth_token(&mut app).await;
    
    // 患者token
    let (_patient_id, patient_account, patient_password) = create_test_user(&app.pool, "patient").await;
    let patient_login = LoginDto {
        account: patient_account,
        password: patient_password,
    };
    let (_, patient_body) = app.post("/api/v1/auth/login", patient_login).await;
    let patient_token = patient_body["data"]["token"].as_str().unwrap();
    
    // 医生1创建常用语
    let create_dto = json!({
        "category": "diagnosis",
        "content": "测试诊断"
    });
    
    let (status, body) = app
        .post_with_auth("/api/v1/templates/common-phrases", create_dto, &token1)
        .await;
    assert_eq!(status, StatusCode::OK);
    let phrase_id = body["data"]["id"].as_str().unwrap();
    
    // 医生2不能更新医生1的常用语
    let update_dto = json!({
        "content": "修改内容"
    });
    
    let (status, _) = app
        .put_with_auth(&format!("/api/v1/templates/common-phrases/{}", phrase_id), update_dto, &token2)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    
    // 医生2不能删除医生1的常用语
    let (status, _) = app
        .delete_with_auth(&format!("/api/v1/templates/common-phrases/{}", phrase_id), &token2)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    
    // 患者不能创建常用语
    let patient_create_dto = json!({
        "category": "diagnosis",
        "content": "患者测试"
    });
    let (status, _) = app
        .post_with_auth("/api/v1/templates/common-phrases", patient_create_dto, &patient_token)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    
    // 患者不能查看常用语
    let (status, _) = app
        .get_with_auth("/api/v1/templates/common-phrases", &patient_token)
        .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
}

#[tokio::test]
async fn test_template_search_and_filter() {
    let mut app = TestApp::new().await;
    
    let (token, _) = get_doctor_auth_token(&mut app).await;
    
    // 创建多个常用语用于测试
    let phrases = vec![
        ("diagnosis", "风寒感冒诊断"),
        ("diagnosis", "风热感冒诊断"),
        ("advice", "多饮水休息"),
        ("symptom", "头痛发热"),
    ];
    
    for (category, content) in phrases {
        let dto = json!({
            "category": category,
            "content": content
        });
        app.post_with_auth("/api/v1/templates/common-phrases", dto, &token).await;
    }
    
    // 测试关键词搜索
    let (status, body) = app
        .get_with_auth("/api/v1/templates/common-phrases?search=感冒", &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    let results = body["data"]["phrases"].as_array().unwrap();
    assert!(results.iter().all(|p| p["content"].as_str().unwrap().contains("感冒")));
    
    // 测试分类筛选
    let (status, body) = app
        .get_with_auth("/api/v1/templates/common-phrases?category=advice", &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    let results = body["data"]["phrases"].as_array().unwrap();
    assert!(results.iter().all(|p| p["category"].as_str().unwrap() == "advice"));
    
    // 测试分页
    let (status, body) = app
        .get_with_auth("/api/v1/templates/common-phrases?page=1&page_size=2", &token)
        .await;
    assert_eq!(status, StatusCode::OK);
    let results = body["data"]["phrases"].as_array().unwrap();
    assert!(results.len() <= 2);
    assert_eq!(body["data"]["pagination"]["page"].as_i64().unwrap(), 1);
    assert_eq!(body["data"]["pagination"]["page_size"].as_i64().unwrap(), 2);
}