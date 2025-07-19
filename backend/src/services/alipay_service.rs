use crate::{
    config::database::DbPool,
    models::payment::*,
    utils::errors::AppError,
};
use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use chrono::{DateTime, Local, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct AlipayConfig {
    pub app_id: String,
    pub private_key: String,
    pub alipay_public_key: String,
    pub notify_url: String,
}

impl AlipayConfig {
    pub async fn from_db(db: &DbPool, config_id: Uuid) -> Result<Self, AppError> {
        let query = r#"
            SELECT app_id, private_key, public_key, notify_url
            FROM payment_configs
            WHERE id = ? AND provider = 'alipay' AND status = 'active'
        "#;
        
        let row = sqlx::query(query)
            .bind(config_id.to_string())
            .fetch_one(db)
            .await
            .map_err(|e| AppError::NotFound(format!("Alipay config not found: {}", e)))?;
        
        Ok(Self {
            app_id: sqlx::Row::get(&row, "app_id"),
            private_key: sqlx::Row::get(&row, "private_key"),
            alipay_public_key: sqlx::Row::get(&row, "public_key"),
            notify_url: sqlx::Row::get(&row, "notify_url"),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlipayBizContent {
    out_trade_no: String,
    total_amount: String,
    subject: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    body: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    product_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    quit_url: Option<String>, // For WAP payment
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AlipayRequest {
    app_id: String,
    method: String,
    format: String,
    charset: String,
    sign_type: String,
    sign: String,
    timestamp: String,
    version: String,
    notify_url: String,
    biz_content: String,
}

#[derive(Debug, Deserialize)]
pub struct AlipayResponse<T> {
    sign: String,
    #[serde(flatten)]
    response: T,
}

#[derive(Debug, Deserialize)]
pub struct AlipayTradeResponse {
    code: String,
    msg: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    out_trade_no: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    trade_no: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    qr_code: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AlipayNotification {
    notify_time: String,
    notify_type: String,
    notify_id: String,
    sign_type: String,
    sign: String,
    out_trade_no: String,
    trade_no: String,
    trade_status: String,
    total_amount: String,
    #[serde(flatten)]
    extra: BTreeMap<String, String>,
}

pub struct AlipayService;

impl AlipayService {
    const GATEWAY_URL: &'static str = "https://openapi.alipay.com/gateway.do";
    const SANDBOX_GATEWAY_URL: &'static str = "https://openapi.alipaydev.com/gateway.do";
    
    /// Create Alipay order
    pub async fn create_order(
        config: &AlipayConfig,
        order: &PaymentOrder,
        method: &str,
        product_code: Option<&str>,
    ) -> Result<AlipayTradeResponse, AppError> {
        let biz_content = AlipayBizContent {
            out_trade_no: order.order_no.clone(),
            total_amount: order.amount.to_string(),
            subject: format!("TCM-{}", order.order_type),
            body: Some(format!("TCM Telemedicine - {}", order.order_type)),
            product_code: product_code.map(|s| s.to_string()),
            quit_url: None,
        };
        
        let params = Self::build_request_params(
            config,
            method,
            &biz_content,
        )?;
        
        let client = Client::new();
        let response = client
            .post(Self::GATEWAY_URL)
            .form(&params)
            .send()
            .await
            .map_err(|e| AppError::InternalServerError(format!("Alipay request failed: {}", e)))?;
        
        let response_body = response
            .text()
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to read response: {}", e)))?;
        
        let response_json: Value = serde_json::from_str(&response_body)
            .map_err(|e| AppError::InternalServerError(format!("Failed to parse response: {}", e)))?;
        
        // Extract the response key (e.g., "alipay_trade_page_pay_response")
        let response_key = format!("{}_response", method.replace(".", "_"));
        let trade_response: AlipayTradeResponse = serde_json::from_value(
            response_json.get(&response_key)
                .ok_or_else(|| AppError::InternalServerError("Invalid response format".to_string()))?
                .clone()
        ).map_err(|e| AppError::InternalServerError(format!("Failed to parse trade response: {}", e)))?;
        
        if trade_response.code != "10000" {
            return Err(AppError::BadRequest(format!(
                "Alipay error: {} - {}",
                trade_response.code, trade_response.msg
            )));
        }
        
        Ok(trade_response)
    }
    
    /// Query order status
    pub async fn query_order(
        config: &AlipayConfig,
        out_trade_no: &str,
    ) -> Result<BTreeMap<String, String>, AppError> {
        let mut biz_content = BTreeMap::new();
        biz_content.insert("out_trade_no".to_string(), out_trade_no.to_string());
        
        let params = Self::build_request_params_map(
            config,
            "alipay.trade.query",
            &biz_content,
        )?;
        
        let client = Client::new();
        let response = client
            .post(Self::GATEWAY_URL)
            .form(&params)
            .send()
            .await
            .map_err(|e| AppError::InternalServerError(format!("Alipay query failed: {}", e)))?;
        
        let response_body = response
            .text()
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to read response: {}", e)))?;
        
        let response_json: Value = serde_json::from_str(&response_body)
            .map_err(|e| AppError::InternalServerError(format!("Failed to parse response: {}", e)))?;
        
        let query_response = response_json.get("alipay_trade_query_response")
            .ok_or_else(|| AppError::InternalServerError("Invalid response format".to_string()))?;
        
        let result: BTreeMap<String, String> = serde_json::from_value(query_response.clone())
            .map_err(|e| AppError::InternalServerError(format!("Failed to parse query response: {}", e)))?;
        
        Ok(result)
    }
    
    /// Process refund
    pub async fn refund(
        config: &AlipayConfig,
        trade_no: &str,
        refund_amount: &str,
        out_request_no: &str,
        refund_reason: &str,
    ) -> Result<BTreeMap<String, String>, AppError> {
        let mut biz_content = BTreeMap::new();
        biz_content.insert("trade_no".to_string(), trade_no.to_string());
        biz_content.insert("refund_amount".to_string(), refund_amount.to_string());
        biz_content.insert("out_request_no".to_string(), out_request_no.to_string());
        biz_content.insert("refund_reason".to_string(), refund_reason.to_string());
        
        let params = Self::build_request_params_map(
            config,
            "alipay.trade.refund",
            &biz_content,
        )?;
        
        let client = Client::new();
        let response = client
            .post(Self::GATEWAY_URL)
            .form(&params)
            .send()
            .await
            .map_err(|e| AppError::InternalServerError(format!("Alipay refund failed: {}", e)))?;
        
        let response_body = response
            .text()
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to read response: {}", e)))?;
        
        let response_json: Value = serde_json::from_str(&response_body)
            .map_err(|e| AppError::InternalServerError(format!("Failed to parse response: {}", e)))?;
        
        let refund_response = response_json.get("alipay_trade_refund_response")
            .ok_or_else(|| AppError::InternalServerError("Invalid response format".to_string()))?;
        
        let result: BTreeMap<String, String> = serde_json::from_value(refund_response.clone())
            .map_err(|e| AppError::InternalServerError(format!("Failed to parse refund response: {}", e)))?;
        
        Ok(result)
    }
    
    /// Verify notification signature
    pub fn verify_notification(
        notification: &BTreeMap<String, String>,
        public_key: &str,
    ) -> Result<bool, AppError> {
        let mut params = notification.clone();
        let sign = params.remove("sign")
            .ok_or_else(|| AppError::BadRequest("Missing sign".to_string()))?;
        let sign_type = params.remove("sign_type");
        
        let content = Self::build_sign_content(&params);
        Self::verify_sign(&content, &sign, public_key)
    }
    
    /// Build request parameters
    fn build_request_params<T: Serialize>(
        config: &AlipayConfig,
        method: &str,
        biz_content: &T,
    ) -> Result<BTreeMap<String, String>, AppError> {
        let biz_content_str = serde_json::to_string(biz_content)
            .map_err(|e| AppError::InternalServerError(format!("Failed to serialize biz_content: {}", e)))?;
        
        let mut params = BTreeMap::new();
        params.insert("app_id".to_string(), config.app_id.clone());
        params.insert("method".to_string(), method.to_string());
        params.insert("format".to_string(), "JSON".to_string());
        params.insert("charset".to_string(), "utf-8".to_string());
        params.insert("sign_type".to_string(), "RSA2".to_string());
        params.insert("timestamp".to_string(), Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
        params.insert("version".to_string(), "1.0".to_string());
        params.insert("notify_url".to_string(), config.notify_url.clone());
        params.insert("biz_content".to_string(), biz_content_str);
        
        let sign = Self::generate_sign(&params, &config.private_key)?;
        params.insert("sign".to_string(), sign);
        
        Ok(params)
    }
    
    /// Build request parameters from map
    fn build_request_params_map(
        config: &AlipayConfig,
        method: &str,
        biz_content: &BTreeMap<String, String>,
    ) -> Result<BTreeMap<String, String>, AppError> {
        let biz_content_str = serde_json::to_string(biz_content)
            .map_err(|e| AppError::InternalServerError(format!("Failed to serialize biz_content: {}", e)))?;
        
        let mut params = BTreeMap::new();
        params.insert("app_id".to_string(), config.app_id.clone());
        params.insert("method".to_string(), method.to_string());
        params.insert("format".to_string(), "JSON".to_string());
        params.insert("charset".to_string(), "utf-8".to_string());
        params.insert("sign_type".to_string(), "RSA2".to_string());
        params.insert("timestamp".to_string(), Local::now().format("%Y-%m-%d %H:%M:%S").to_string());
        params.insert("version".to_string(), "1.0".to_string());
        params.insert("notify_url".to_string(), config.notify_url.clone());
        params.insert("biz_content".to_string(), biz_content_str);
        
        let sign = Self::generate_sign(&params, &config.private_key)?;
        params.insert("sign".to_string(), sign);
        
        Ok(params)
    }
    
    /// Build sign content
    fn build_sign_content(params: &BTreeMap<String, String>) -> String {
        params.iter()
            .filter(|(k, v)| !v.is_empty() && *k != "sign" && *k != "sign_type")
            .map(|(k, v)| format!("{}={}", k, v))
            .collect::<Vec<_>>()
            .join("&")
    }
    
    /// Generate RSA2 signature
    fn generate_sign(
        params: &BTreeMap<String, String>,
        private_key: &str,
    ) -> Result<String, AppError> {
        use rsa::{RsaPrivateKey, pkcs1::DecodeRsaPrivateKey, pkcs1v15::SigningKey};
        use rsa::signature::{Signer, SignatureEncoding};
        
        let content = Self::build_sign_content(params);
        
        // Parse private key
        let private_key = RsaPrivateKey::from_pkcs1_pem(private_key)
            .map_err(|e| AppError::InternalServerError(format!("Failed to parse private key: {}", e)))?;
        
        let signing_key = SigningKey::<Sha256>::new(private_key);
        let signature = signing_key.sign(content.as_bytes());
        
        Ok(BASE64.encode(signature.to_bytes()))
    }
    
    /// Verify RSA2 signature
    fn verify_sign(
        content: &str,
        sign: &str,
        public_key: &str,
    ) -> Result<bool, AppError> {
        use rsa::{RsaPublicKey, pkcs1::DecodeRsaPublicKey, pkcs1v15::VerifyingKey};
        use rsa::signature::{Verifier, Signature};
        
        // Decode base64 signature
        let signature_bytes = BASE64.decode(sign)
            .map_err(|e| AppError::BadRequest(format!("Invalid signature format: {}", e)))?;
        
        // Parse public key
        let public_key = RsaPublicKey::from_pkcs1_pem(public_key)
            .map_err(|e| AppError::InternalServerError(format!("Failed to parse public key: {}", e)))?;
        
        let verifying_key = VerifyingKey::<Sha256>::new(public_key);
        
        // Create signature from bytes
        let signature = rsa::pkcs1v15::Signature::try_from(signature_bytes.as_slice())
            .map_err(|e| AppError::BadRequest(format!("Invalid signature: {}", e)))?;
        
        Ok(verifying_key.verify(content.as_bytes(), &signature).is_ok())
    }
    
    /// Generate payment URL for PC web payment
    pub fn generate_page_pay_url(
        config: &AlipayConfig,
        order: &PaymentOrder,
    ) -> Result<String, AppError> {
        let biz_content = AlipayBizContent {
            out_trade_no: order.order_no.clone(),
            total_amount: order.amount.to_string(),
            subject: format!("TCM-{}", order.order_type),
            body: Some(format!("TCM Telemedicine - {}", order.order_type)),
            product_code: Some("FAST_INSTANT_TRADE_PAY".to_string()),
            quit_url: None,
        };
        
        let params = Self::build_request_params(
            config,
            "alipay.trade.page.pay",
            &biz_content,
        )?;
        
        let query_string = params.iter()
            .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
            .collect::<Vec<_>>()
            .join("&");
        
        Ok(format!("{}?{}", Self::GATEWAY_URL, query_string))
    }
}