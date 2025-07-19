# Payment System Setup Guide

## Overview

This guide explains how to set up and configure the payment system for the TCM Telemedicine Platform.

## Payment Methods Supported

1. **Balance Payment** (余额支付) - Fully implemented
2. **WeChat Pay** (微信支付) - Interface ready, requires API integration
3. **Alipay** (支付宝) - Interface ready, requires API integration

## Initial Setup

### 1. Database Tables

The payment system tables are automatically created when you run migrations:

```bash
sqlx migrate run
```

This creates the following tables:
- `payment_orders` - Order records
- `payment_transactions` - Transaction logs
- `refund_records` - Refund requests
- `payment_configs` - Payment method configurations
- `price_configs` - Service pricing
- `user_balances` - User account balances
- `balance_transactions` - Balance change history

### 2. Default Price Configuration

The migration automatically inserts default pricing:

| Service Type | Service Name | Price (CNY) |
|-------------|--------------|-------------|
| appointment_offline | 线下问诊 | 50.00 |
| appointment_online | 视频问诊 | 30.00 |
| prescription | 处方费 | 10.00 |
| consultation | 图文咨询 | 20.00 |

To update prices, use the admin API or directly update the `price_configs` table.

### 3. User Balance Setup

Each user needs a balance record. The system automatically creates one when accessing balance for the first time.

To manually create a balance for a user:
```sql
INSERT INTO user_balances (id, user_id, balance, frozen_balance, total_income, total_expense)
VALUES (UUID(), 'user_uuid_here', 0, 0, 0, 0);
```

## Payment Flow

### 1. Order Creation

When a service requires payment:

```javascript
// POST /api/v1/payment/orders
{
  "user_id": "user_uuid",
  "appointment_id": "appointment_uuid", // Optional
  "order_type": "consultation",
  "amount": "30.00",
  "description": "图文咨询服务"
}
```

### 2. Payment Initiation

User selects payment method and initiates payment:

```javascript
// POST /api/v1/payment/pay
{
  "order_id": "order_uuid",
  "payment_method": "balance", // or "wechat", "alipay"
  "return_url": "https://yourapp.com/payment/return" // For Alipay
}
```

### 3. Payment Processing

- **Balance Payment**: Immediate processing, deducts from user balance
- **WeChat Pay**: Returns QR code or prepay data for SDK
- **Alipay**: Returns payment URL for redirect

### 4. Payment Callback

External payment gateways call back to confirm payment:

```javascript
// POST /payment/callback?method=alipay
{
  "out_trade_no": "ORD20240120123456",
  "trade_no": "2024012022001412345",
  "trade_status": "TRADE_SUCCESS",
  "total_amount": "30.00"
}
```

## WeChat Pay Configuration

### 1. Prerequisites

- WeChat merchant account
- API certificates
- App ID (for mini-program or official account)

### 2. Configuration

Update payment configuration via admin API:

```bash
# App ID
curl -X PUT http://localhost:3000/api/v1/payment/admin/config/wechat \
  -H "Authorization: Bearer <admin_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "config_key": "app_id",
    "config_value": "wx1234567890abcdef",
    "is_encrypted": false
  }'

# Merchant ID
curl -X PUT http://localhost:3000/api/v1/payment/admin/config/wechat \
  -H "Authorization: Bearer <admin_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "config_key": "mch_id",
    "config_value": "1234567890",
    "is_encrypted": false
  }'

# API Key (encrypted)
curl -X PUT http://localhost:3000/api/v1/payment/admin/config/wechat \
  -H "Authorization: Bearer <admin_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "config_key": "api_key",
    "config_value": "your-api-key-here",
    "is_encrypted": true
  }'

# Certificate Path
curl -X PUT http://localhost:3000/api/v1/payment/admin/config/wechat \
  -H "Authorization: Bearer <admin_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "config_key": "cert_path",
    "config_value": "/path/to/cert/apiclient_cert.pem",
    "is_encrypted": false
  }'

# Callback URL
curl -X PUT http://localhost:3000/api/v1/payment/admin/config/wechat \
  -H "Authorization: Bearer <admin_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "config_key": "notify_url",
    "config_value": "https://yourdomain.com/payment/callback?method=wechat",
    "is_encrypted": false
  }'
```

### 3. Implementation

To complete WeChat Pay integration, implement the actual API calls in `payment_service.rs`:

```rust
async fn process_wechat_payment(
    db: &DbPool,
    order: &PaymentOrder,
    transaction_id: &Uuid,
) -> Result<PaymentResponse, AppError> {
    // Get configuration
    let config = Self::get_payment_config(db, PaymentMethod::Wechat).await?;
    
    // TODO: Implement actual WeChat Pay API call
    // 1. Create unified order
    // 2. Get prepay_id
    // 3. Generate payment parameters for client SDK
    
    // Current implementation returns mock data
}
```

## Alipay Configuration

### 1. Prerequisites

- Alipay merchant account
- App ID
- RSA private/public key pair

### 2. Configuration

