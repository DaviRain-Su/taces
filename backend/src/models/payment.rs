use chrono::{DateTime, NaiveDate, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type)]
#[sqlx(type_name = "order_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum OrderType {
    Appointment,
    Consultation,
    Prescription,
    Other,
}

impl std::fmt::Display for OrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OrderType::Appointment => write!(f, "Appointment"),
            OrderType::Consultation => write!(f, "Consultation"),
            OrderType::Prescription => write!(f, "Prescription"),
            OrderType::Other => write!(f, "Other"),
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type, PartialEq)]
#[sqlx(type_name = "order_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum OrderStatus {
    Pending,
    Paid,
    Cancelled,
    Refunded,
    PartialRefunded,
    Expired,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type)]
#[sqlx(type_name = "payment_method", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum PaymentMethod {
    Wechat,
    Alipay,
    BankCard,
    Balance,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type)]
#[sqlx(type_name = "transaction_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum TransactionType {
    Payment,
    Refund,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type, PartialEq)]
#[sqlx(type_name = "transaction_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum TransactionStatus {
    Pending,
    Success,
    Failed,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type, PartialEq)]
#[sqlx(type_name = "refund_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum RefundStatus {
    Pending,
    Processing,
    Success,
    Failed,
    Cancelled,
}

#[derive(Debug, Serialize, Deserialize, Clone, sqlx::Type)]
#[sqlx(type_name = "balance_transaction_type", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum BalanceTransactionType {
    Income,
    Expense,
    Freeze,
    Unfreeze,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PaymentOrder {
    pub id: Uuid,
    pub order_no: String,
    pub user_id: Uuid,
    pub appointment_id: Option<Uuid>,
    pub order_type: OrderType,
    pub amount: Decimal,
    pub currency: String,
    pub status: OrderStatus,
    pub payment_method: Option<PaymentMethod>,
    pub payment_time: Option<DateTime<Utc>>,
    pub expire_time: DateTime<Utc>,
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateOrderDto {
    pub user_id: Uuid,
    pub appointment_id: Option<Uuid>,
    pub order_type: OrderType,
    // TODO: Add custom validation for Decimal
    pub amount: Decimal,
    #[validate(length(max = 500))]
    pub description: Option<String>,
    pub metadata: Option<serde_json::Value>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PaymentTransaction {
    pub id: Uuid,
    pub transaction_no: String,
    pub order_id: Uuid,
    pub payment_method: PaymentMethod,
    pub transaction_type: TransactionType,
    pub amount: Decimal,
    pub status: TransactionStatus,
    pub external_transaction_id: Option<String>,
    pub prepay_id: Option<String>,
    pub trade_no: Option<String>,
    pub request_data: Option<serde_json::Value>,
    pub response_data: Option<serde_json::Value>,
    pub callback_data: Option<serde_json::Value>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub initiated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct InitiatePaymentDto {
    pub order_id: Uuid,
    pub payment_method: PaymentMethod,
    #[validate(length(max = 100))]
    pub return_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct RefundRecord {
    pub id: Uuid,
    pub refund_no: String,
    pub order_id: Uuid,
    pub transaction_id: Uuid,
    pub user_id: Uuid,
    pub refund_amount: Decimal,
    pub refund_reason: String,
    pub status: RefundStatus,
    pub reviewed_by: Option<Uuid>,
    pub reviewed_at: Option<DateTime<Utc>>,
    pub review_notes: Option<String>,
    pub external_refund_id: Option<String>,
    pub refund_response: Option<serde_json::Value>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateRefundDto {
    pub order_id: Uuid,
    // TODO: Add custom validation for Decimal
    pub refund_amount: Decimal,
    #[validate(length(min = 1, max = 500))]
    pub refund_reason: String,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct ReviewRefundDto {
    pub approved: bool,
    #[validate(length(max = 500))]
    pub review_notes: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PaymentConfig {
    pub id: Uuid,
    pub payment_method: PaymentMethod,
    pub config_key: String,
    pub config_value: String,
    pub is_encrypted: bool,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct PriceConfig {
    pub id: Uuid,
    pub service_type: String,
    pub service_name: String,
    pub price: Decimal,
    pub discount_price: Option<Decimal>,
    pub is_active: bool,
    pub effective_date: Option<NaiveDate>,
    pub expiry_date: Option<NaiveDate>,
    pub description: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct UserBalance {
    pub id: Uuid,
    pub user_id: Uuid,
    pub balance: Decimal,
    pub frozen_balance: Decimal,
    pub total_income: Decimal,
    pub total_expense: Decimal,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct BalanceTransaction {
    pub id: Uuid,
    pub user_id: Uuid,
    pub transaction_type: BalanceTransactionType,
    pub amount: Decimal,
    pub balance_before: Decimal,
    pub balance_after: Decimal,
    pub related_type: Option<String>,
    pub related_id: Option<Uuid>,
    pub description: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentResponse {
    pub order_id: Uuid,
    pub order_no: String,
    pub payment_method: PaymentMethod,
    pub payment_url: Option<String>, // For redirect payments
    pub qr_code: Option<String>,     // For QR code payments
    pub prepay_data: Option<serde_json::Value>, // For SDK payments
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentCallbackData {
    pub order_no: String,
    pub external_transaction_id: String,
    pub amount: Decimal,
    pub status: String,
    pub payment_time: DateTime<Utc>,
    pub raw_data: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderListQuery {
    pub user_id: Option<Uuid>,
    pub status: Option<OrderStatus>,
    pub order_type: Option<OrderType>,
    pub start_date: Option<DateTime<Utc>>,
    pub end_date: Option<DateTime<Utc>>,
    pub page: Option<i64>,
    pub page_size: Option<i64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OrderListResponse {
    pub orders: Vec<PaymentOrder>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PaymentStatistics {
    pub total_orders: i64,
    pub total_amount: Decimal,
    pub paid_orders: i64,
    pub paid_amount: Decimal,
    pub refunded_orders: i64,
    pub refunded_amount: Decimal,
}

// WeChat Pay specific structures
#[derive(Debug, Serialize, Deserialize)]
pub struct WechatPrepayRequest {
    pub appid: String,
    pub mchid: String,
    pub description: String,
    pub out_trade_no: String,
    pub time_expire: String,
    pub notify_url: String,
    pub amount: WechatAmount,
    pub payer: Option<WechatPayer>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WechatAmount {
    pub total: i32, // Amount in cents
    pub currency: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WechatPayer {
    pub openid: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct WechatPrepayResponse {
    pub prepay_id: String,
}

// Alipay specific structures
#[derive(Debug, Serialize, Deserialize)]
pub struct AlipayTradeRequest {
    pub out_trade_no: String,
    pub total_amount: String,
    pub subject: String,
    pub body: Option<String>,
    pub timeout_express: Option<String>,
    pub return_url: Option<String>,
    pub notify_url: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlipayTradeResponse {
    pub code: String,
    pub msg: String,
    pub trade_no: Option<String>,
    pub out_trade_no: Option<String>,
    pub qr_code: Option<String>,
}
