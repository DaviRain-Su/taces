use backend::{
    models::{
        payment::*,
        user::{CreateUserDto, UserRole},
    },
    utils::test_helpers::*,
};
use hyper::StatusCode;
use rust_decimal::Decimal;
use serde_json::json;
use std::str::FromStr;
use uuid::Uuid;

#[tokio::test]
async fn test_create_order() {
    let (app, state) = setup_test_app().await;
    let (admin_token, admin_user_id) = create_test_admin(&state).await;
    let (patient_token, patient_user_id) = create_test_patient(&state, "patient1").await;

    // Create order
    let order_dto = CreateOrderDto {
        user_id: patient_user_id,
        appointment_id: None,
        order_type: OrderType::Consultation,
        amount: Decimal::from_str("30.00").unwrap(),
        description: Some("图文咨询服务".to_string()),
        metadata: None,
    };

    let response = app
        .post("/api/v1/payment/orders")
        .json(&order_dto)
        .header("Authorization", format!("Bearer {}", patient_token))
        .await;

    assert_eq!(response.status(), StatusCode::CREATED);
    let body: serde_json::Value = response.json().await;
    assert!(body["success"].as_bool().unwrap());
    assert!(body["data"]["order_no"].as_str().is_some());
    assert_eq!(body["data"]["amount"].as_str().unwrap(), "30");
    assert_eq!(body["data"]["status"].as_str().unwrap(), "pending");

    cleanup_test_data(&state).await;
}

#[tokio::test]
async fn test_get_order() {
    let (app, state) = setup_test_app().await;
    let (patient_token, patient_user_id) = create_test_patient(&state, "patient1").await;

    // Create order first
    let order_dto = CreateOrderDto {
        user_id: patient_user_id,
        appointment_id: None,
        order_type: OrderType::Consultation,
        amount: Decimal::from_str("30.00").unwrap(),
        description: Some("图文咨询服务".to_string()),
        metadata: None,
    };

    let create_response = app
        .post("/api/v1/payment/orders")
        .json(&order_dto)
        .header("Authorization", format!("Bearer {}", patient_token))
        .await;

    let create_body: serde_json::Value = create_response.json().await;
    let order_id = create_body["data"]["id"].as_str().unwrap();

    // Get order
    let response = app
        .get(format!("/api/v1/payment/orders/{}", order_id))
        .header("Authorization", format!("Bearer {}", patient_token))
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = response.json().await;
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["data"]["id"].as_str().unwrap(), order_id);

    cleanup_test_data(&state).await;
}

#[tokio::test]
async fn test_list_orders() {
    let (app, state) = setup_test_app().await;
    let (patient_token, patient_user_id) = create_test_patient(&state, "patient1").await;

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

        app.post("/api/v1/payment/orders")
            .json(&order_dto)
            .header("Authorization", format!("Bearer {}", patient_token))
            .await;
    }

    // List orders
    let response = app
        .get("/api/v1/payment/orders")
        .header("Authorization", format!("Bearer {}", patient_token))
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = response.json().await;
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["data"]["orders"].as_array().unwrap().len(), 3);
    assert_eq!(body["data"]["total"].as_i64().unwrap(), 3);

    cleanup_test_data(&state).await;
}

#[tokio::test]
async fn test_cancel_order() {
    let (app, state) = setup_test_app().await;
    let (patient_token, patient_user_id) = create_test_patient(&state, "patient1").await;

    // Create order
    let order_dto = CreateOrderDto {
        user_id: patient_user_id,
        appointment_id: None,
        order_type: OrderType::Consultation,
        amount: Decimal::from_str("30.00").unwrap(),
        description: Some("图文咨询服务".to_string()),
        metadata: None,
    };

    let create_response = app
        .post("/api/v1/payment/orders")
        .json(&order_dto)
        .header("Authorization", format!("Bearer {}", patient_token))
        .await;

    let create_body: serde_json::Value = create_response.json().await;
    let order_id = create_body["data"]["id"].as_str().unwrap();

    // Cancel order
    let response = app
        .put(format!("/api/v1/payment/orders/{}/cancel", order_id))
        .header("Authorization", format!("Bearer {}", patient_token))
        .await;

    assert_eq!(response.status(), StatusCode::OK);

    // Verify order is cancelled
    let get_response = app
        .get(format!("/api/v1/payment/orders/{}", order_id))
        .header("Authorization", format!("Bearer {}", patient_token))
        .await;

    let get_body: serde_json::Value = get_response.json().await;
    assert_eq!(get_body["data"]["status"].as_str().unwrap(), "cancelled");

    cleanup_test_data(&state).await;
}

