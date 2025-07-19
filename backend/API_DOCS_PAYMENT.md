# Payment System API Documentation

## Overview

The payment system provides comprehensive payment processing capabilities for the TCM Telemedicine Platform, including order management, payment processing, refunds, balance management, and pricing configuration.

## Authentication

All payment endpoints require authentication via JWT token in the Authorization header:
```
Authorization: Bearer <token>
```

Exception: Payment callback endpoint (`POST /payment/callback`) does not require authentication as it's called by external payment gateways.

## Permissions

- **Patient**: Can manage their own orders, payments, and refunds
- **Doctor**: Same as patient permissions
- **Admin**: Full access to all payment data, can review refunds and update configurations

## API Endpoints

### Order Management

#### Create Order
```http
POST /api/v1/payment/orders
```

Create a new payment order.

**Request Body:**
```json
{
  "user_id": "uuid",
  "appointment_id": "uuid (optional)",
  "order_type": "appointment|consultation|prescription|other",
  "amount": "30.00",
  "description": "Service description (optional)",
  "metadata": {} // Optional additional data
}
```

**Response:**
```json
{
  "success": true,
  "message": "订单创建成功",
  "data": {
    "id": "uuid",
    "order_no": "ORD20240120123456",
    "user_id": "uuid",
    "appointment_id": "uuid",
    "order_type": "consultation",
    "amount": "30.00",
    "currency": "CNY",
    "status": "pending",
    "payment_method": null,
    "payment_time": null,
    "expire_time": "2024-01-20T15:00:00Z",
    "description": "图文咨询服务",
    "metadata": {},
    "created_at": "2024-01-20T13:00:00Z",
    "updated_at": "2024-01-20T13:00:00Z"
  }
}
```

#### List Orders
```http
GET /api/v1/payment/orders
```

Get user's orders with optional filtering.

**Query Parameters:**
- `user_id` (optional, admin only): Filter by user
- `status` (optional): pending|paid|cancelled|refunded|partial_refunded|expired
- `order_type` (optional): appointment|consultation|prescription|other
- `start_date` (optional): ISO 8601 datetime
- `end_date` (optional): ISO 8601 datetime
- `page` (optional, default: 1): Page number
- `page_size` (optional, default: 20, max: 100): Items per page

**Response:**
```json
{
  "success": true,
  "message": "获取订单列表成功",
  "data": {
    "orders": [
      {
        "id": "uuid",
        "order_no": "ORD20240120123456",
        "user_id": "uuid",
        "order_type": "consultation",
        "amount": "30.00",
        "status": "paid",
        "payment_method": "alipay",
        "payment_time": "2024-01-20T13:30:00Z",
        "created_at": "2024-01-20T13:00:00Z"
      }
    ],
    "total": 50,
    "page": 1,
    "page_size": 20
  }
}
```

#### Get Order Details
```http
GET /api/v1/payment/orders/:id
```

Get detailed information about a specific order.

**Response:**
```json
{
  "success": true,
  "message": "获取订单成功",
  "data": {
    "id": "uuid",
    "order_no": "ORD20240120123456",
    "user_id": "uuid",
    "appointment_id": "uuid",
    "order_type": "consultation",
    "amount": "30.00",
    "currency": "CNY",
    "status": "paid",
    "payment_method": "alipay",
    "payment_time": "2024-01-20T13:30:00Z",
    "expire_time": "2024-01-20T15:00:00Z",
    "description": "图文咨询服务",
    "metadata": {},
    "created_at": "2024-01-20T13:00:00Z",
    "updated_at": "2024-01-20T13:30:00Z"
  }
}
```

#### Cancel Order
```http
PUT /api/v1/payment/orders/:id/cancel
```

Cancel a pending order. Only orders with `pending` status can be cancelled.

**Response:**
```json
{
  "success": true,
  "message": "订单取消成功",
  "data": null
}
```

### Payment Processing

#### Initiate Payment
```http
POST /api/v1/payment/pay
```

Initiate payment for an order.

**Request Body:**
```json
{
  "order_id": "uuid",
  "payment_method": "wechat|alipay|balance",
  "return_url": "https://example.com/return" // Optional, for Alipay
}
```

**Response (WeChat Pay):**
```json
{
  "success": true,
  "message": "支付发起成功",
  "data": {
    "order_id": "uuid",
    "order_no": "ORD20240120123456",
    "payment_method": "wechat",
    "payment_url": null,
    "qr_code": "wxp://f2f0...",
    "prepay_data": {
      "prepay_id": "wx_prepay_xxx",
      "appid": "wx123456",
      "timestamp": 1705750200,
      "nonce_str": "uuid"
    }
  }
}
```

