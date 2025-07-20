use crate::common::TestApp;
use axum::http::StatusCode;
use backend::{
    models::{
        payment::*,
        user::LoginDto,
    },
    utils::test_helpers::create_test_user,
};
use chrono;
use rust_decimal::Decimal;
use serde_json::{json, Value};
use sqlx;
use std::str::FromStr;
use uuid::Uuid;

async fn get_auth_token(app: &mut TestApp, account: &str, password: &str) -> String {
    let login_dto = LoginDto {
        account: account.to_string(),
        password: password.to_string(),
    };

    let (_, body) = app.post("/api/v1/auth/login", login_dto).await;
    body["data"]["token"].as_str().unwrap().to_string()
}

#[tokio::test]
async fn test_create_order() {
    let mut app = TestApp::new().await;
    let (patient_user_id, patient_account, patient_password) = create_test_user(&app.pool, "patient").await;
    let patient_token = get_auth_token(&mut app, &patient_account, &patient_password).await;

    // Create order
    let order_dto = CreateOrderDto {
        user_id: patient_user_id,
        appointment_id: None,
        order_type: OrderType::Consultation,
        amount: Decimal::from_str("30.00").unwrap(),
        description: Some("图文咨询服务".to_string()),
        metadata: None,
    };

    let (status, body) = app.post_with_auth("/api/v1/payment/orders", order_dto, &patient_token).await;

    assert_eq!(status, StatusCode::CREATED);
    assert!(body["success"].as_bool().unwrap());
    assert!(body["data"]["order_no"].as_str().is_some());
    assert_eq!(body["data"]["amount"].as_str().unwrap(), "30");
    assert_eq!(body["data"]["status"].as_str().unwrap(), "pending");
}

#[tokio::test]
async fn test_get_order() {
    let mut app = TestApp::new().await;
    let (patient_user_id, patient_account, patient_password) = create_test_user(&app.pool, "patient").await;
    let patient_token = get_auth_token(&mut app, &patient_account, &patient_password).await;

    // Create order first
    let order_dto = CreateOrderDto {
        user_id: patient_user_id,
        appointment_id: None,
        order_type: OrderType::Consultation,
        amount: Decimal::from_str("30.00").unwrap(),
        description: Some("图文咨询服务".to_string()),
        metadata: None,
    };

    let (_, create_body) = app.post_with_auth("/api/v1/payment/orders", order_dto, &patient_token).await;
    let order_id = create_body["data"]["id"].as_str().unwrap();

    // Get order
    let (status, body) = app.get_with_auth(&format!("/api/v1/payment/orders/{}", order_id), &patient_token).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["data"]["id"].as_str().unwrap(), order_id);
}

#[tokio::test]
async fn test_list_orders() {
    let mut app = TestApp::new().await;
    let (patient_user_id, patient_account, patient_password) = create_test_user(&app.pool, "patient").await;
    let patient_token = get_auth_token(&mut app, &patient_account, &patient_password).await;

    // Create multiple orders
    for i in 0..3 {
        let order_dto = CreateOrderDto {
            user_id: patient_user_id,
            appointment_id: None,
            order_type: OrderType::Consultation,
            amount: Decimal::from_str(&format!("{}.00", (i + 1) * 10)).unwrap(),
            description: Some(format!("订单 {}", i + 1)),
            metadata: None,
        };

        app.post_with_auth("/api/v1/payment/orders", order_dto, &patient_token).await;
    }

    // List orders
    let (status, body) = app.get_with_auth("/api/v1/payment/orders", &patient_token).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["data"]["orders"].as_array().unwrap().len(), 3);
    assert_eq!(body["data"]["total"].as_i64().unwrap(), 3);
}

#[tokio::test]
async fn test_cancel_order() {
    let mut app = TestApp::new().await;
    let (patient_user_id, patient_account, patient_password) = create_test_user(&app.pool, "patient").await;
    let patient_token = get_auth_token(&mut app, &patient_account, &patient_password).await;

    // Create order
    let order_dto = CreateOrderDto {
        user_id: patient_user_id,
        appointment_id: None,
        order_type: OrderType::Consultation,
        amount: Decimal::from_str("30.00").unwrap(),
        description: Some("图文咨询服务".to_string()),
        metadata: None,
    };

    let (_, create_body) = app.post_with_auth("/api/v1/payment/orders", order_dto, &patient_token).await;
    let order_id = create_body["data"]["id"].as_str().unwrap();

    // Cancel order
    let (status, _) = app.put_with_auth(&format!("/api/v1/payment/orders/{}/cancel", order_id), json!({}), &patient_token).await;
    assert_eq!(status, StatusCode::OK);

    // Verify order is cancelled
    let (_, get_body) = app.get_with_auth(&format!("/api/v1/payment/orders/{}", order_id), &patient_token).await;
    assert_eq!(get_body["data"]["status"].as_str().unwrap(), "cancelled");
}

