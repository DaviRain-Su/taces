use crate::{
    config::database::DbPool,
    models::payment::*,
    utils::errors::AppError,
};
use chrono::{DateTime, Utc};
use hmac::{Hmac, Mac};
use md5::Digest as Md5Digest;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::Sha256;
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct WeChatPayConfig {
    pub app_id: String,
    pub mch_id: String,
    pub api_key: String,
    pub notify_url: String,
}

impl WeChatPayConfig {
    pub async fn from_db(db: &DbPool, config_id: Uuid) -> Result<Self, AppError> {
        let query = r#"
            SELECT app_id, merchant_id, api_key, notify_url
            FROM payment_configs
            WHERE id = ? AND provider = 'wechat' AND status = 'active'
        "#;
        
        let row = sqlx::query(query)
            .bind(config_id.to_string())
            .fetch_one(db)
            .await
            .map_err(|e| AppError::NotFound(format!("WeChat Pay config not found: {}", e)))?;
        
        Ok(Self {
            app_id: sqlx::Row::get(&row, "app_id"),
            mch_id: sqlx::Row::get(&row, "merchant_id"),
            api_key: sqlx::Row::get(&row, "api_key"),
            notify_url: sqlx::Row::get(&row, "notify_url"),
        })
    }
}

#[derive(Debug, Serialize)]
pub struct WeChatPayRequest {
    appid: String,
    mch_id: String,
    nonce_str: String,
    sign: String,
    body: String,
    out_trade_no: String,
    total_fee: i32, // Amount in cents
    spbill_create_ip: String,
    notify_url: String,
    trade_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    openid: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct WeChatPayResponse {
    return_code: String,
    return_msg: Option<String>,
    #[serde(flatten)]
    data: Option<WeChatPayResponseData>,
}

#[derive(Debug, Deserialize)]
pub struct WeChatPayResponseData {
    result_code: String,
    err_code: Option<String>,
    err_code_des: Option<String>,
    prepay_id: Option<String>,
    code_url: Option<String>, // QR code URL for native payment
}

#[derive(Debug, Deserialize)]
pub struct WeChatPayNotification {
    return_code: String,
    result_code: String,
    out_trade_no: String,
    transaction_id: String,
    total_fee: i32,
    time_end: String,
    #[serde(flatten)]
    extra: HashMap<String, String>,
}

pub struct WeChatPayService;

impl WeChatPayService {
    const API_URL: &'static str = "https://api.mch.weixin.qq.com";
    
    /// Create unified order for WeChat Pay
    pub async fn create_order(
        db: &DbPool,
        config: &WeChatPayConfig,
        order: &PaymentOrder,
        trade_type: &str,
        openid: Option<String>,
    ) -> Result<WeChatPayResponse, AppError> {
        let client = Client::new();
        let nonce_str = Self::generate_nonce_str();
        
        let mut params = HashMap::new();
        params.insert("appid", config.app_id.clone());
        params.insert("mch_id", config.mch_id.clone());
        params.insert("nonce_str", nonce_str.clone());
        params.insert("body", format!("TCM-{}", order.order_type));
        params.insert("out_trade_no", order.order_no.clone());
        params.insert("total_fee", order.amount.to_string());
        params.insert("spbill_create_ip", "127.0.0.1".to_string());
        params.insert("notify_url", config.notify_url.clone());
        params.insert("trade_type", trade_type.to_string());
        
        if let Some(openid) = openid {
            params.insert("openid", openid);
        }
        
        let sign = Self::generate_sign(&params, &config.api_key);
        
        let request = WeChatPayRequest {
            appid: config.app_id.clone(),
            mch_id: config.mch_id.clone(),
            nonce_str,
            sign,
            body: format!("TCM-{}", order.order_type),
            out_trade_no: order.order_no.clone(),
            total_fee: (order.amount * rust_decimal::Decimal::from(100)).to_string().parse().unwrap(),
            spbill_create_ip: "127.0.0.1".to_string(),
            notify_url: config.notify_url.clone(),
            trade_type: trade_type.to_string(),
            openid,
        };
        
        let xml = Self::struct_to_xml(&request)?;
        
        let response = client
            .post(format!("{}/pay/unifiedorder", Self::API_URL))
            .header("Content-Type", "text/xml")
            .body(xml)
            .send()
            .await
            .map_err(|e| AppError::InternalServerError(format!("WeChat Pay request failed: {}", e)))?;
        
        let response_body = response
            .text()
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to read response: {}", e)))?;
        
        Self::xml_to_struct(&response_body)
    }
    