```bash
# App ID
curl -X PUT http://localhost:3000/api/v1/payment/admin/config/alipay \
  -H "Authorization: Bearer <admin_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "config_key": "app_id",
    "config_value": "2021001234567890",
    "is_encrypted": false
  }'

# Private Key (encrypted)
curl -X PUT http://localhost:3000/api/v1/payment/admin/config/alipay \
  -H "Authorization: Bearer <admin_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "config_key": "private_key",
    "config_value": "your-rsa-private-key",
    "is_encrypted": true
  }'

# Alipay Public Key (encrypted)
curl -X PUT http://localhost:3000/api/v1/payment/admin/config/alipay \
  -H "Authorization: Bearer <admin_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "config_key": "public_key",
    "config_value": "alipay-public-key",
    "is_encrypted": true
  }'

# Callback URLs
curl -X PUT http://localhost:3000/api/v1/payment/admin/config/alipay \
  -H "Authorization: Bearer <admin_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "config_key": "notify_url",
    "config_value": "https://yourdomain.com/payment/callback?method=alipay",
    "is_encrypted": false
  }'

curl -X PUT http://localhost:3000/api/v1/payment/admin/config/alipay \
  -H "Authorization: Bearer <admin_token>" \
  -H "Content-Type: application/json" \
  -d '{
    "config_key": "return_url",
    "config_value": "https://yourdomain.com/payment/success",
    "is_encrypted": false
  }'
```

### 3. Implementation

To complete Alipay integration, implement the actual API calls in `payment_service.rs`:

```rust
async fn process_alipay_payment(
    db: &DbPool,
    order: &PaymentOrder,
    transaction_id: &Uuid,
    return_url: Option<String>,
) -> Result<PaymentResponse, AppError> {
    // Get configuration
    let config = Self::get_payment_config(db, PaymentMethod::Alipay).await?;
    
    // TODO: Implement actual Alipay API call
    // 1. Build payment request
    // 2. Sign request
    // 3. Generate payment URL or QR code
    
    // Current implementation returns mock data
}
```

## Refund Process

### 1. User Requests Refund

```javascript
// POST /api/v1/payment/refunds
{
  "order_id": "order_uuid",
  "refund_amount": "30.00",
  "refund_reason": "服务未提供"
}
```

### 2. Admin Reviews Refund

```javascript
// PUT /api/v1/payment/admin/refunds/:refund_id/review
{
  "approved": true,
  "review_notes": "同意退款"
}
```

### 3. Refund Processing

- **Balance Refunds**: Immediate, adds amount back to user balance
- **Third-party Refunds**: Requires API call to payment provider

## Testing

### 1. Balance Payment Test

```bash
# Create test user with balance
mysql -u tcm_user -p tcm_telemedicine
INSERT INTO user_balances (id, user_id, balance) 
VALUES (UUID(), 'test-user-uuid', 100.00);

# Create order
curl -X POST http://localhost:3000/api/v1/payment/orders \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "user_id": "test-user-uuid",
    "order_type": "consultation",
    "amount": "30.00"
  }'

# Pay with balance
curl -X POST http://localhost:3000/api/v1/payment/pay \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{
    "order_id": "returned-order-id",
    "payment_method": "balance"
  }'
```

### 2. Mock External Payment

```bash
# Create order and initiate payment
# ... (same as above)

# Simulate payment callback
curl -X POST http://localhost:3000/payment/callback?method=alipay \
  -H "Content-Type: application/json" \
  -d '{
    "out_trade_no": "order-number-from-order",
    "trade_no": "mock-trade-number",
    "trade_status": "TRADE_SUCCESS",
    "total_amount": "30.00"
  }'
```

## Production Considerations

### 1. Security

- Use HTTPS for all payment endpoints
- Validate callback signatures from payment providers
- Encrypt sensitive configuration values
- Implement request rate limiting
- Log all payment transactions

### 2. Reliability

- Implement idempotency for payment callbacks
- Add retry logic for failed third-party API calls
- Set up monitoring and alerts for payment failures
- Regular reconciliation with payment providers

### 3. Compliance

- Follow PCI DSS guidelines if handling card payments
- Comply with local financial regulations
- Implement proper refund policies
- Maintain audit logs

### 4. Performance

- Use database transactions for balance operations
- Implement caching for price configurations
- Queue heavy operations (like bulk refunds)
- Monitor payment processing times

## Troubleshooting

### Common Issues

1. **Order Expiration**
   - Orders expire after 2 hours by default
   - Adjust in `payment_service.rs` if needed

2. **Balance Insufficient**
   - Ensure user has sufficient balance before balance payment
   - Consider implementing top-up functionality

3. **Callback Failures**
   - Check callback URL is publicly accessible
   - Verify callback data format matches expectations
   - Check logs for signature validation errors

4. **Configuration Issues**
   - Verify all required configs are set
   - Check encrypted values are properly stored
   - Ensure certificates are accessible (for WeChat Pay)

### Debug Mode

Enable debug logging for payment operations:

```bash
RUST_LOG=backend=debug cargo run
```

This will log detailed information about payment processing.

## Support

For payment gateway specific issues:
- WeChat Pay: [微信支付文档](https://pay.weixin.qq.com/docs/)
- Alipay: [支付宝开放平台](https://opendocs.alipay.com/)

For system issues, check:
- Application logs
- Database payment_transactions table
- Payment gateway merchant dashboard