#[tokio::test]
async fn test_initiate_payment() {
    let (app, state) = setup_test_app().await;
    let (patient_token, patient_user_id) = create_test_patient(&state, "patient1").await;

    // Create order first
    let order_dto = CreateOrderDto {
        user_id: patient_user_id,
        appointment_id: None,
        order_type: OrderType::Consultation,
        amount: Decimal::from_str("30.00").unwrap(),
        description: Some("图文咨询服务".to_string()),
        metadata: None,
    };

    let create_response = app
        .post("/api/v1/payment/orders")
        .json(&order_dto)
        .header("Authorization", format!("Bearer {}", patient_token))
        .await;

    let create_body: serde_json::Value = create_response.json().await;
    let order_id = Uuid::parse_str(create_body["data"]["id"].as_str().unwrap()).unwrap();

    // Initiate payment
    let payment_dto = InitiatePaymentDto {
        order_id,
        payment_method: PaymentMethod::Alipay,
        return_url: Some("https://example.com/return".to_string()),
    };

    let response = app
        .post("/api/v1/payment/pay")
        .json(&payment_dto)
        .header("Authorization", format!("Bearer {}", patient_token))
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = response.json().await;
    assert!(body["success"].as_bool().unwrap());
    assert!(body["data"]["payment_url"].as_str().is_some());

    cleanup_test_data(&state).await;
}

#[tokio::test]
async fn test_get_price_config() {
    let (app, state) = setup_test_app().await;

    // Get price config for consultation
    let response = app
        .get("/api/v1/payment/prices/consultation")
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = response.json().await;
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["data"]["service_type"].as_str().unwrap(), "consultation");
    assert_eq!(body["data"]["price"].as_str().unwrap(), "20");

    cleanup_test_data(&state).await;
}

#[tokio::test]
async fn test_list_price_configs() {
    let (app, state) = setup_test_app().await;

    // List all price configs
    let response = app
        .get("/api/v1/payment/prices")
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = response.json().await;
    assert!(body["success"].as_bool().unwrap());
    let configs = body["data"].as_array().unwrap();
    assert!(configs.len() >= 4); // At least 4 default configs

    cleanup_test_data(&state).await;
}

#[tokio::test]
async fn test_create_refund() {
    let (app, state) = setup_test_app().await;
    let (patient_token, patient_user_id) = create_test_patient(&state, "patient1").await;

    // Create and pay an order first (simulated)
    let order_dto = CreateOrderDto {
        user_id: patient_user_id,
        appointment_id: None,
        order_type: OrderType::Consultation,
        amount: Decimal::from_str("30.00").unwrap(),
        description: Some("图文咨询服务".to_string()),
        metadata: None,
    };

    let create_response = app
        .post("/api/v1/payment/orders")
        .json(&order_dto)
        .header("Authorization", format!("Bearer {}", patient_token))
        .await;

    let create_body: serde_json::Value = create_response.json().await;
    let order_id = Uuid::parse_str(create_body["data"]["id"].as_str().unwrap()).unwrap();

    // Manually update order to paid status (simulating successful payment)
    sqlx::query!(
        "UPDATE payment_orders SET status = 'paid', payment_method = 'alipay', payment_time = NOW() WHERE id = ?",
        order_id
    )
    .execute(&state.pool)
    .await
    .unwrap();

    // Create a payment transaction record
    let transaction_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO payment_transactions (
            id, transaction_no, order_id, payment_method, 
            transaction_type, amount, status, initiated_at, completed_at
        ) VALUES (?, ?, ?, 'alipay', 'payment', ?, 'success', NOW(), NOW())
        "#,
        transaction_id,
        format!("TXN{}", chrono::Utc::now().timestamp()),
        order_id,
        order_dto.amount
    )
    .execute(&state.pool)
    .await
    .unwrap();

    // Create refund
    let refund_dto = CreateRefundDto {
        order_id,
        refund_amount: Decimal::from_str("30.00").unwrap(),
        refund_reason: "服务未提供".to_string(),
    };

    let response = app
        .post("/api/v1/payment/refunds")
        .json(&refund_dto)
        .header("Authorization", format!("Bearer {}", patient_token))
        .await;

    assert_eq!(response.status(), StatusCode::CREATED);
    let body: serde_json::Value = response.json().await;
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["data"]["status"].as_str().unwrap(), "pending");

    cleanup_test_data(&state).await;
}

#[tokio::test]
async fn test_get_user_balance() {
    let (app, state) = setup_test_app().await;
    let (patient_token, patient_user_id) = create_test_patient(&state, "patient1").await;

    // Create user balance first
    let balance_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO user_balances (
            id, user_id, balance, frozen_balance, 
            total_income, total_expense, created_at, updated_at
        ) VALUES (?, ?, 100.00, 0, 100.00, 0, NOW(), NOW())
        "#,
        balance_id,
        patient_user_id
    )
    .execute(&state.pool)
    .await
    .unwrap();

    // Get balance
    let response = app
        .get(format!("/api/v1/payment/balance/{}", patient_user_id))
        .header("Authorization", format!("Bearer {}", patient_token))
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = response.json().await;
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["data"]["balance"].as_str().unwrap(), "100");

    cleanup_test_data(&state).await;
}