**Response (Alipay):**
```json
{
  "success": true,
  "message": "支付发起成功",
  "data": {
    "order_id": "uuid",
    "order_no": "ORD20240120123456",
    "payment_method": "alipay",
    "payment_url": "https://openapi.alipay.com/gateway.do?trade_no=xxx",
    "qr_code": null,
    "prepay_data": null
  }
}
```

**Response (Balance):**
```json
{
  "success": true,
  "message": "支付发起成功",
  "data": {
    "order_id": "uuid",
    "order_no": "ORD20240120123456",
    "payment_method": "balance",
    "payment_url": null,
    "qr_code": null,
    "prepay_data": null
  }
}
```

#### Payment Callback
```http
POST /payment/callback?method=wechat|alipay
```

Endpoint for payment gateway callbacks. No authentication required.

**Request Body (varies by payment method):**
```json
{
  // WeChat Pay callback
  "out_trade_no": "ORD20240120123456",
  "transaction_id": "wx_transaction_id",
  "trade_state": "SUCCESS",
  "amount": {
    "total": 3000  // in cents
  }
}

// OR

{
  // Alipay callback
  "out_trade_no": "ORD20240120123456",
  "trade_no": "alipay_trade_no",
  "trade_status": "TRADE_SUCCESS",
  "total_amount": "30.00"
}
```

**Response:**
```json
{
  "code": "SUCCESS",
  "message": "成功"
}
```

### Refund Management

#### Create Refund Request
```http
POST /api/v1/payment/refunds
```

Request a refund for a paid order.

**Request Body:**
```json
{
  "order_id": "uuid",
  "refund_amount": "30.00",
  "refund_reason": "服务未提供"
}
```

**Response:**
```json
{
  "success": true,
  "message": "退款申请创建成功",
  "data": {
    "id": "uuid",
    "refund_no": "RFD20240120123456",
    "order_id": "uuid",
    "transaction_id": "uuid",
    "user_id": "uuid",
    "refund_amount": "30.00",
    "refund_reason": "服务未提供",
    "status": "pending",
    "reviewed_by": null,
    "reviewed_at": null,
    "review_notes": null,
    "created_at": "2024-01-20T14:00:00Z",
    "updated_at": "2024-01-20T14:00:00Z"
  }
}
```

#### Get Refund Details
```http
GET /api/v1/payment/refunds/:id
```

Get refund request details.

**Response:**
```json
{
  "success": true,
  "message": "获取退款记录成功",
  "data": {
    "id": "uuid",
    "refund_no": "RFD20240120123456",
    "order_id": "uuid",
    "transaction_id": "uuid",
    "user_id": "uuid",
    "refund_amount": "30.00",
    "refund_reason": "服务未提供",
    "status": "success",
    "reviewed_by": "admin_uuid",
    "reviewed_at": "2024-01-20T15:00:00Z",
    "review_notes": "同意退款",
    "completed_at": "2024-01-20T15:05:00Z",
    "created_at": "2024-01-20T14:00:00Z",
    "updated_at": "2024-01-20T15:05:00Z"
  }
}
```

#### Review Refund (Admin Only)
```http
PUT /api/v1/payment/admin/refunds/:id/review
```

Review and process a refund request.

**Request Body:**
```json
{
  "approved": true,
  "review_notes": "同意退款"
}
```

**Response:**
```json
{
  "success": true,
  "message": "退款审核完成",
  "data": null
}
```

### Balance Management

#### Get User Balance
```http
GET /api/v1/payment/balance/:user_id
```

Get user's account balance. Users can only access their own balance unless admin.

**Response:**
```json
{
  "success": true,
  "message": "获取余额成功",
  "data": {
    "id": "uuid",
    "user_id": "uuid",
    "balance": "100.00",
    "frozen_balance": "0.00",
    "total_income": "500.00",
    "total_expense": "400.00",
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-20T14:00:00Z"
  }
}
```

#### Get Balance Transactions
```http
GET /api/v1/payment/balance/:user_id/transactions
```

Get user's balance transaction history.

**Query Parameters:**
- `page` (optional, default: 1): Page number
- `page_size` (optional, default: 20, max: 100): Items per page

**Response:**
```json
{
  "success": true,
  "message": "获取余额变动记录成功",
  "data": [
    {
      "id": "uuid",
      "user_id": "uuid",
      "transaction_type": "income|expense|freeze|unfreeze",
      "amount": "30.00",
      "balance_before": "100.00",
      "balance_after": "70.00",
      "related_type": "order",
      "related_id": "order_uuid",
      "description": "订单支付: ORD20240120123456",
      "created_at": "2024-01-20T13:30:00Z"
    }
  ]
}
```

### Price Configuration

#### List Price Configs
```http
GET /api/v1/payment/prices
```