#[tokio::test]
async fn test_initiate_payment() {
    let mut app = TestApp::new().await;
    let (patient_user_id, patient_account, patient_password) = create_test_user(&app.pool, "patient").await;
    let patient_token = get_auth_token(&mut app, &patient_account, &patient_password).await;

    // Create order first
    let order_dto = CreateOrderDto {
        user_id: patient_user_id,
        appointment_id: None,
        order_type: OrderType::Consultation,
        amount: Decimal::from_str("30.00").unwrap(),
        description: Some("图文咨询服务".to_string()),
        metadata: None,
    };

    let (_, create_body) = app.post_with_auth("/api/v1/payment/orders", order_dto, &patient_token).await;
    let order_id = Uuid::parse_str(create_body["data"]["id"].as_str().unwrap()).unwrap();

    // Initiate payment
    let payment_dto = InitiatePaymentDto {
        order_id,
        payment_method: PaymentMethod::Alipay,
        return_url: Some("https://example.com/return".to_string()),
    };

    let (status, body) = app.post_with_auth("/api/v1/payment/pay", payment_dto, &patient_token).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());
    assert!(body["data"]["payment_url"].as_str().is_some());
}

#[tokio::test]
async fn test_get_price_config() {
    let mut app = TestApp::new().await;

    // Get price config for consultation
    let (status, body) = app.get("/api/v1/payment/prices/consultation").await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["data"]["service_type"].as_str().unwrap(), "consultation");
    assert_eq!(body["data"]["price"].as_str().unwrap(), "20");
}

#[tokio::test]
async fn test_list_price_configs() {
    let mut app = TestApp::new().await;

    // List all price configs
    let (status, body) = app.get("/api/v1/payment/prices").await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());
    let configs = body["data"].as_array().unwrap();
    assert!(configs.len() >= 4); // At least 4 default configs
}

#[tokio::test]
async fn test_create_refund() {
    let mut app = TestApp::new().await;
    let (patient_user_id, patient_account, patient_password) = create_test_user(&app.pool, "patient").await;
    let patient_token = get_auth_token(&mut app, &patient_account, &patient_password).await;

    // Create and pay an order first (simulated)
    let amount = Decimal::from_str("30.00").unwrap();
    let order_dto = CreateOrderDto {
        user_id: patient_user_id,
        appointment_id: None,
        order_type: OrderType::Consultation,
        amount,
        description: Some("图文咨询服务".to_string()),
        metadata: None,
    };

    let (_, create_body) = app.post_with_auth("/api/v1/payment/orders", order_dto, &patient_token).await;
    let order_id = Uuid::parse_str(create_body["data"]["id"].as_str().unwrap()).unwrap();

    // Manually update order to paid status (simulating successful payment)
    sqlx::query(
        "UPDATE payment_orders SET status = 'paid', payment_method = 'alipay', payment_time = NOW() WHERE id = ?"
    )
    .bind(order_id)
    .execute(&app.pool)
    .await
    .unwrap();

    // Create a payment transaction record
    let transaction_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO payment_transactions (
            id, transaction_no, order_id, payment_method, 
            transaction_type, amount, status, initiated_at, completed_at
        ) VALUES (?, ?, ?, 'alipay', 'payment', ?, 'success', NOW(), NOW())
        "#
    )
    .bind(transaction_id)
    .bind(format!("TXN{}", chrono::Utc::now().timestamp()))
    .bind(order_id)
    .bind(amount)
    .execute(&app.pool)
    .await
    .unwrap();

    // Create refund
    let refund_dto = CreateRefundDto {
        order_id,
        refund_amount: Decimal::from_str("30.00").unwrap(),
        refund_reason: "服务未提供".to_string(),
    };

    let (status, body) = app.post_with_auth("/api/v1/payment/refunds", refund_dto, &patient_token).await;

    assert_eq!(status, StatusCode::CREATED);
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["data"]["status"].as_str().unwrap(), "pending");
}

#[tokio::test]
async fn test_get_user_balance() {
    let mut app = TestApp::new().await;
    let (patient_user_id, patient_account, patient_password) = create_test_user(&app.pool, "patient").await;
    let patient_token = get_auth_token(&mut app, &patient_account, &patient_password).await;

    // Create user balance first
    let balance_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO user_balances (
            id, user_id, balance, frozen_balance, 
            total_income, total_expense, created_at, updated_at
        ) VALUES (?, ?, 100.00, 0, 100.00, 0, NOW(), NOW())
        "#
    )
    .bind(balance_id)
    .bind(patient_user_id)
    .execute(&app.pool)
    .await
    .unwrap();

    // Get balance
    let (status, body) = app.get_with_auth(&format!("/api/v1/payment/balance/{}", patient_user_id), &patient_token).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["data"]["balance"].as_str().unwrap(), "100");
}