    /// Query order status
    pub async fn query_order(
        config: &WeChatPayConfig,
        out_trade_no: &str,
    ) -> Result<HashMap<String, String>, AppError> {
        let client = Client::new();
        let nonce_str = Self::generate_nonce_str();
        
        let mut params = HashMap::new();
        params.insert("appid", config.app_id.clone());
        params.insert("mch_id", config.mch_id.clone());
        params.insert("out_trade_no", out_trade_no.to_string());
        params.insert("nonce_str", nonce_str);
        
        let sign = Self::generate_sign(&params, &config.api_key);
        params.insert("sign", sign);
        
        let xml = Self::map_to_xml(&params)?;
        
        let response = client
            .post(format!("{}/pay/orderquery", Self::API_URL))
            .header("Content-Type", "text/xml")
            .body(xml)
            .send()
            .await
            .map_err(|e| AppError::InternalServerError(format!("WeChat Pay query failed: {}", e)))?;
        
        let response_body = response
            .text()
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to read response: {}", e)))?;
        
        Self::xml_to_map(&response_body)
    }
    
    /// Process refund
    pub async fn refund(
        config: &WeChatPayConfig,
        transaction_id: &str,
        out_refund_no: &str,
        total_fee: i32,
        refund_fee: i32,
        reason: &str,
    ) -> Result<HashMap<String, String>, AppError> {
        let client = Client::new();
        let nonce_str = Self::generate_nonce_str();
        
        let mut params = HashMap::new();
        params.insert("appid", config.app_id.clone());
        params.insert("mch_id", config.mch_id.clone());
        params.insert("nonce_str", nonce_str);
        params.insert("transaction_id", transaction_id.to_string());
        params.insert("out_refund_no", out_refund_no.to_string());
        params.insert("total_fee", total_fee.to_string());
        params.insert("refund_fee", refund_fee.to_string());
        params.insert("refund_desc", reason.to_string());
        
        let sign = Self::generate_sign(&params, &config.api_key);
        params.insert("sign", sign);
        
        let xml = Self::map_to_xml(&params)?;
        
        // Note: Refund API requires client certificate
        // This is a simplified version - production should include cert handling
        let response = client
            .post(format!("{}/secapi/pay/refund", Self::API_URL))
            .header("Content-Type", "text/xml")
            .body(xml)
            .send()
            .await
            .map_err(|e| AppError::InternalServerError(format!("WeChat Pay refund failed: {}", e)))?;
        
        let response_body = response
            .text()
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to read response: {}", e)))?;
        
        Self::xml_to_map(&response_body)
    }
    
    /// Verify notification signature
    pub fn verify_notification(
        notification: &HashMap<String, String>,
        api_key: &str,
    ) -> bool {
        let mut params = notification.clone();
        let received_sign = params.remove("sign").unwrap_or_default();
        
        let calculated_sign = Self::generate_sign(&params, api_key);
        calculated_sign == received_sign
    }
    
