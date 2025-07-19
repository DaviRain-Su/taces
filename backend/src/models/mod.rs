use serde::{Deserialize, Serialize};

pub mod appointment;
pub mod circle_post;
pub mod content;
pub mod department;
pub mod doctor;
pub mod live_stream;
pub mod patient_group;
pub mod patient_profile;
pub mod prescription;
pub mod user;

pub use appointment::*;
pub use circle_post::*;
pub use content::*;
pub use department::*;
pub use doctor::*;
pub use live_stream::*;
pub use patient_group::*;
pub use patient_profile::*;
pub use prescription::*;
pub use user::*;

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