#[tokio::test]
async fn test_admin_review_refund() {
    let mut app = TestApp::new().await;
    let (_admin_user_id, admin_account, admin_password) = create_test_user(&app.pool, "admin").await;
    let admin_token = get_auth_token(&mut app, &admin_account, &admin_password).await;
    let (patient_user_id, patient_account, patient_password) = create_test_user(&app.pool, "patient").await;
    let _patient_token = get_auth_token(&mut app, &patient_account, &patient_password).await;

    // Create paid order and refund request
    let order_id = Uuid::new_v4();
    let order_no = format!("ORD{}", chrono::Utc::now().timestamp());
    
    sqlx::query(
        r#"
        INSERT INTO payment_orders (
            id, order_no, user_id, order_type, amount, currency,
            status, payment_method, payment_time, expire_time, created_at, updated_at
        ) VALUES (?, ?, ?, 'consultation', 30.00, 'CNY', 'paid', 'alipay', NOW(), DATE_ADD(NOW(), INTERVAL 2 HOUR), NOW(), NOW())
        "#
    )
    .bind(order_id)
    .bind(order_no)
    .bind(patient_user_id)
    .execute(&app.pool)
    .await
    .unwrap();

    // Create transaction
    let transaction_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO payment_transactions (
            id, transaction_no, order_id, payment_method, 
            transaction_type, amount, status, initiated_at, completed_at
        ) VALUES (?, ?, ?, 'alipay', 'payment', 30.00, 'success', NOW(), NOW())
        "#
    )
    .bind(transaction_id)
    .bind(format!("TXN{}", chrono::Utc::now().timestamp()))
    .bind(order_id)
    .execute(&app.pool)
    .await
    .unwrap();

    // Create refund record
    let refund_id = Uuid::new_v4();
    sqlx::query(
        r#"
        INSERT INTO refund_records (
            id, refund_no, order_id, transaction_id, user_id,
            refund_amount, refund_reason, status, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, 30.00, '服务未提供', 'pending', NOW(), NOW())
        "#
    )
    .bind(refund_id)
    .bind(format!("RFD{}", chrono::Utc::now().timestamp()))
    .bind(order_id)
    .bind(transaction_id)
    .bind(patient_user_id)
    .execute(&app.pool)
    .await
    .unwrap();

    // Review refund as admin
    let review_dto = ReviewRefundDto {
        approved: true,
        review_notes: Some("同意退款".to_string()),
    };

    let (status, _) = app.put_with_auth(&format!("/api/v1/payment/admin/refunds/{}/review", refund_id), review_dto, &admin_token).await;

    assert_eq!(status, StatusCode::OK);
}

#[tokio::test]
async fn test_payment_statistics() {
    let mut app = TestApp::new().await;
    let (patient_user_id, patient_account, patient_password) = create_test_user(&app.pool, "patient").await;
    let patient_token = get_auth_token(&mut app, &patient_account, &patient_password).await;

    // Create some orders with different statuses
    for i in 0..3 {
        let order_id = Uuid::new_v4();
        let status = match i {
            0 => "paid",
            1 => "paid",
            _ => "pending",
        };
        
        sqlx::query(
            r#"
            INSERT INTO payment_orders (
                id, order_no, user_id, order_type, amount, currency,
                status, expire_time, created_at, updated_at
            ) VALUES (?, ?, ?, 'consultation', ?, 'CNY', ?, DATE_ADD(NOW(), INTERVAL 2 HOUR), NOW(), NOW())
            "#
        )
        .bind(order_id)
        .bind(format!("ORD{}{}", chrono::Utc::now().timestamp(), i))
        .bind(patient_user_id)
        .bind(Decimal::from(30 + i * 10))
        .bind(status)
        .execute(&app.pool)
        .await
        .unwrap();
    }

    // Get statistics
    let (status, body) = app.get_with_auth("/api/v1/payment/statistics", &patient_token).await;

    assert_eq!(status, StatusCode::OK);
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["data"]["total_orders"].as_i64().unwrap(), 3);
    assert_eq!(body["data"]["paid_orders"].as_i64().unwrap(), 2);
}

#[tokio::test]
async fn test_unauthorized_access() {
    let mut app = TestApp::new().await;
    let (patient_user_id, patient_account, patient_password) = create_test_user(&app.pool, "patient").await;
    let patient_token = get_auth_token(&mut app, &patient_account, &patient_password).await;
    let (_other_user_id, other_account, other_password) = create_test_user(&app.pool, "patient2").await;
    let other_token = get_auth_token(&mut app, &other_account, &other_password).await;

    // Create order for patient1
    let order_dto = CreateOrderDto {
        user_id: patient_user_id,
        appointment_id: None,
        order_type: OrderType::Consultation,
        amount: Decimal::from_str("30.00").unwrap(),
        description: Some("图文咨询服务".to_string()),
        metadata: None,
    };

    let (_, create_body) = app.post_with_auth("/api/v1/payment/orders", order_dto, &patient_token).await;
    let order_id = create_body["data"]["id"].as_str().unwrap();

    // Try to access with other user's token
    let (status, _) = app.get_with_auth(&format!("/api/v1/payment/orders/{}", order_id), &other_token).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
}