    /// Generate payment parameters for frontend
    pub fn generate_payment_params(
        config: &WeChatPayConfig,
        prepay_id: &str,
    ) -> HashMap<String, String> {
        let timestamp = Utc::now().timestamp().to_string();
        let nonce_str = Self::generate_nonce_str();
        let package = format!("prepay_id={}", prepay_id);
        
        let mut params = HashMap::new();
        params.insert("appId", config.app_id.clone());
        params.insert("timeStamp", timestamp.clone());
        params.insert("nonceStr", nonce_str.clone());
        params.insert("package", package.clone());
        params.insert("signType", "MD5".to_string());
        
        let sign = Self::generate_sign(&params, &config.api_key);
        
        params.insert("paySign", sign);
        params
    }
    
    /// Generate nonce string
    fn generate_nonce_str() -> String {
        Uuid::new_v4().to_string().replace("-", "")
    }
    
    /// Generate sign using MD5
    fn generate_sign(params: &HashMap<String, String>, api_key: &str) -> String {
        let mut keys: Vec<&String> = params.keys().collect();
        keys.sort();
        
        let mut sign_str = String::new();
        for key in keys {
            if let Some(value) = params.get(key) {
                if !value.is_empty() && key != "sign" {
                    sign_str.push_str(&format!("{}={}&", key, value));
                }
            }
        }
        sign_str.push_str(&format!("key={}", api_key));
        
        let mut hasher = md5::Md5::new();
        hasher.update(sign_str.as_bytes());
        let result = hasher.finalize();
        format!("{:X}", result)
    }
    
    /// Convert struct to XML
    fn struct_to_xml<T: Serialize>(data: &T) -> Result<String, AppError> {
        let json_value = serde_json::to_value(data)
            .map_err(|e| AppError::InternalServerError(format!("Failed to serialize: {}", e)))?;
        
        let mut xml = String::from("<xml>");
        
        if let serde_json::Value::Object(map) = json_value {
            for (key, value) in map {
                if let serde_json::Value::String(s) = value {
                    xml.push_str(&format!("<{}><![CDATA[{}]]></{}>", key, s, key));
                } else {
                    xml.push_str(&format!("<{}>{}</{}>", key, value, key));
                }
            }
        }
        
        xml.push_str("</xml>");
        Ok(xml)
    }
    
    /// Convert map to XML
    fn map_to_xml(params: &HashMap<String, String>) -> Result<String, AppError> {
        let mut xml = String::from("<xml>");
        
        for (key, value) in params {
            xml.push_str(&format!("<{}><![CDATA[{}]]></{}>", key, value, key));
        }
        
        xml.push_str("</xml>");
        Ok(xml)
    }
    
    /// Parse XML to struct
    fn xml_to_struct(xml: &str) -> Result<WeChatPayResponse, AppError> {
        // Simplified XML parsing - production should use proper XML parser
        let map = Self::xml_to_map(xml)?;
        
        let response = WeChatPayResponse {
            return_code: map.get("return_code").cloned().unwrap_or_default(),
            return_msg: map.get("return_msg").cloned(),
            data: if map.get("return_code") == Some(&"SUCCESS".to_string()) {
                Some(WeChatPayResponseData {
                    result_code: map.get("result_code").cloned().unwrap_or_default(),
                    err_code: map.get("err_code").cloned(),
                    err_code_des: map.get("err_code_des").cloned(),
                    prepay_id: map.get("prepay_id").cloned(),
                    code_url: map.get("code_url").cloned(),
                })
            } else {
                None
            },
        };
        
        Ok(response)
    }
    
    /// Parse XML to HashMap
    fn xml_to_map(xml: &str) -> Result<HashMap<String, String>, AppError> {
        let mut map = HashMap::new();
        
        // Simple XML parsing - extract content between tags
        let re = regex::Regex::new(r"<(\w+)>(?:<!\[CDATA\[)?(.*?)(?:\]\]>)?</\1>")
            .map_err(|e| AppError::InternalServerError(format!("Regex error: {}", e)))?;
        
        for cap in re.captures_iter(xml) {
            if let (Some(key), Some(value)) = (cap.get(1), cap.get(2)) {
                map.insert(key.as_str().to_string(), value.as_str().to_string());
            }
        }
        
        Ok(map)
    }
}