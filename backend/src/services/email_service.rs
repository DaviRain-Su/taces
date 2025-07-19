use crate::{
    config::database::DbPool,
    models::notification::*,
    utils::errors::AppError,
};
use chrono::{DateTime, Utc};
use handlebars::Handlebars;
use lettre::{
    message::{header::ContentType, Mailbox, Message},
    transport::smtp::authentication::Credentials,
    AsyncSmtpTransport, AsyncTransport, Tokio1Executor,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct EmailConfig {
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_username: String,
    pub smtp_password: String,
    pub from_email: String,
    pub from_name: String,
    pub use_tls: bool,
}

impl EmailConfig {
    pub fn from_env() -> Option<Self> {
        Some(Self {
            smtp_host: std::env::var("SMTP_HOST").ok()?,
            smtp_port: std::env::var("SMTP_PORT")
                .ok()?
                .parse()
                .unwrap_or(587),
            smtp_username: std::env::var("SMTP_USERNAME").ok()?,
            smtp_password: std::env::var("SMTP_PASSWORD").ok()?,
            from_email: std::env::var("SMTP_FROM_EMAIL")
                .unwrap_or_else(|_| "noreply@tcm-clinic.com".to_string()),
            from_name: std::env::var("SMTP_FROM_NAME")
                .unwrap_or_else(|_| "香河香草中医诊所".to_string()),
            use_tls: std::env::var("SMTP_USE_TLS")
                .unwrap_or_else(|_| "true".to_string())
                .parse()
                .unwrap_or(true),
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailMessage {
    pub to_email: String,
    pub to_name: Option<String>,
    pub subject: String,
    pub template_name: String,
    pub template_data: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EmailSendResult {
    pub success: bool,
    pub message_id: Option<String>,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone)]
pub struct EmailTemplate {
    pub name: String,
    pub subject: String,
    pub html_template: String,
    pub text_template: String,
}

pub struct EmailService;

impl EmailService {
    /// Send email message
    pub async fn send_email(
        config: &EmailConfig,
        message: EmailMessage,
    ) -> Result<EmailSendResult, AppError> {
        // Create SMTP transport
        let smtp_transport = if config.use_tls {
            AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&config.smtp_host)
        } else {
            AsyncSmtpTransport::<Tokio1Executor>::relay(&config.smtp_host)
        }
        .map_err(|e| AppError::InternalServerError(format!("Failed to create SMTP transport: {}", e)))?
        .port(config.smtp_port)
        .credentials(Credentials::new(
            config.smtp_username.clone(),
            config.smtp_password.clone(),
        ))
        .build();
        
        // Get email template
        let template = Self::get_email_template(&message.template_name)?;
        
        // Render email content
        let html_content = Self::render_template(&template.html_template, &message.template_data)?;
        let text_content = Self::render_template(&template.text_template, &message.template_data)?;
        
        // Build email
        let from_mailbox: Mailbox = format!("{} <{}>", config.from_name, config.from_email)
            .parse()
            .map_err(|e| AppError::InternalServerError(format!("Invalid from email: {}", e)))?;
        
        let to_mailbox: Mailbox = if let Some(name) = &message.to_name {
            format!("{} <{}>", name, message.to_email)
        } else {
            message.to_email.clone()
        }
        .parse()
        .map_err(|e| AppError::InternalServerError(format!("Invalid to email: {}", e)))?;
        
        let email = Message::builder()
            .from(from_mailbox)
            .to(to_mailbox)
            .subject(&message.subject)
            .multipart(
                lettre::message::MultiPart::alternative()
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(ContentType::TEXT_PLAIN)
                            .body(text_content),
                    )
                    .singlepart(
                        lettre::message::SinglePart::builder()
                            .header(ContentType::TEXT_HTML)
                            .body(html_content),
                    ),
            )
            .map_err(|e| AppError::InternalServerError(format!("Failed to build email: {}", e)))?;
        
        // Send email
        match smtp_transport.send(email).await {
            Ok(response) => Ok(EmailSendResult {
                success: true,
                message_id: Some(response.message().to_string()),
                error_message: None,
            }),
            Err(e) => Ok(EmailSendResult {
                success: false,
                message_id: None,
                error_message: Some(format!("Failed to send email: {}", e)),
            }),
        }
    }
    
    /// Send appointment reminder email
    pub async fn send_appointment_reminder(
        config: &EmailConfig,
        to_email: &str,
        patient_name: &str,
        doctor_name: &str,
        appointment_time: &str,
        clinic_address: &str,
    ) -> Result<EmailSendResult, AppError> {
        let mut template_data = HashMap::new();
        template_data.insert("patient_name".to_string(), patient_name.to_string());
        template_data.insert("doctor_name".to_string(), doctor_name.to_string());
        template_data.insert("appointment_time".to_string(), appointment_time.to_string());
        template_data.insert("clinic_address".to_string(), clinic_address.to_string());
        
        let message = EmailMessage {
            to_email: to_email.to_string(),
            to_name: Some(patient_name.to_string()),
            subject: format!("预约提醒 - {}医生", doctor_name),
            template_name: "appointment_reminder".to_string(),
            template_data,
        };
        
        Self::send_email(config, message).await
    }
    
    /// Send prescription ready email
    pub async fn send_prescription_ready(
        config: &EmailConfig,
        to_email: &str,
        patient_name: &str,
        prescription_code: &str,
        doctor_name: &str,
    ) -> Result<EmailSendResult, AppError> {
        let mut template_data = HashMap::new();
        template_data.insert("patient_name".to_string(), patient_name.to_string());
        template_data.insert("prescription_code".to_string(), prescription_code.to_string());
        template_data.insert("doctor_name".to_string(), doctor_name.to_string());
        
        let message = EmailMessage {
            to_email: to_email.to_string(),
            to_name: Some(patient_name.to_string()),
            subject: "您的处方已开具完成".to_string(),
            template_name: "prescription_ready".to_string(),
            template_data,
        };
        
        Self::send_email(config, message).await
    }
    
    /// Send welcome email
    pub async fn send_welcome_email(
        config: &EmailConfig,
        to_email: &str,
        user_name: &str,
    ) -> Result<EmailSendResult, AppError> {
        let mut template_data = HashMap::new();
        template_data.insert("user_name".to_string(), user_name.to_string());
        
        let message = EmailMessage {
            to_email: to_email.to_string(),
            to_name: Some(user_name.to_string()),
            subject: "欢迎加入香河香草中医诊所".to_string(),
            template_name: "welcome".to_string(),
            template_data,
        };
        
        Self::send_email(config, message).await
    }
    
    /// Send password reset email
    pub async fn send_password_reset(
        config: &EmailConfig,
        to_email: &str,
        user_name: &str,
        reset_link: &str,
    ) -> Result<EmailSendResult, AppError> {
        let mut template_data = HashMap::new();
        template_data.insert("user_name".to_string(), user_name.to_string());
        template_data.insert("reset_link".to_string(), reset_link.to_string());
        
        let message = EmailMessage {
            to_email: to_email.to_string(),
            to_name: Some(user_name.to_string()),
            subject: "密码重置请求".to_string(),
            template_name: "password_reset".to_string(),
            template_data,
        };
        
        Self::send_email(config, message).await
    }
    
    /// Get email template
    fn get_email_template(template_name: &str) -> Result<EmailTemplate, AppError> {
        match template_name {
            "appointment_reminder" => Ok(EmailTemplate {
                name: template_name.to_string(),
                subject: "预约提醒".to_string(),
                html_template: r#"
                    <html>
                    <body>
                        <h2>预约提醒</h2>
                        <p>尊敬的{{patient_name}}：</p>
                        <p>您预约的{{doctor_name}}医生的就诊时间是<strong>{{appointment_time}}</strong>。</p>
                        <p>就诊地址：{{clinic_address}}</p>
                        <p>请准时到诊，如需改期请提前联系我们。</p>
                        <p>祝您身体健康！</p>
                        <p>香河香草中医诊所</p>
                    </body>
                    </html>
                "#.to_string(),
                text_template: r#"
预约提醒

尊敬的{{patient_name}}：

您预约的{{doctor_name}}医生的就诊时间是{{appointment_time}}。

就诊地址：{{clinic_address}}

请准时到诊，如需改期请提前联系我们。

祝您身体健康！

香河香草中医诊所
                "#.to_string(),
            }),
            
            "prescription_ready" => Ok(EmailTemplate {
                name: template_name.to_string(),
                subject: "处方已开具".to_string(),
                html_template: r#"
                    <html>
                    <body>
                        <h2>处方已开具完成</h2>
                        <p>尊敬的{{patient_name}}：</p>
                        <p>您的处方（编号：<strong>{{prescription_code}}</strong>）已由{{doctor_name}}医生开具完成。</p>
                        <p>请登录系统查看处方详情，或到药房取药。</p>
                        <p>如有任何疑问，请联系我们。</p>
                        <p>祝您早日康复！</p>
                        <p>香河香草中医诊所</p>
                    </body>
                    </html>
                "#.to_string(),
                text_template: r#"
处方已开具完成

尊敬的{{patient_name}}：

您的处方（编号：{{prescription_code}}）已由{{doctor_name}}医生开具完成。

请登录系统查看处方详情，或到药房取药。

如有任何疑问，请联系我们。

祝您早日康复！

香河香草中医诊所
                "#.to_string(),
            }),
            
            "welcome" => Ok(EmailTemplate {
                name: template_name.to_string(),
                subject: "欢迎加入".to_string(),
                html_template: r#"
                    <html>
                    <body>
                        <h2>欢迎加入香河香草中医诊所</h2>
                        <p>尊敬的{{user_name}}：</p>
                        <p>感谢您注册香河香草中医诊所在线诊疗平台。</p>
                        <p>您现在可以：</p>
                        <ul>
                            <li>在线预约挂号</li>
                            <li>查看处方记录</li>
                            <li>参与健康直播</li>
                            <li>获取健康资讯</li>
                        </ul>
                        <p>如有任何问题，请随时联系我们。</p>
                        <p>香河香草中医诊所</p>
                    </body>
                    </html>
                "#.to_string(),
                text_template: r#"
欢迎加入香河香草中医诊所

尊敬的{{user_name}}：

感谢您注册香河香草中医诊所在线诊疗平台。

您现在可以：
- 在线预约挂号
- 查看处方记录
- 参与健康直播
- 获取健康资讯

如有任何问题，请随时联系我们。

香河香草中医诊所
                "#.to_string(),
            }),
            
            "password_reset" => Ok(EmailTemplate {
                name: template_name.to_string(),
                subject: "密码重置".to_string(),
                html_template: r#"
                    <html>
                    <body>
                        <h2>密码重置请求</h2>
                        <p>尊敬的{{user_name}}：</p>
                        <p>我们收到了您的密码重置请求。</p>
                        <p>请点击以下链接重置您的密码：</p>
                        <p><a href="{{reset_link}}">{{reset_link}}</a></p>
                        <p>此链接将在24小时后失效。</p>
                        <p>如果您没有请求重置密码，请忽略此邮件。</p>
                        <p>香河香草中医诊所</p>
                    </body>
                    </html>
                "#.to_string(),
                text_template: r#"
密码重置请求

尊敬的{{user_name}}：

我们收到了您的密码重置请求。

请访问以下链接重置您的密码：
{{reset_link}}

此链接将在24小时后失效。

如果您没有请求重置密码，请忽略此邮件。

香河香草中医诊所
                "#.to_string(),
            }),
            
            _ => Err(AppError::NotFound(format!("Email template '{}' not found", template_name))),
        }
    }
    
    /// Render template with data
    fn render_template(
        template: &str,
        data: &HashMap<String, String>,
    ) -> Result<String, AppError> {
        let mut handlebars = Handlebars::new();
        handlebars.register_template_string("template", template)
            .map_err(|e| AppError::InternalServerError(format!("Failed to register template: {}", e)))?;
        
        handlebars.render("template", data)
            .map_err(|e| AppError::InternalServerError(format!("Failed to render template: {}", e)))
    }
    
    /// Store email record in database
    pub async fn store_email_record(
        db: &DbPool,
        to_email: &str,
        subject: &str,
        template_name: &str,
        result: &EmailSendResult,
    ) -> Result<(), AppError> {
        let query = r#"
            INSERT INTO email_records (
                id, to_email, subject, template_name, 
                status, message_id, error_message, sent_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
        "#;
        
        sqlx::query(query)
            .bind(Uuid::new_v4().to_string())
            .bind(to_email)
            .bind(subject)
            .bind(template_name)
            .bind(if result.success { "success" } else { "failed" })
            .bind(&result.message_id)
            .bind(&result.error_message)
            .bind(Utc::now())
            .execute(db)
            .await
            .map_err(|e| AppError::InternalServerError(format!("Failed to store email record: {}", e)))?;
        
        Ok(())
    }
}