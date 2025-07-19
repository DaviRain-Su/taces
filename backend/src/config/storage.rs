use aws_config::{BehaviorVersion, Region};
use aws_sdk_s3::{Client as S3Client, config::Credentials};
use std::env;

pub struct StorageConfig {
    pub storage_type: StorageType,
    pub endpoint: Option<String>,
    pub region: String,
    pub access_key_id: String,
    pub secret_access_key: String,
    pub bucket_name: String,
}

#[derive(Debug, Clone)]
pub enum StorageType {
    S3,
    OSS, // Aliyun OSS
}

impl StorageConfig {
    pub fn from_env() -> Self {
        let storage_type = match env::var("STORAGE_TYPE").unwrap_or_else(|_| "S3".to_string()).as_str() {
            "OSS" => StorageType::OSS,
            _ => StorageType::S3,
        };
        
        Self {
            storage_type,
            endpoint: env::var("STORAGE_ENDPOINT").ok(),
            region: env::var("STORAGE_REGION").unwrap_or_else(|_| "us-east-1".to_string()),
            access_key_id: env::var("STORAGE_ACCESS_KEY_ID").unwrap_or_else(|_| "".to_string()),
            secret_access_key: env::var("STORAGE_SECRET_ACCESS_KEY").unwrap_or_else(|_| "".to_string()),
            bucket_name: env::var("STORAGE_BUCKET_NAME").unwrap_or_else(|_| "tcm-telemedicine".to_string()),
        }
    }
}

pub async fn create_s3_client() -> Result<S3Client, Box<dyn std::error::Error>> {
    let config = StorageConfig::from_env();
    
    let credentials = Credentials::new(
        &config.access_key_id,
        &config.secret_access_key,
        None,
        None,
        "tcm-storage"
    );
    
    let mut s3_config_builder = aws_sdk_s3::Config::builder()
        .behavior_version(BehaviorVersion::latest())
        .region(Region::new(config.region))
        .credentials_provider(credentials);
    
    // For OSS or custom S3-compatible storage
    if let Some(endpoint) = config.endpoint {
        s3_config_builder = s3_config_builder
            .endpoint_url(endpoint)
            .force_path_style(true);
    }
    
    let s3_config = s3_config_builder.build();
    Ok(S3Client::from_conf(s3_config))
}

pub async fn create_s3_client_optional() -> Option<S3Client> {
    // Only create S3 client if credentials are provided
    if env::var("STORAGE_ACCESS_KEY_ID").is_ok() && env::var("STORAGE_SECRET_ACCESS_KEY").is_ok() {
        match create_s3_client().await {
            Ok(client) => {
                tracing::info!("S3 client created successfully");
                Some(client)
            }
            Err(e) => {
                tracing::warn!("Failed to create S3 client: {}", e);
                None
            }
        }
    } else {
        tracing::info!("S3 credentials not provided, file storage will use local filesystem");
        None
    }
}