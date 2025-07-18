use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use validator::Validate;

pub mod user;
pub mod doctor;
pub mod appointment;
pub mod prescription;
pub mod live_stream;
pub mod circle_post;

pub use user::*;
pub use doctor::*;
pub use appointment::*;
pub use prescription::*;
pub use live_stream::*;
pub use circle_post::*;

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse<T> {
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T> {
    pub fn success(message: &str, data: T) -> Self {
        Self {
            success: true,
            message: message.to_string(),
            data: Some(data),
        }
    }

    pub fn error(message: &str) -> Self {
        Self {
            success: false,
            message: message.to_string(),
            data: None,
        }
    }
}