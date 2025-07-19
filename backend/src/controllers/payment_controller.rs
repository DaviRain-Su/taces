use crate::{
    middleware::auth::AuthUser,
    models::{payment::*, ApiResponse},
    services::payment_service::PaymentService,
    utils::errors::AppError,
    AppState,
};
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};
use rust_decimal::Decimal;
use serde::Deserialize;
use std::str::FromStr;
use uuid::Uuid;

// Order endpoints
pub async fn create_order(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(dto): Json<CreateOrderDto>,
) -> Result<impl IntoResponse, AppError> {
    let order = PaymentService::create_order(&state.pool, dto).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success("订单创建成功", order)),
    ))
}

pub async fn get_order(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(order_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let order = PaymentService::get_order(&state.pool, order_id).await?;

    // Check authorization
    if order.user_id != auth_user.user_id && auth_user.role != "admin" {
        return Err(AppError::Forbidden);
    }

    Ok(Json(ApiResponse::success("获取订单成功", order)))
}

pub async fn list_orders(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<OrderListQuery>,
) -> Result<impl IntoResponse, AppError> {
    // Filter by user unless admin
    let mut filtered_query = query;
    if auth_user.role != "admin" {
        filtered_query.user_id = Some(auth_user.user_id);
    }

    let response = PaymentService::list_orders(&state.pool, filtered_query).await?;

    Ok(Json(ApiResponse::success("获取订单列表成功", response)))
}

pub async fn cancel_order(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(order_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let order = PaymentService::get_order(&state.pool, order_id).await?;

    // Check authorization
    if order.user_id != auth_user.user_id && auth_user.role != "admin" {
        return Err(AppError::Forbidden);
    }

    PaymentService::cancel_order(&state.pool, order_id).await?;

    Ok(Json(ApiResponse::success("订单取消成功", ())))
}

// Payment endpoints
pub async fn initiate_payment(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(dto): Json<InitiatePaymentDto>,
) -> Result<impl IntoResponse, AppError> {
    let order = PaymentService::get_order(&state.pool, dto.order_id).await?;

    // Check authorization
    if order.user_id != auth_user.user_id {
        return Err(AppError::Forbidden);
    }

    let response = PaymentService::initiate_payment(&state.pool, dto).await?;

    Ok(Json(ApiResponse::success("支付发起成功", response)))
}

#[derive(Deserialize)]
pub struct PaymentCallbackQuery {
    pub method: String,
}

pub async fn payment_callback(
    State(state): State<AppState>,
    Query(query): Query<PaymentCallbackQuery>,
    Json(data): Json<serde_json::Value>,
) -> Result<impl IntoResponse, AppError> {
    // Parse payment method
    let payment_method = match query.method.as_str() {
        "wechat" => PaymentMethod::Wechat,
        "alipay" => PaymentMethod::Alipay,
        _ => return Err(AppError::BadRequest("无效的支付方式".to_string())),
    };

    // TODO: Validate callback signature based on payment method

    // Parse callback data based on payment method
    let callback_data = match payment_method {
        PaymentMethod::Wechat => {
            // Parse WeChat callback
            PaymentCallbackData {
                order_no: data["out_trade_no"]
                    .as_str()
                    .ok_or_else(|| AppError::BadRequest("缺少订单号".to_string()))?
                    .to_string(),
                external_transaction_id: data["transaction_id"]
                    .as_str()
                    .ok_or_else(|| AppError::BadRequest("缺少交易ID".to_string()))?
                    .to_string(),
                amount: rust_decimal::Decimal::from_str_exact(
                    &(data["amount"]["total"].as_i64().unwrap_or(0) as f64 / 100.0).to_string(),
                )
                .map_err(|_| AppError::BadRequest("金额格式错误".to_string()))?,
                status: if data["trade_state"].as_str() == Some("SUCCESS") {
                    "success".to_string()
                } else {
                    "failed".to_string()
                },
                payment_time: chrono::Utc::now(), // TODO: Parse from callback
                raw_data: data.clone(),
            }
        }
        PaymentMethod::Alipay => {
            // Parse Alipay callback
            PaymentCallbackData {
                order_no: data["out_trade_no"]
                    .as_str()
                    .ok_or_else(|| AppError::BadRequest("缺少订单号".to_string()))?
                    .to_string(),
                external_transaction_id: data["trade_no"]
                    .as_str()
                    .ok_or_else(|| AppError::BadRequest("缺少交易ID".to_string()))?
                    .to_string(),
                amount: rust_decimal::Decimal::from_str_exact(
                    data["total_amount"]
                        .as_str()
                        .ok_or_else(|| AppError::BadRequest("缺少金额".to_string()))?,
                )
                .map_err(|_| AppError::BadRequest("金额格式错误".to_string()))?,
                status: if data["trade_status"].as_str() == Some("TRADE_SUCCESS") {
                    "success".to_string()
                } else {
                    "failed".to_string()
                },
                payment_time: chrono::Utc::now(), // TODO: Parse from callback
                raw_data: data.clone(),
            }
        }
        _ => return Err(AppError::BadRequest("不支持的支付方式".to_string())),
    };

    PaymentService::handle_payment_callback(&state.pool, payment_method, callback_data).await?;

    // Return success response for payment gateway
    match payment_method {
        PaymentMethod::Wechat => Ok(Json(serde_json::json!({
            "code": "SUCCESS",
            "message": "成功"
        }))),
        PaymentMethod::Alipay => Ok(Json(serde_json::json!("success"))),
        _ => Ok(Json(serde_json::json!({"success": true}))),
    }
}

// Refund endpoints
pub async fn create_refund(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(dto): Json<CreateRefundDto>,
) -> Result<impl IntoResponse, AppError> {
    let order = PaymentService::get_order(&state.pool, dto.order_id).await?;

    // Check authorization
    if order.user_id != auth_user.user_id && auth_user.role != "admin" {
        return Err(AppError::Forbidden);
    }

    let refund = PaymentService::create_refund(&state.pool, dto, auth_user.user_id).await?;

    Ok((
        StatusCode::CREATED,
        Json(ApiResponse::success("退款申请创建成功", refund)),
    ))
}

pub async fn get_refund(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(refund_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    let refund = PaymentService::get_refund(&state.pool, refund_id).await?;

    // Check authorization
    if refund.user_id != auth_user.user_id && auth_user.role != "admin" {
        return Err(AppError::Forbidden);
    }

    Ok(Json(ApiResponse::success("获取退款记录成功", refund)))
}

pub async fn review_refund(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(refund_id): Path<Uuid>,
    Json(dto): Json<ReviewRefundDto>,
) -> Result<impl IntoResponse, AppError> {
    // Only admin can review refunds
    if auth_user.role != "admin" {
        return Err(AppError::Forbidden);
    }

    PaymentService::review_refund(&state.pool, refund_id, dto, auth_user.user_id).await?;

    Ok(Json(ApiResponse::success("退款审核完成", ())))
}

// Balance endpoints
pub async fn get_user_balance(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(user_id): Path<Uuid>,
) -> Result<impl IntoResponse, AppError> {
    // Check authorization
    if user_id != auth_user.user_id && auth_user.role != "admin" {
        return Err(AppError::Forbidden);
    }

    let balance = PaymentService::get_user_balance(&state.pool, user_id)
        .await
        .or_else(|_| async {
            PaymentService::create_user_balance(&state.pool, user_id).await
        })
        .await?;

    Ok(Json(ApiResponse::success("获取余额成功", balance)))
}

#[derive(Deserialize)]
pub struct BalanceTransactionsQuery {
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

pub async fn get_balance_transactions(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(user_id): Path<Uuid>,
    Query(query): Query<BalanceTransactionsQuery>,
) -> Result<impl IntoResponse, AppError> {
    // Check authorization
    if user_id != auth_user.user_id && auth_user.role != "admin" {
        return Err(AppError::Forbidden);
    }

    let page = query.page.unwrap_or(1).max(1);
    let page_size = query.page_size.unwrap_or(20).min(100);

    let transactions =
        PaymentService::get_balance_transactions(&state.pool, user_id, page, page_size).await?;

    Ok(Json(ApiResponse::success("获取余额变动记录成功", transactions)))
}

// Price configuration endpoints
pub async fn get_price_config(
    State(state): State<AppState>,
    Path(service_type): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let price_config = PaymentService::get_price_config(&state.pool, &service_type).await?;

    match price_config {
        Some(config) => Ok(Json(ApiResponse::success("获取价格配置成功", config))),
        None => Err(AppError::NotFound("价格配置不存在".to_string())),
    }
}

pub async fn list_price_configs(
    State(state): State<AppState>,
    Query(query): Query<std::collections::HashMap<String, String>>,
) -> Result<impl IntoResponse, AppError> {
    let is_active = query
        .get("is_active")
        .and_then(|v| v.parse::<bool>().ok());

    let configs = PaymentService::list_price_configs(&state.pool, is_active).await?;

    Ok(Json(ApiResponse::success("获取价格配置列表成功", configs)))
}

// Statistics endpoints
#[derive(Deserialize)]
pub struct PaymentStatisticsQuery {
    pub user_id: Option<Uuid>,
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
}

pub async fn get_payment_statistics(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Query(query): Query<PaymentStatisticsQuery>,
) -> Result<impl IntoResponse, AppError> {
    // Filter by user unless admin
    let user_id = if auth_user.role == "admin" {
        query.user_id
    } else {
        Some(auth_user.user_id)
    };

    let statistics = PaymentService::get_payment_statistics(
        &state.pool,
        user_id,
        query.start_date,
        query.end_date,
    )
    .await?;

    Ok(Json(ApiResponse::success("获取支付统计成功", statistics)))
}

// Admin endpoints
#[derive(Deserialize)]
pub struct UpdatePaymentConfigDto {
    pub config_key: String,
    pub config_value: String,
    pub is_encrypted: bool,
}

pub async fn update_payment_config(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(payment_method): Path<String>,
    Json(dto): Json<UpdatePaymentConfigDto>,
) -> Result<impl IntoResponse, AppError> {
    // Only admin can update payment config
    if auth_user.role != "admin" {
        return Err(AppError::Forbidden);
    }

    let method = match payment_method.as_str() {
        "wechat" => PaymentMethod::Wechat,
        "alipay" => PaymentMethod::Alipay,
        _ => return Err(AppError::BadRequest("无效的支付方式".to_string())),
    };

    PaymentService::update_payment_config(
        &state.pool,
        method,
        &dto.config_key,
        &dto.config_value,
        dto.is_encrypted,
    )
    .await?;

    Ok(Json(ApiResponse::success("支付配置更新成功", ())))
}