#[tokio::test]
async fn test_admin_review_refund() {
    let (app, state) = setup_test_app().await;
    let (admin_token, admin_user_id) = create_test_admin(&state).await;
    let (patient_token, patient_user_id) = create_test_patient(&state, "patient1").await;

    // Create paid order and refund request
    let order_id = Uuid::new_v4();
    let order_no = format!("ORD{}", chrono::Utc::now().timestamp());
    
    sqlx::query!(
        r#"
        INSERT INTO payment_orders (
            id, order_no, user_id, order_type, amount, currency,
            status, payment_method, payment_time, expire_time, created_at, updated_at
        ) VALUES (?, ?, ?, 'consultation', 30.00, 'CNY', 'paid', 'alipay', NOW(), NOW() + INTERVAL 2 HOUR, NOW(), NOW())
        "#,
        order_id,
        order_no,
        patient_user_id
    )
    .execute(&state.pool)
    .await
    .unwrap();

    // Create transaction
    let transaction_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO payment_transactions (
            id, transaction_no, order_id, payment_method, 
            transaction_type, amount, status, initiated_at, completed_at
        ) VALUES (?, ?, ?, 'alipay', 'payment', 30.00, 'success', NOW(), NOW())
        "#,
        transaction_id,
        format!("TXN{}", chrono::Utc::now().timestamp()),
        order_id
    )
    .execute(&state.pool)
    .await
    .unwrap();

    // Create refund record
    let refund_id = Uuid::new_v4();
    sqlx::query!(
        r#"
        INSERT INTO refund_records (
            id, refund_no, order_id, transaction_id, user_id,
            refund_amount, refund_reason, status, created_at, updated_at
        ) VALUES (?, ?, ?, ?, ?, 30.00, '服务未提供', 'pending', NOW(), NOW())
        "#,
        refund_id,
        format!("RFD{}", chrono::Utc::now().timestamp()),
        order_id,
        transaction_id,
        patient_user_id
    )
    .execute(&state.pool)
    .await
    .unwrap();

    // Review refund as admin
    let review_dto = ReviewRefundDto {
        approved: true,
        review_notes: Some("同意退款".to_string()),
    };

    let response = app
        .put(format!("/api/v1/payment/admin/refunds/{}/review", refund_id))
        .json(&review_dto)
        .header("Authorization", format!("Bearer {}", admin_token))
        .await;

    assert_eq!(response.status(), StatusCode::OK);

    cleanup_test_data(&state).await;
}

#[tokio::test]
async fn test_payment_statistics() {
    let (app, state) = setup_test_app().await;
    let (patient_token, patient_user_id) = create_test_patient(&state, "patient1").await;

    // Create some orders with different statuses
    for i in 0..3 {
        let order_id = Uuid::new_v4();
        let status = match i {
            0 => "paid",
            1 => "paid",
            _ => "pending",
        };
        
        sqlx::query!(
            r#"
            INSERT INTO payment_orders (
                id, order_no, user_id, order_type, amount, currency,
                status, expire_time, created_at, updated_at
            ) VALUES (?, ?, ?, 'consultation', ?, 'CNY', ?, NOW() + INTERVAL 2 HOUR, NOW(), NOW())
            "#,
            order_id,
            format!("ORD{}{}", chrono::Utc::now().timestamp(), i),
            patient_user_id,
            Decimal::from(30 + i * 10),
            status
        )
        .execute(&state.pool)
        .await
        .unwrap();
    }

    // Get statistics
    let response = app
        .get("/api/v1/payment/statistics")
        .header("Authorization", format!("Bearer {}", patient_token))
        .await;

    assert_eq!(response.status(), StatusCode::OK);
    let body: serde_json::Value = response.json().await;
    assert!(body["success"].as_bool().unwrap());
    assert_eq!(body["data"]["total_orders"].as_i64().unwrap(), 3);
    assert_eq!(body["data"]["paid_orders"].as_i64().unwrap(), 2);

    cleanup_test_data(&state).await;
}

#[tokio::test]
async fn test_unauthorized_access() {
    let (app, state) = setup_test_app().await;
    let (patient_token, patient_user_id) = create_test_patient(&state, "patient1").await;
    let (other_token, other_user_id) = create_test_patient(&state, "patient2").await;

    // Create order for patient1
    let order_dto = CreateOrderDto {
        user_id: patient_user_id,
        appointment_id: None,
        order_type: OrderType::Consultation,
        amount: Decimal::from_str("30.00").unwrap(),
        description: Some("图文咨询服务".to_string()),
        metadata: None,
    };

    let create_response = app
        .post("/api/v1/payment/orders")
        .json(&order_dto)
        .header("Authorization", format!("Bearer {}", patient_token))
        .await;

    let create_body: serde_json::Value = create_response.json().await;
    let order_id = create_body["data"]["id"].as_str().unwrap();

    // Try to access with other user's token
    let response = app
        .get(format!("/api/v1/payment/orders/{}", order_id))
        .header("Authorization", format!("Bearer {}", other_token))
        .await;

    assert_eq!(response.status(), StatusCode::FORBIDDEN);

    cleanup_test_data(&state).await;
}