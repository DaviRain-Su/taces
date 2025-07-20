use crate::{
    config::database::DbPool,
    utils::errors::AppError,
};
use chrono::Utc;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct SmsConfig {
    pub provider: SmsProvider,
    pub access_key: String,
    pub secret_key: String,
    pub sign_name: String,
    pub region: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SmsProvider {
    Aliyun,
    Tencent,
    Twilio,
}

impl SmsConfig {
    pub fn from_env() -> Option<Self> {
        let provider = match std::env::var("SMS_PROVIDER").ok()?.as_str() {
            "aliyun" => SmsProvider::Aliyun,
            "tencent" => SmsProvider::Tencent,
            "twilio" => SmsProvider::Twilio,
            _ => return None,
        };
        
        Some(Self {
            provider,
            access_key: std::env::var("SMS_ACCESS_KEY").ok()?,
            secret_key: std::env::var("SMS_SECRET_KEY").ok()?,
            sign_name: std::env::var("SMS_SIGN_NAME").unwrap_or_else(|_| "香河香草中医".to_string()),
            region: std::env::var("SMS_REGION").ok(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SmsMessage {
    pub phone: String,
    pub template_code: String,
    pub template_params: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SmsSendResult {
    pub success: bool,
    pub message_id: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SmsTemplate {
    pub id: Uuid,
    pub code: String,
    pub name: String,
    pub content: String,
    pub provider: String,
    pub template_id: String,
    pub params: Vec<String>,
}

pub struct SmsService;

impl SmsService {
    /// Send SMS message
    pub async fn send_sms(
        config: &SmsConfig,
        message: SmsMessage,
    ) -> Result<SmsSendResult, AppError> {
        match config.provider {
            SmsProvider::Aliyun => Self::send_aliyun_sms(config, message).await,
            SmsProvider::Tencent => Self::send_tencent_sms(config, message).await,
            SmsProvider::Twilio => Self::send_twilio_sms(config, message).await,
        }
    }
    
    /// Send appointment reminder SMS
    pub async fn send_appointment_reminder(
        config: &SmsConfig,
        phone: &str,
        patient_name: &str,
        doctor_name: &str,
        appointment_time: &str,
    ) -> Result<SmsSendResult, AppError> {
        let mut params = HashMap::new();
        params.insert("patient_name".to_string(), patient_name.to_string());
        params.insert("doctor_name".to_string(), doctor_name.to_string());
        params.insert("time".to_string(), appointment_time.to_string());
        
        let message = SmsMessage {
            phone: phone.to_string(),
            template_code: "APPOINTMENT_REMINDER".to_string(),
            template_params: params,
        };
        
        Self::send_sms(config, message).await
    }
    
    /// Send prescription ready SMS
    pub async fn send_prescription_ready(
        config: &SmsConfig,
        phone: &str,
        patient_name: &str,
        prescription_code: &str,
    ) -> Result<SmsSendResult, AppError> {
        let mut params = HashMap::new();
        params.insert("patient_name".to_string(), patient_name.to_string());
        params.insert("prescription_code".to_string(), prescription_code.to_string());
        
        let message = SmsMessage {
            phone: phone.to_string(),
            template_code: "PRESCRIPTION_READY".to_string(),
            template_params: params,
        };
        
        Self::send_sms(config, message).await
    }
    
    /// Send verification code SMS
    pub async fn send_verification_code(
        config: &SmsConfig,
        phone: &str,
        code: &str,
    ) -> Result<SmsSendResult, AppError> {
        let mut params = HashMap::new();
        params.insert("code".to_string(), code.to_string());
        
        let message = SmsMessage {
            phone: phone.to_string(),
            template_code: "VERIFICATION_CODE".to_string(),
            template_params: params,
        };
        
        Self::send_sms(config, message).await
    }
    
    /// Send Aliyun SMS
    async fn send_aliyun_sms(
        config: &SmsConfig,
        message: SmsMessage,
    ) -> Result<SmsSendResult, AppError> {
        // This is a simplified implementation
        // Production should use proper Aliyun SDK with signature calculation
        
        let client = Client::new();
        let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
        
        let mut params = HashMap::new();
        params.insert("Action", "SendSms".to_string());
        params.insert("Version", "2017-05-25".to_string());
        params.insert("RegionId", config.region.clone().unwrap_or_else(|| "cn-hangzhou".to_string()));
        params.insert("PhoneNumbers", message.phone);
        params.insert("SignName", config.sign_name.clone());
        params.insert("TemplateCode", Self::get_aliyun_template_code(&message.template_code));
        params.insert("TemplateParam", serde_json::to_string(&message.template_params).unwrap());
        params.insert("AccessKeyId", config.access_key.clone());
        params.insert("Timestamp", timestamp);
        params.insert("Format", "JSON".to_string());
        
        // In production, add proper signature calculation here
        
        let response = client
            .post("https://dysmsapi.aliyuncs.com/")
            .form(&params)
            .send()
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to send SMS: {}", e)))?;
        
        let status = response.status();
        let body = response.text().await
            .map_err(|e| AppError::InternalServerError(format!("Failed to read response: {}", e)))?;
        
        if status.is_success() {
            Ok(SmsSendResult {
                success: true,
                message_id: Some(Uuid::new_v4().to_string()),
                error_message: None,
            })
        } else {
            Ok(SmsSendResult {
                success: false,
                message_id: None,
                error_message: Some(body),
            })
        }
    }
    
    /// Send Tencent SMS
    async fn send_tencent_sms(
        config: &SmsConfig,
        message: SmsMessage,
    ) -> Result<SmsSendResult, AppError> {
        // This is a simplified implementation
        // Production should use proper Tencent Cloud SDK
        
        let client = Client::new();
        let url = "https://sms.tencentcloudapi.com/";
        
        let body = serde_json::json!({
            "PhoneNumberSet": [message.phone],
            "SmsSdkAppId": config.access_key,
            "SignName": config.sign_name,
            "TemplateId": Self::get_tencent_template_id(&message.template_code),
            "TemplateParamSet": message.template_params.values().collect::<Vec<_>>(),
        });
        
        // In production, add proper signature calculation here
        
        let response = client
            .post(url)
            .json(&body)
            .send()
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to send SMS: {}", e)))?;
        
        let status = response.status();
        
        if status.is_success() {
            Ok(SmsSendResult {
                success: true,
                message_id: Some(Uuid::new_v4().to_string()),
                error_message: None,
            })
        } else {
            Ok(SmsSendResult {
                success: false,
                message_id: None,
                error_message: Some("Failed to send SMS".to_string()),
            })
        }
    }
    
    /// Send Twilio SMS
    async fn send_twilio_sms(
        config: &SmsConfig,
        message: SmsMessage,
    ) -> Result<SmsSendResult, AppError> {
        let client = Client::new();
        let url = format!(
            "https://api.twilio.com/2010-04-01/Accounts/{}/Messages.json",
            config.access_key
        );
        
        let body_text = Self::format_template_message(&message.template_code, &message.template_params);
        
        let mut form = HashMap::new();
        form.insert("To", message.phone);
        form.insert("From", config.sign_name.clone()); // Twilio phone number
        form.insert("Body", body_text);
        
        let response = client
            .post(&url)
            .basic_auth(&config.access_key, Some(&config.secret_key))
            .form(&form)
            .send()
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to send SMS: {}", e)))?;
        
        let status = response.status();
        
        if status.is_success() {
            let response_body: serde_json::Value = response.json().await
                .map_err(|e| AppError::InternalServerError(format!("Failed to parse response: {}", e)))?;
            
            Ok(SmsSendResult {
                success: true,
                message_id: response_body["sid"].as_str().map(|s| s.to_string()),
                error_message: None,
            })
        } else {
            Ok(SmsSendResult {
                success: false,
                message_id: None,
                error_message: Some("Failed to send SMS".to_string()),
            })
        }
    }
    
    /// Get Aliyun template code
    fn get_aliyun_template_code(template_code: &str) -> String {
        match template_code {
            "APPOINTMENT_REMINDER" => "SMS_123456789".to_string(),
            "PRESCRIPTION_READY" => "SMS_987654321".to_string(),
            "VERIFICATION_CODE" => "SMS_111111111".to_string(),
            _ => template_code.to_string(),
        }
    }
    
    /// Get Tencent template ID
    fn get_tencent_template_id(template_code: &str) -> String {
        match template_code {
            "APPOINTMENT_REMINDER" => "123456".to_string(),
            "PRESCRIPTION_READY" => "654321".to_string(),
            "VERIFICATION_CODE" => "111111".to_string(),
            _ => template_code.to_string(),
        }
    }
    
    /// Format template message for providers that don't support templates
    fn format_template_message(
        template_code: &str,
        params: &HashMap<String, String>,
    ) -> String {
        match template_code {
            "APPOINTMENT_REMINDER" => {
                format!(
                    "尊敬的{}，您预约的{}医生的就诊时间是{}，请准时到诊。",
                    params.get("patient_name").unwrap_or(&"患者".to_string()),
                    params.get("doctor_name").unwrap_or(&"".to_string()),
                    params.get("time").unwrap_or(&"".to_string())
                )
            }
            "PRESCRIPTION_READY" => {
                format!(
                    "尊敬的{}，您的处方（编号：{}）已开具完成，请查看详情。",
                    params.get("patient_name").unwrap_or(&"患者".to_string()),
                    params.get("prescription_code").unwrap_or(&"".to_string())
                )
            }
            "VERIFICATION_CODE" => {
                format!(
                    "您的验证码是：{}，请在5分钟内使用。",
                    params.get("code").unwrap_or(&"".to_string())
                )
            }
            _ => "香河香草中医诊所提醒您".to_string(),
        }
    }
    
    /// Store SMS record in database
    pub async fn store_sms_record(
        db: &DbPool,
        phone: &str,
        template_code: &str,
        params: &HashMap<String, String>,
        result: &SmsSendResult,
    ) -> Result<(), AppError> {
        let query = r#"
            INSERT INTO sms_records (
                id, phone, template_code, template_params, 
                status, message_id, error_message, sent_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#;
        
        sqlx::query(query)
            .bind(Uuid::new_v4().to_string())
            .bind(phone)
            .bind(template_code)
            .bind(serde_json::to_string(params).unwrap())
            .bind(if result.success { "success" } else { "failed" })
            .bind(&result.message_id)
            .bind(&result.error_message)
            .bind(Utc::now())
            .execute(db)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to store SMS record: {}", e)))?;
        
        Ok(())
    }
}