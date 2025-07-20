use crate::config::database::DbPool;
use crate::models::payment::*;
use crate::utils::errors::AppError;
use chrono::{Duration, Utc};
use rust_decimal::Decimal;
use sqlx::{MySql, Transaction};
use std::collections::HashMap;
use uuid::Uuid;

pub struct PaymentService;

impl PaymentService {
    // Order management
    pub async fn create_order(
        db: &DbPool,
        create_dto: CreateOrderDto,
    ) -> Result<PaymentOrder, AppError> {
        let order_id = Uuid::new_v4();
        let order_no = Self::generate_order_no();
        let now = Utc::now();
        let expire_time = now + Duration::hours(2); // 2 hour expiration

        let query = r#"
            INSERT INTO payment_orders (
                id, order_no, user_id, appointment_id, order_type,
                amount, currency, status, expire_time, description,
                metadata, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, 'CNY', 'pending', ?, ?, ?, ?, ?)
        "#;

        sqlx::query(query)
            .bind(order_id.to_string())
            .bind(&order_no)
            .bind(create_dto.user_id.to_string())
            .bind(create_dto.appointment_id.map(|id| id.to_string()))
            .bind(&create_dto.order_type)
            .bind(&create_dto.amount)
            .bind(&expire_time)
            .bind(create_dto.description.as_deref())
            .bind(&create_dto.metadata)
            .bind(&now)
            .bind(&now)
            .execute(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Self::get_order(db, order_id).await
    }

    pub async fn get_order(db: &DbPool, order_id: Uuid) -> Result<PaymentOrder, AppError> {
        let query = r#"
            SELECT * FROM payment_orders WHERE id = ?
        "#;

        sqlx::query_as::<_, PaymentOrder>(query)
            .bind(&order_id)
            .fetch_one(db)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => AppError::NotFound("订单不存在".to_string()),
                _ => AppError::DatabaseError(e.to_string()),
            })
    }

    pub async fn get_order_by_no(db: &DbPool, order_no: &str) -> Result<PaymentOrder, AppError> {
        let query = r#"
            SELECT * FROM payment_orders WHERE order_no = ?
        "#;

        sqlx::query_as::<_, PaymentOrder>(query)
            .bind(order_no)
            .fetch_one(db)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => AppError::NotFound("订单不存在".to_string()),
                _ => AppError::DatabaseError(e.to_string()),
            })
    }

    pub async fn list_orders(
        db: &DbPool,
        query: OrderListQuery,
    ) -> Result<OrderListResponse, AppError> {
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).min(100);
        let offset = (page - 1) * page_size;

        let mut where_clauses = vec![];

        if query.user_id.is_some() {
            where_clauses.push("user_id = ?");
        }

        if query.status.is_some() {
            where_clauses.push("status = ?");
        }

        if query.order_type.is_some() {
            where_clauses.push("order_type = ?");
        }

        if query.start_date.is_some() {
            where_clauses.push("created_at >= ?");
        }

        if query.end_date.is_some() {
            where_clauses.push("created_at <= ?");
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        // Count total
        let count_query = format!(
            "SELECT COUNT(*) as count FROM payment_orders {}",
            where_clause
        );

        let mut count_query_builder = sqlx::query_scalar::<_, i64>(&count_query);

        // Bind parameters in order
        if let Some(user_id) = &query.user_id {
            count_query_builder = count_query_builder.bind(user_id.to_string());
        }
        if let Some(status) = &query.status {
            count_query_builder = count_query_builder.bind(status);
        }
        if let Some(order_type) = &query.order_type {
            count_query_builder = count_query_builder.bind(order_type);
        }
        if let Some(start_date) = &query.start_date {
            count_query_builder = count_query_builder.bind(start_date);
        }
        if let Some(end_date) = &query.end_date {
            count_query_builder = count_query_builder.bind(end_date);
        }

        let total = count_query_builder
            .fetch_one(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Fetch orders
        let orders_query = format!(
            "SELECT * FROM payment_orders {} ORDER BY created_at DESC LIMIT ? OFFSET ?",
            where_clause
        );

        let mut orders_query_builder = sqlx::query_as::<_, PaymentOrder>(&orders_query);

        // Bind parameters in the same order
        if let Some(user_id) = &query.user_id {
            orders_query_builder = orders_query_builder.bind(user_id.to_string());
        }
        if let Some(status) = &query.status {
            orders_query_builder = orders_query_builder.bind(status);
        }
        if let Some(order_type) = &query.order_type {
            orders_query_builder = orders_query_builder.bind(order_type);
        }
        if let Some(start_date) = &query.start_date {
            orders_query_builder = orders_query_builder.bind(start_date);
        }
        if let Some(end_date) = &query.end_date {
            orders_query_builder = orders_query_builder.bind(end_date);
        }

        let orders = orders_query_builder
            .bind(page_size)
            .bind(offset)
            .fetch_all(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(OrderListResponse {
            orders,
            total,
            page,
            page_size,
        })
    }

    pub async fn cancel_order(db: &DbPool, order_id: Uuid) -> Result<(), AppError> {
        let order = Self::get_order(db, order_id).await?;

        if order.status != OrderStatus::Pending {
            return Err(AppError::BadRequest("只能取消待支付的订单".to_string()));
        }

        let query = r#"
            UPDATE payment_orders
            SET status = 'cancelled', updated_at = ?
            WHERE id = ? AND status = 'pending'
        "#;

        let result = sqlx::query(query)
            .bind(&Utc::now())
            .bind(&order_id)
            .execute(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AppError::BadRequest("订单状态已变更".to_string()));
        }

        Ok(())
    }

    // Payment processing
    pub async fn initiate_payment(
        db: &DbPool,
        dto: InitiatePaymentDto,
    ) -> Result<PaymentResponse, AppError> {
        let order = Self::get_order(db, dto.order_id).await?;

        if order.status != OrderStatus::Pending {
            return Err(AppError::BadRequest("订单状态不正确".to_string()));
        }

        if Utc::now() > order.expire_time {
            return Err(AppError::BadRequest("订单已过期".to_string()));
        }

        // Create transaction record
        let transaction_id = Uuid::new_v4();
        let transaction_no = Self::generate_transaction_no();

        let query = r#"
            INSERT INTO payment_transactions (
                id, transaction_no, order_id, payment_method,
                transaction_type, amount, status, initiated_at
            ) VALUES (?, ?, ?, ?, 'payment', ?, 'pending', ?)
        "#;

        sqlx::query(query)
            .bind(&transaction_id)
            .bind(&transaction_no)
            .bind(&order.id)
            .bind(&dto.payment_method)
            .bind(&order.amount)
            .bind(&Utc::now())
            .execute(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Process payment based on method
        match dto.payment_method {
            PaymentMethod::Wechat => {
                Self::process_wechat_payment(db, &order, &transaction_id).await
            }
            PaymentMethod::Alipay => {
                Self::process_alipay_payment(db, &order, &transaction_id, dto.return_url).await
            }
            PaymentMethod::Balance => {
                Self::process_balance_payment(db, &order, &transaction_id).await
            }
            _ => Err(AppError::BadRequest("不支持的支付方式".to_string())),
        }
    }

    async fn process_wechat_payment(
        db: &DbPool,
        order: &PaymentOrder,
        transaction_id: &Uuid,
    ) -> Result<PaymentResponse, AppError> {
        // Get WeChat Pay configuration
        let config = Self::get_payment_config(db, PaymentMethod::Wechat).await?;

        // TODO: Implement actual WeChat Pay API integration
        // For now, return mock data
        let prepay_id = format!("wx_prepay_{}", Uuid::new_v4());

        // Update transaction with prepay_id
        let query = r#"
            UPDATE payment_transactions
            SET prepay_id = ?, request_data = ?
            WHERE id = ?
        "#;

        let request_data = serde_json::json!({
            "appid": config.get("app_id"),
            "mchid": config.get("mch_id"),
            "out_trade_no": order.order_no,
            "amount": order.amount,
        });

        sqlx::query(query)
            .bind(&prepay_id)
            .bind(&request_data)
            .bind(transaction_id)
            .execute(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(PaymentResponse {
            order_id: order.id,
            order_no: order.order_no.clone(),
            payment_method: PaymentMethod::Wechat,
            payment_url: None,
            qr_code: Some(format!("wxp://f2f0{}", prepay_id)), // Mock QR code
            prepay_data: Some(serde_json::json!({
                "prepay_id": prepay_id,
                "appid": config.get("app_id"),
                "timestamp": Utc::now().timestamp(),
                "nonce_str": Uuid::new_v4().to_string(),
            })),
        })
    }

    async fn process_alipay_payment(
        db: &DbPool,
        order: &PaymentOrder,
        transaction_id: &Uuid,
        return_url: Option<String>,
    ) -> Result<PaymentResponse, AppError> {
        // Get Alipay configuration
        let config = Self::get_payment_config(db, PaymentMethod::Alipay).await?;

        // TODO: Implement actual Alipay API integration
        // For now, return mock data
        let trade_no = format!("alipay_{}", Uuid::new_v4());

        // Update transaction
        let query = r#"
            UPDATE payment_transactions
            SET trade_no = ?, request_data = ?
            WHERE id = ?
        "#;

        let request_data = serde_json::json!({
            "app_id": config.get("app_id"),
            "out_trade_no": order.order_no,
            "total_amount": order.amount,
            "return_url": return_url,
        });

        sqlx::query(query)
            .bind(&trade_no)
            .bind(&request_data)
            .bind(transaction_id)
            .execute(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(PaymentResponse {
            order_id: order.id,
            order_no: order.order_no.clone(),
            payment_method: PaymentMethod::Alipay,
            payment_url: Some(format!(
                "https://openapi.alipay.com/gateway.do?trade_no={}",
                trade_no
            )),
            qr_code: None,
            prepay_data: None,
        })
    }

    async fn process_balance_payment(
        db: &DbPool,
        order: &PaymentOrder,
        transaction_id: &Uuid,
    ) -> Result<PaymentResponse, AppError> {
        let mut tx = db
            .begin()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Check user balance
        let balance = Self::get_user_balance_tx(&mut tx, order.user_id).await?;

        if balance.balance < order.amount {
            return Err(AppError::BadRequest("余额不足".to_string()));
        }

        // Deduct balance
        Self::update_balance_tx(
            &mut tx,
            order.user_id,
            BalanceTransactionType::Expense,
            order.amount,
            Some("order".to_string()),
            Some(order.id),
            &format!("订单支付: {}", order.order_no),
        )
        .await?;

        // Update transaction status
        let query = r#"
            UPDATE payment_transactions
            SET status = 'success', completed_at = ?
            WHERE id = ?
        "#;

        sqlx::query(query)
            .bind(&Utc::now())
            .bind(transaction_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Update order status
        let query = r#"
            UPDATE payment_orders
            SET status = 'paid', payment_method = 'balance', payment_time = ?, updated_at = ?
            WHERE id = ?
        "#;

        let now = Utc::now();
        sqlx::query(query)
            .bind(&now)
            .bind(&now)
            .bind(&order.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Update appointment status if applicable
        if let Some(appointment_id) = order.appointment_id {
            let query = r#"
                UPDATE appointments
                SET status = 'confirmed', updated_at = ?
                WHERE id = ?
            "#;

            sqlx::query(query)
                .bind(&now)
                .bind(&appointment_id)
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        }

        tx.commit()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(PaymentResponse {
            order_id: order.id,
            order_no: order.order_no.clone(),
            payment_method: PaymentMethod::Balance,
            payment_url: None,
            qr_code: None,
            prepay_data: None,
        })
    }

    // Payment callback handling
    pub async fn handle_payment_callback(
        db: &DbPool,
        payment_method: PaymentMethod,
        callback_data: PaymentCallbackData,
    ) -> Result<(), AppError> {
        let mut tx = db
            .begin()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Get order and transaction
        let order = Self::get_order_by_no(db, &callback_data.order_no).await?;

        let query = r#"
            SELECT * FROM payment_transactions
            WHERE order_id = ? AND payment_method = ? AND status = 'pending'
            ORDER BY initiated_at DESC LIMIT 1
        "#;

        let transaction = sqlx::query_as::<_, PaymentTransaction>(query)
            .bind(&order.id)
            .bind(&payment_method)
            .fetch_one(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Update transaction
        let query = r#"
            UPDATE payment_transactions
            SET status = ?, external_transaction_id = ?, 
                callback_data = ?, completed_at = ?
            WHERE id = ?
        "#;

        let status = if callback_data.status == "success" {
            TransactionStatus::Success
        } else {
            TransactionStatus::Failed
        };

        sqlx::query(query)
            .bind(&status)
            .bind(&callback_data.external_transaction_id)
            .bind(&callback_data.raw_data)
            .bind(&Utc::now())
            .bind(&transaction.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Update order if payment successful
        if status == TransactionStatus::Success {
            let query = r#"
                UPDATE payment_orders
                SET status = 'paid', payment_method = ?, payment_time = ?, updated_at = ?
                WHERE id = ?
            "#;

            sqlx::query(query)
                .bind(&payment_method)
                .bind(&callback_data.payment_time)
                .bind(&Utc::now())
                .bind(&order.id)
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            // Update appointment status if applicable
            if let Some(appointment_id) = order.appointment_id {
                let query = r#"
                    UPDATE appointments
                    SET status = 'confirmed', updated_at = ?
                    WHERE id = ?
                "#;

                sqlx::query(query)
                    .bind(&Utc::now())
                    .bind(&appointment_id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            }
        }

        tx.commit()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    // Refund management
    pub async fn create_refund(
        db: &DbPool,
        dto: CreateRefundDto,
        user_id: Uuid,
    ) -> Result<RefundRecord, AppError> {
        let order = Self::get_order(db, dto.order_id).await?;

        // Validate order status
        if order.status != OrderStatus::Paid {
            return Err(AppError::BadRequest("只能退款已支付的订单".to_string()));
        }

        // Validate refund amount
        if dto.refund_amount > order.amount {
            return Err(AppError::BadRequest("退款金额不能大于订单金额".to_string()));
        }

        // Get the successful transaction
        let query = r#"
            SELECT * FROM payment_transactions
            WHERE order_id = ? AND transaction_type = 'payment' AND status = 'success'
            ORDER BY completed_at DESC LIMIT 1
        "#;

        let transaction = sqlx::query_as::<_, PaymentTransaction>(query)
            .bind(&order.id)
            .fetch_one(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Create refund record
        let refund_id = Uuid::new_v4();
        let refund_no = Self::generate_refund_no();

        let query = r#"
            INSERT INTO refund_records (
                id, refund_no, order_id, transaction_id, user_id,
                refund_amount, refund_reason, status, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, 'pending', ?, ?)
        "#;

        let now = Utc::now();
        sqlx::query(query)
            .bind(&refund_id)
            .bind(&refund_no)
            .bind(&order.id)
            .bind(&transaction.id)
            .bind(&user_id)
            .bind(&dto.refund_amount)
            .bind(&dto.refund_reason)
            .bind(&now)
            .bind(&now)
            .execute(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Self::get_refund(db, refund_id).await
    }

    pub async fn get_refund(db: &DbPool, refund_id: Uuid) -> Result<RefundRecord, AppError> {
        let query = r#"
            SELECT * FROM refund_records WHERE id = ?
        "#;

        sqlx::query_as::<_, RefundRecord>(query)
            .bind(&refund_id)
            .fetch_one(db)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => AppError::NotFound("退款记录不存在".to_string()),
                _ => AppError::DatabaseError(e.to_string()),
            })
    }

    pub async fn review_refund(
        db: &DbPool,
        refund_id: Uuid,
        dto: ReviewRefundDto,
        reviewer_id: Uuid,
    ) -> Result<(), AppError> {
        let refund = Self::get_refund(db, refund_id).await?;

        if refund.status != RefundStatus::Pending {
            return Err(AppError::BadRequest("退款申请已处理".to_string()));
        }

        if dto.approved {
            // Process refund
            Self::process_refund(db, &refund, reviewer_id, dto.review_notes).await
        } else {
            // Reject refund
            let query = r#"
                UPDATE refund_records
                SET status = 'cancelled', reviewed_by = ?, reviewed_at = ?,
                    review_notes = ?, updated_at = ?
                WHERE id = ?
            "#;

            let now = Utc::now();
            sqlx::query(query)
                .bind(&reviewer_id)
                .bind(&now)
                .bind(&dto.review_notes)
                .bind(&now)
                .bind(&refund_id)
                .execute(db)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;

            Ok(())
        }
    }

    async fn process_refund(
        db: &DbPool,
        refund: &RefundRecord,
        reviewer_id: Uuid,
        review_notes: Option<String>,
    ) -> Result<(), AppError> {
        let mut tx = db
            .begin()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Update refund status to processing
        let query = r#"
            UPDATE refund_records
            SET status = 'processing', reviewed_by = ?, reviewed_at = ?,
                review_notes = ?, updated_at = ?
            WHERE id = ?
        "#;

        let now = Utc::now();
        sqlx::query(query)
            .bind(&reviewer_id)
            .bind(&now)
            .bind(&review_notes)
            .bind(&now)
            .bind(&refund.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Get original order and transaction
        let order = Self::get_order(db, refund.order_id).await?;
        let transaction = Self::get_transaction(db, refund.transaction_id).await?;

        // Process refund based on payment method
        match transaction.payment_method {
            PaymentMethod::Balance => {
                // Refund to balance
                Self::update_balance_tx(
                    &mut tx,
                    refund.user_id,
                    BalanceTransactionType::Income,
                    refund.refund_amount,
                    Some("refund".to_string()),
                    Some(refund.id),
                    &format!("退款: {}", refund.refund_no),
                )
                .await?;

                // Update refund status
                let query = r#"
                    UPDATE refund_records
                    SET status = 'success', completed_at = ?, updated_at = ?
                    WHERE id = ?
                "#;

                sqlx::query(query)
                    .bind(&now)
                    .bind(&now)
                    .bind(&refund.id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            }
            _ => {
                // TODO: Implement third-party refund API calls
                // For now, just mark as success
                let query = r#"
                    UPDATE refund_records
                    SET status = 'success', completed_at = ?, updated_at = ?
                    WHERE id = ?
                "#;

                sqlx::query(query)
                    .bind(&now)
                    .bind(&now)
                    .bind(&refund.id)
                    .execute(&mut *tx)
                    .await
                    .map_err(|e| AppError::DatabaseError(e.to_string()))?;
            }
        }

        // Update order status
        let new_status = if refund.refund_amount == order.amount {
            OrderStatus::Refunded
        } else {
            OrderStatus::PartialRefunded
        };

        let query = r#"
            UPDATE payment_orders
            SET status = ?, updated_at = ?
            WHERE id = ?
        "#;

        sqlx::query(query)
            .bind(&new_status)
            .bind(&now)
            .bind(&order.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Create refund transaction record
        let refund_transaction_id = Uuid::new_v4();
        let refund_transaction_no = Self::generate_transaction_no();

        let query = r#"
            INSERT INTO payment_transactions (
                id, transaction_no, order_id, payment_method,
                transaction_type, amount, status, initiated_at, completed_at
            ) VALUES (?, ?, ?, ?, 'refund', ?, 'success', ?, ?)
        "#;

        sqlx::query(query)
            .bind(&refund_transaction_id)
            .bind(&refund_transaction_no)
            .bind(&order.id)
            .bind(&transaction.payment_method)
            .bind(&refund.refund_amount)
            .bind(&now)
            .bind(&now)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        tx.commit()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    // Balance management
    pub async fn get_user_balance(db: &DbPool, user_id: Uuid) -> Result<UserBalance, AppError> {
        let query = r#"
            SELECT * FROM user_balances WHERE user_id = ?
        "#;

        sqlx::query_as::<_, UserBalance>(query)
            .bind(&user_id)
            .fetch_optional(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?
            .ok_or_else(|| {
                // Create balance record if not exists
                AppError::NotFound("用户余额记录不存在".to_string())
            })
    }

    async fn get_user_balance_tx(
        tx: &mut Transaction<'_, MySql>,
        user_id: Uuid,
    ) -> Result<UserBalance, AppError> {
        let query = r#"
            SELECT * FROM user_balances WHERE user_id = ? FOR UPDATE
        "#;

        sqlx::query_as::<_, UserBalance>(query)
            .bind(&user_id)
            .fetch_optional(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?
            .ok_or_else(|| AppError::NotFound("用户余额记录不存在".to_string()))
    }

    pub async fn create_user_balance(db: &DbPool, user_id: Uuid) -> Result<UserBalance, AppError> {
        let balance_id = Uuid::new_v4();
        let now = Utc::now();

        let query = r#"
            INSERT INTO user_balances (
                id, user_id, balance, frozen_balance, 
                total_income, total_expense, created_at, updated_at
            ) VALUES (?, ?, 0, 0, 0, 0, ?, ?)
        "#;

        sqlx::query(query)
            .bind(&balance_id)
            .bind(&user_id)
            .bind(&now)
            .bind(&now)
            .execute(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Self::get_user_balance(db, user_id).await
    }

    async fn update_balance_tx(
        tx: &mut Transaction<'_, MySql>,
        user_id: Uuid,
        transaction_type: BalanceTransactionType,
        amount: Decimal,
        related_type: Option<String>,
        related_id: Option<Uuid>,
        description: &str,
    ) -> Result<(), AppError> {
        // Get current balance
        let balance = Self::get_user_balance_tx(tx, user_id).await?;
        let balance_before = balance.balance;

        // Calculate new balance
        let balance_after = match transaction_type {
            BalanceTransactionType::Income => balance.balance + amount,
            BalanceTransactionType::Expense => {
                if balance.balance < amount {
                    return Err(AppError::BadRequest("余额不足".to_string()));
                }
                balance.balance - amount
            }
            BalanceTransactionType::Freeze => {
                if balance.balance < amount {
                    return Err(AppError::BadRequest("可用余额不足".to_string()));
                }
                balance.balance - amount
            }
            BalanceTransactionType::Unfreeze => balance.balance + amount,
        };

        // Update balance
        let query = match transaction_type {
            BalanceTransactionType::Income => {
                r#"
                UPDATE user_balances
                SET balance = balance + ?, total_income = total_income + ?, updated_at = ?
                WHERE user_id = ?
            "#
            }
            BalanceTransactionType::Expense => {
                r#"
                UPDATE user_balances
                SET balance = balance - ?, total_expense = total_expense + ?, updated_at = ?
                WHERE user_id = ?
            "#
            }
            BalanceTransactionType::Freeze => {
                r#"
                UPDATE user_balances
                SET balance = balance - ?, frozen_balance = frozen_balance + ?, updated_at = ?
                WHERE user_id = ?
            "#
            }
            BalanceTransactionType::Unfreeze => {
                r#"
                UPDATE user_balances
                SET balance = balance + ?, frozen_balance = frozen_balance - ?, updated_at = ?
                WHERE user_id = ?
            "#
            }
        };

        let now = Utc::now();
        sqlx::query(query)
            .bind(&amount)
            .bind(&amount)
            .bind(&now)
            .bind(&user_id)
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Create transaction record
        let transaction_id = Uuid::new_v4();
        let query = r#"
            INSERT INTO balance_transactions (
                id, user_id, transaction_type, amount,
                balance_before, balance_after, related_type,
                related_id, description, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
        "#;

        sqlx::query(query)
            .bind(&transaction_id)
            .bind(&user_id)
            .bind(&transaction_type)
            .bind(&amount)
            .bind(&balance_before)
            .bind(&balance_after)
            .bind(&related_type)
            .bind(&related_id)
            .bind(description)
            .bind(&now)
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn get_balance_transactions(
        db: &DbPool,
        user_id: Uuid,
        page: i64,
        page_size: i64,
    ) -> Result<Vec<BalanceTransaction>, AppError> {
        let offset = (page - 1) * page_size;

        let query = r#"
            SELECT * FROM balance_transactions
            WHERE user_id = ?
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
        "#;

        sqlx::query_as::<_, BalanceTransaction>(query)
            .bind(&user_id)
            .bind(page_size)
            .bind(offset)
            .fetch_all(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    // Configuration management
    pub async fn get_payment_config(
        db: &DbPool,
        payment_method: PaymentMethod,
    ) -> Result<HashMap<String, String>, AppError> {
        let query = r#"
            SELECT config_key, config_value FROM payment_configs
            WHERE payment_method = ?
        "#;

        let configs: Vec<(String, String)> = sqlx::query_as(query)
            .bind(&payment_method)
            .fetch_all(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(configs.into_iter().collect())
    }

    pub async fn update_payment_config(
        db: &DbPool,
        payment_method: PaymentMethod,
        config_key: &str,
        config_value: &str,
        is_encrypted: bool,
    ) -> Result<(), AppError> {
        let query = r#"
            INSERT INTO payment_configs (id, payment_method, config_key, config_value, is_encrypted, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON DUPLICATE KEY UPDATE
            config_value = VALUES(config_value),
            is_encrypted = VALUES(is_encrypted),
            updated_at = VALUES(updated_at)
        "#;

        let now = Utc::now();
        sqlx::query(query)
            .bind(&Uuid::new_v4())
            .bind(&payment_method)
            .bind(config_key)
            .bind(config_value)
            .bind(is_encrypted)
            .bind(&now)
            .bind(&now)
            .execute(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    // Price management
    pub async fn get_price_config(
        db: &DbPool,
        service_type: &str,
    ) -> Result<Option<PriceConfig>, AppError> {
        let query = r#"
            SELECT * FROM price_configs
            WHERE service_type = ? AND is_active = true
            AND (effective_date IS NULL OR effective_date <= CURDATE())
            AND (expiry_date IS NULL OR expiry_date >= CURDATE())
            ORDER BY created_at DESC
            LIMIT 1
        "#;

        sqlx::query_as::<_, PriceConfig>(query)
            .bind(service_type)
            .fetch_optional(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn list_price_configs(
        db: &DbPool,
        is_active: Option<bool>,
    ) -> Result<Vec<PriceConfig>, AppError> {
        let query = match is_active {
            Some(_active) => {
                r#"
                SELECT * FROM price_configs
                WHERE is_active = ?
                ORDER BY service_type, created_at DESC
            "#
            }
            None => {
                r#"
                SELECT * FROM price_configs
                ORDER BY service_type, created_at DESC
            "#
            }
        };

        if let Some(active) = is_active {
            sqlx::query_as::<_, PriceConfig>(query)
                .bind(active)
                .fetch_all(db)
                .await
        } else {
            sqlx::query_as::<_, PriceConfig>(query).fetch_all(db).await
        }
        .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    // Statistics
    pub async fn get_payment_statistics(
        db: &DbPool,
        user_id: Option<Uuid>,
        start_date: Option<chrono::DateTime<Utc>>,
        end_date: Option<chrono::DateTime<Utc>>,
    ) -> Result<PaymentStatistics, AppError> {
        let mut where_clauses = vec![];

        if user_id.is_some() {
            where_clauses.push("user_id = ?");
        }

        if start_date.is_some() {
            where_clauses.push("created_at >= ?");
        }

        if end_date.is_some() {
            where_clauses.push("created_at <= ?");
        }

        let where_clause = if where_clauses.is_empty() {
            String::new()
        } else {
            format!("WHERE {}", where_clauses.join(" AND "))
        };

        let query = format!(
            r#"
            SELECT 
                COUNT(*) as total_orders,
                COALESCE(SUM(amount), 0) as total_amount,
                COUNT(CASE WHEN status = 'paid' THEN 1 END) as paid_orders,
                COALESCE(SUM(CASE WHEN status = 'paid' THEN amount END), 0) as paid_amount,
                COUNT(CASE WHEN status IN ('refunded', 'partial_refunded') THEN 1 END) as refunded_orders,
                COALESCE(SUM(CASE WHEN status IN ('refunded', 'partial_refunded') THEN amount END), 0) as refunded_amount
            FROM payment_orders
            {}
            "#,
            where_clause
        );

        let mut query_builder = sqlx::query(&query);

        if let Some(uid) = user_id {
            query_builder = query_builder.bind(uid);
        }

        if let Some(start) = start_date {
            query_builder = query_builder.bind(start);
        }

        if let Some(end) = end_date {
            query_builder = query_builder.bind(end);
        }

        let row = query_builder
            .fetch_one(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        use sqlx::Row;
        Ok(PaymentStatistics {
            total_orders: row.get::<Option<i64>, _>("total_orders").unwrap_or(0),
            total_amount: row
                .get::<Option<Decimal>, _>("total_amount")
                .unwrap_or(Decimal::ZERO),
            paid_orders: row.get::<Option<i64>, _>("paid_orders").unwrap_or(0),
            paid_amount: row
                .get::<Option<Decimal>, _>("paid_amount")
                .unwrap_or(Decimal::ZERO),
            refunded_orders: row.get::<Option<i64>, _>("refunded_orders").unwrap_or(0),
            refunded_amount: row
                .get::<Option<Decimal>, _>("refunded_amount")
                .unwrap_or(Decimal::ZERO),
        })
    }

    // Helper methods
    async fn get_transaction(
        db: &DbPool,
        transaction_id: Uuid,
    ) -> Result<PaymentTransaction, AppError> {
        let query = r#"
            SELECT * FROM payment_transactions WHERE id = ?
        "#;

        sqlx::query_as::<_, PaymentTransaction>(query)
            .bind(&transaction_id)
            .fetch_one(db)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => AppError::NotFound("交易记录不存在".to_string()),
                _ => AppError::DatabaseError(e.to_string()),
            })
    }

    fn generate_order_no() -> String {
        let timestamp = Utc::now().format("%Y%m%d%H%M%S");
        let random = rand::random::<u32>() % 10000;
        format!("ORD{}{:04}", timestamp, random)
    }

    fn generate_transaction_no() -> String {
        let timestamp = Utc::now().format("%Y%m%d%H%M%S");
        let random = rand::random::<u32>() % 10000;
        format!("TXN{}{:04}", timestamp, random)
    }

    fn generate_refund_no() -> String {
        let timestamp = Utc::now().format("%Y%m%d%H%M%S");
        let random = rand::random::<u32>() % 10000;
        format!("RFD{}{:04}", timestamp, random)
    }
}