Get all service price configurations. This endpoint is public.

**Query Parameters:**
- `is_active` (optional): true|false - Filter by active status

**Response:**
```json
{
  "success": true,
  "message": "获取价格配置列表成功",
  "data": [
    {
      "id": "uuid",
      "service_type": "consultation",
      "service_name": "图文咨询",
      "price": "20.00",
      "discount_price": null,
      "is_active": true,
      "effective_date": null,
      "expiry_date": null,
      "description": "图文咨询服务费",
      "created_at": "2024-01-01T00:00:00Z",
      "updated_at": "2024-01-01T00:00:00Z"
    }
  ]
}
```

#### Get Service Price
```http
GET /api/v1/payment/prices/:service_type
```

Get price for a specific service type. This endpoint is public.

**Response:**
```json
{
  "success": true,
  "message": "获取价格配置成功",
  "data": {
    "id": "uuid",
    "service_type": "consultation",
    "service_name": "图文咨询",
    "price": "20.00",
    "discount_price": null,
    "is_active": true,
    "effective_date": null,
    "expiry_date": null,
    "description": "图文咨询服务费",
    "created_at": "2024-01-01T00:00:00Z",
    "updated_at": "2024-01-01T00:00:00Z"
  }
}
```

### Statistics

#### Get Payment Statistics
```http
GET /api/v1/payment/statistics
```

Get payment statistics. Regular users see only their own data, admins can see all or filter by user.

**Query Parameters:**
- `user_id` (optional, admin only): Filter by user
- `start_date` (optional): ISO 8601 datetime
- `end_date` (optional): ISO 8601 datetime

**Response:**
```json
{
  "success": true,
  "message": "获取支付统计成功",
  "data": {
    "total_orders": 100,
    "total_amount": "3000.00",
    "paid_orders": 80,
    "paid_amount": "2400.00",
    "refunded_orders": 10,
    "refunded_amount": "300.00"
  }
}
```

### Admin Configuration

#### Update Payment Config (Admin Only)
```http
PUT /api/v1/payment/admin/config/:payment_method
```

Update payment method configuration.

**Path Parameters:**
- `payment_method`: wechat|alipay

**Request Body:**
```json
{
  "config_key": "app_id",
  "config_value": "wx123456789",
  "is_encrypted": false
}
```

**Response:**
```json
{
  "success": true,
  "message": "支付配置更新成功",
  "data": null
}
```

## Error Responses

All endpoints may return the following error responses:

### 400 Bad Request
```json
{
  "success": false,
  "message": "Invalid request parameters"
}
```

### 401 Unauthorized
```json
{
  "success": false,
  "message": "未授权"
}
```

### 403 Forbidden
```json
{
  "success": false,
  "message": "禁止访问"
}
```

### 404 Not Found
```json
{
  "success": false,
  "message": "订单不存在"
}
```

### 500 Internal Server Error
```json
{
  "success": false,
  "message": "Internal server error"
}
```

## Integration Examples

### Complete Payment Flow

1. **Create Order**
```bash
curl -X POST http://localhost:3000/api/v1/payment/orders \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "user_uuid",
    "order_type": "consultation",
    "amount": "30.00",
    "description": "图文咨询服务"
  }'
```

2. **Initiate Payment**
```bash
curl -X POST http://localhost:3000/api/v1/payment/pay \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "order_id": "order_uuid",
    "payment_method": "alipay"
  }'
```

3. **Handle Callback** (Called by payment gateway)
```bash
curl -X POST http://localhost:3000/payment/callback?method=alipay \
  -H "Content-Type: application/json" \
  -d '{
    "out_trade_no": "ORD20240120123456",
    "trade_no": "2024012022001412345",
    "trade_status": "TRADE_SUCCESS",
    "total_amount": "30.00"
  }'
```

### Refund Flow

1. **Request Refund**
```bash
curl -X POST http://localhost:3000/api/v1/payment/refunds \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "order_id": "order_uuid",
    "refund_amount": "30.00",
    "refund_reason": "服务未提供"
  }'
```

2. **Admin Review** (Admin only)
```bash
curl -X PUT http://localhost:3000/api/v1/payment/admin/refunds/refund_uuid/review \
  -H "Authorization: Bearer <admin_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "approved": true,
    "review_notes": "同意退款"
  }'
```

## Notes

1. All monetary amounts are in decimal format with 2 decimal places (e.g., "30.00")
2. Order expiration time is set to 2 hours after creation
3. Balance payments are processed immediately
4. WeChat Pay and Alipay integrations require additional configuration
5. Refunds to balance are processed immediately, third-party refunds may take time
6. Payment configurations should be encrypted when storing sensitive data like API keys