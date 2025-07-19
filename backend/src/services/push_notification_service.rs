use crate::{
    config::database::DbPool,
    models::notification::*,
    utils::errors::AppError,
};
use chrono::{DateTime, Utc};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PushConfig {
    pub provider: PushProvider,
    pub api_key: String,
    pub api_secret: Option<String>,
    pub app_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PushProvider {
    FCM,      // Firebase Cloud Messaging (Android/iOS/Web)
    APNs,     // Apple Push Notification service
    Jpush,    // 极光推送 (Popular in China)
    Getui,    // 个推 (Popular in China)
}

impl PushConfig {
    pub fn from_env() -> Option<Self> {
        let provider = match std::env::var("PUSH_PROVIDER").ok()?.as_str() {
            "fcm" => PushProvider::FCM,
            "apns" => PushProvider::APNs,
            "jpush" => PushProvider::Jpush,
            "getui" => PushProvider::Getui,
            _ => return None,
        };
        
        Some(Self {
            provider,
            api_key: std::env::var("PUSH_API_KEY").ok()?,
            api_secret: std::env::var("PUSH_API_SECRET").ok(),
            app_id: std::env::var("PUSH_APP_ID").ok(),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PushMessage {
    pub device_tokens: Vec<String>,
    pub title: String,
    pub body: String,
    pub data: Option<HashMap<String, String>>,
    pub badge: Option<i32>,
    pub sound: Option<String>,
    pub click_action: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PushResult {
    pub success: bool,
    pub message_id: Option<String>,
    pub failed_tokens: Vec<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeviceToken {
    pub id: Uuid,
    pub user_id: Uuid,
    pub token: String,
    pub platform: String, // ios, android, web
    pub provider: String, // fcm, apns, jpush, getui
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub struct PushNotificationService;

impl PushNotificationService {
    /// Send push notification
    pub async fn send_push(
        config: &PushConfig,
        message: PushMessage,
    ) -> Result<PushResult, AppError> {
        match config.provider {
            PushProvider::FCM => Self::send_fcm_push(config, message).await,
            PushProvider::APNs => Self::send_apns_push(config, message).await,
            PushProvider::Jpush => Self::send_jpush_push(config, message).await,
            PushProvider::Getui => Self::send_getui_push(config, message).await,
        }
    }
    
    /// Send appointment reminder push notification
    pub async fn send_appointment_reminder_push(
        config: &PushConfig,
        device_tokens: Vec<String>,
        patient_name: &str,
        doctor_name: &str,
        appointment_time: &str,
    ) -> Result<PushResult, AppError> {
        let mut data = HashMap::new();
        data.insert("type".to_string(), "appointment_reminder".to_string());
        data.insert("patient_name".to_string(), patient_name.to_string());
        data.insert("doctor_name".to_string(), doctor_name.to_string());
        data.insert("appointment_time".to_string(), appointment_time.to_string());
        
        let message = PushMessage {
            device_tokens,
            title: "预约提醒".to_string(),
            body: format!("您预约的{}医生就诊时间是{}", doctor_name, appointment_time),
            data: Some(data),
            badge: Some(1),
            sound: Some("default".to_string()),
            click_action: Some("APPOINTMENT_DETAIL".to_string()),
        };
        
        Self::send_push(config, message).await
    }
    
    /// Send prescription ready push notification
    pub async fn send_prescription_ready_push(
        config: &PushConfig,
        device_tokens: Vec<String>,
        patient_name: &str,
        prescription_code: &str,
    ) -> Result<PushResult, AppError> {
        let mut data = HashMap::new();
        data.insert("type".to_string(), "prescription_ready".to_string());
        data.insert("prescription_code".to_string(), prescription_code.to_string());
        
        let message = PushMessage {
            device_tokens,
            title: "处方已开具".to_string(),
            body: format!("您的处方（{}）已开具完成", prescription_code),
            data: Some(data),
            badge: Some(1),
            sound: Some("default".to_string()),
            click_action: Some("PRESCRIPTION_DETAIL".to_string()),
        };
        
        Self::send_push(config, message).await
    }
    
    /// Send FCM push notification
    async fn send_fcm_push(
        config: &PushConfig,
        message: PushMessage,
    ) -> Result<PushResult, AppError> {
        let client = Client::new();
        let url = "https://fcm.googleapis.com/fcm/send";
        
        let payload = serde_json::json!({
            "registration_ids": message.device_tokens,
            "notification": {
                "title": message.title,
                "body": message.body,
                "sound": message.sound.unwrap_or_else(|| "default".to_string()),
                "badge": message.badge,
                "click_action": message.click_action,
            },
            "data": message.data,
            "priority": "high",
        });
        
        let response = client
            .post(url)
            .header("Authorization", format!("key={}", config.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to send FCM push: {}", e)))?;
        
        let status = response.status();
        let body = response.json::<serde_json::Value>().await
            .map_err(|e| AppError::InternalServerError(format!("Failed to parse response: {}", e)))?;
        
        if status.is_success() {
            let success = body["success"].as_u64().unwrap_or(0) > 0;
            let failed_tokens = body["results"]
                .as_array()
                .unwrap_or(&vec![])
                .iter()
                .enumerate()
                .filter_map(|(i, result)| {
                    if result["error"].is_string() {
                        message.device_tokens.get(i).cloned()
                    } else {
                        None
                    }
                })
                .collect();
            
            Ok(PushResult {
                success,
                message_id: body["multicast_id"].as_u64().map(|id| id.to_string()),
                failed_tokens,
                error_message: None,
            })
        } else {
            Ok(PushResult {
                success: false,
                message_id: None,
                failed_tokens: message.device_tokens.clone(),
                error_message: Some(format!("FCM error: {}", body)),
            })
        }
    }
    
    /// Send APNs push notification
    async fn send_apns_push(
        config: &PushConfig,
        message: PushMessage,
    ) -> Result<PushResult, AppError> {
        // Simplified implementation - production should use proper APNs HTTP/2 API
        // with JWT authentication or certificate-based authentication
        
        Ok(PushResult {
            success: false,
            message_id: None,
            failed_tokens: message.device_tokens.clone(),
            error_message: Some("APNs not implemented".to_string()),
        })
    }
    
    /// Send Jpush push notification
    async fn send_jpush_push(
        config: &PushConfig,
        message: PushMessage,
    ) -> Result<PushResult, AppError> {
        let client = Client::new();
        let url = "https://api.jpush.cn/v3/push";
        
        let auth = base64::encode(format!("{}:{}", 
            config.app_id.as_ref().unwrap_or(&String::new()),
            config.api_secret.as_ref().unwrap_or(&String::new())
        ));
        
        let payload = serde_json::json!({
            "platform": "all",
            "audience": {
                "registration_id": message.device_tokens,
            },
            "notification": {
                "android": {
                    "alert": message.body,
                    "title": message.title,
                    "extras": message.data,
                },
                "ios": {
                    "alert": message.body,
                    "sound": message.sound.unwrap_or_else(|| "default".to_string()),
                    "badge": message.badge,
                    "extras": message.data,
                },
            },
            "message": {
                "msg_content": message.body,
                "title": message.title,
                "extras": message.data,
            },
        });
        
        let response = client
            .post(url)
            .header("Authorization", format!("Basic {}", auth))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to send Jpush: {}", e)))?;
        
        let status = response.status();
        
        if status.is_success() {
            let body = response.json::<serde_json::Value>().await
                .map_err(|e| AppError::InternalServerError(format!("Failed to parse response: {}", e)))?;
            
            Ok(PushResult {
                success: true,
                message_id: body["msg_id"].as_str().map(|s| s.to_string()),
                failed_tokens: vec![],
                error_message: None,
            })
        } else {
            Ok(PushResult {
                success: false,
                message_id: None,
                failed_tokens: message.device_tokens.clone(),
                error_message: Some("Jpush error".to_string()),
            })
        }
    }
    
    /// Send Getui push notification
    async fn send_getui_push(
        config: &PushConfig,
        message: PushMessage,
    ) -> Result<PushResult, AppError> {
        // Simplified implementation - production should use proper Getui API
        
        Ok(PushResult {
            success: false,
            message_id: None,
            failed_tokens: message.device_tokens.clone(),
            error_message: Some("Getui not implemented".to_string()),
        })
    }
    
    /// Register device token
    pub async fn register_device_token(
        db: &DbPool,
        user_id: Uuid,
        token: &str,
        platform: &str,
        provider: &str,
    ) -> Result<DeviceToken, AppError> {
        // First, deactivate any existing tokens for this device
        let update_query = r#"
            UPDATE device_tokens
            SET is_active = false
            WHERE token = ? AND user_id != ?
        "#;
        
        sqlx::query(update_query)
            .bind(token)
            .bind(user_id.to_string())
            .execute(db)
            .await?;
        
        // Check if token already exists for this user
        let check_query = r#"
            SELECT id FROM device_tokens
            WHERE user_id = ? AND token = ?
        "#;
        
        let existing = sqlx::query(check_query)
            .bind(user_id.to_string())
            .bind(token)
            .fetch_optional(db)
            .await?;
        
        if let Some(row) = existing {
            // Update existing token
            let id: String = sqlx::Row::get(&row, "id");
            let update_query = r#"
                UPDATE device_tokens
                SET platform = ?, provider = ?, is_active = true, updated_at = ?
                WHERE id = ?
            "#;
            
            sqlx::query(update_query)
                .bind(platform)
                .bind(provider)
                .bind(Utc::now())
                .bind(&id)
                .execute(db)
                .await?;
            
            Self::get_device_token_by_id(db, Uuid::parse_str(&id).unwrap()).await
        } else {
            // Create new token
            let id = Uuid::new_v4();
            let now = Utc::now();
            
            let insert_query = r#"
                INSERT INTO device_tokens (
                    id, user_id, token, platform, provider, 
                    is_active, created_at, updated_at
                )
                VALUES (?, ?, ?, ?, ?, true, ?, ?)
            "#;
            
            sqlx::query(insert_query)
                .bind(id.to_string())
                .bind(user_id.to_string())
                .bind(token)
                .bind(platform)
                .bind(provider)
                .bind(&now)
                .bind(&now)
                .execute(db)
                .await?;
            
            Self::get_device_token_by_id(db, id).await
        }
    }
    
    /// Get active device tokens for user
    pub async fn get_user_device_tokens(
        db: &DbPool,
        user_id: Uuid,
    ) -> Result<Vec<DeviceToken>, AppError> {
        let query = r#"
            SELECT id, user_id, token, platform, provider, is_active, created_at, updated_at
            FROM device_tokens
            WHERE user_id = ? AND is_active = true
        "#;
        
        let rows = sqlx::query(query)
            .bind(user_id.to_string())
            .fetch_all(db)
            .await?;
        
        let mut tokens = Vec::new();
        for row in rows {
            tokens.push(DeviceToken {
                id: Uuid::parse_str(sqlx::Row::get(&row, "id")).unwrap(),
                user_id: Uuid::parse_str(sqlx::Row::get(&row, "user_id")).unwrap(),
                token: sqlx::Row::get(&row, "token"),
                platform: sqlx::Row::get(&row, "platform"),
                provider: sqlx::Row::get(&row, "provider"),
                is_active: sqlx::Row::get(&row, "is_active"),
                created_at: sqlx::Row::get(&row, "created_at"),
                updated_at: sqlx::Row::get(&row, "updated_at"),
            });
        }
        
        Ok(tokens)
    }
    
    /// Get device token by ID
    async fn get_device_token_by_id(
        db: &DbPool,
        id: Uuid,
    ) -> Result<DeviceToken, AppError> {
        let query = r#"
            SELECT id, user_id, token, platform, provider, is_active, created_at, updated_at
            FROM device_tokens
            WHERE id = ?
        "#;
        
        let row = sqlx::query(query)
            .bind(id.to_string())
            .fetch_one(db)
            .await
            .map_err(|e| AppError::NotFound(format!("Device token not found: {}", e)))?;
        
        Ok(DeviceToken {
            id: Uuid::parse_str(sqlx::Row::get(&row, "id")).unwrap(),
            user_id: Uuid::parse_str(sqlx::Row::get(&row, "user_id")).unwrap(),
            token: sqlx::Row::get(&row, "token"),
            platform: sqlx::Row::get(&row, "platform"),
            provider: sqlx::Row::get(&row, "provider"),
            is_active: sqlx::Row::get(&row, "is_active"),
            created_at: sqlx::Row::get(&row, "created_at"),
            updated_at: sqlx::Row::get(&row, "updated_at"),
        })
    }
    
    /// Store push notification record
    pub async fn store_push_record(
        db: &DbPool,
        user_id: Uuid,
        title: &str,
        body: &str,
        result: &PushResult,
    ) -> Result<(), AppError> {
        let query = r#"
            INSERT INTO push_records (
                id, user_id, title, body, 
                status, message_id, error_message, sent_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#;
        
        sqlx::query(query)
            .bind(Uuid::new_v4().to_string())
            .bind(user_id.to_string())
            .bind(title)
            .bind(body)
            .bind(if result.success { "success" } else { "failed" })
            .bind(&result.message_id)
            .bind(&result.error_message)
            .bind(Utc::now())
            .execute(db)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to store push record: {}", e)))?;
        
        Ok(())
    }
}