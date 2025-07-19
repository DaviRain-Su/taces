use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, NaiveDate};
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize)]
pub struct PatientProfile {
    pub id: Uuid,
    pub user_id: Uuid,
    pub name: String,
    pub id_number: String,
    pub phone: String,
    pub gender: Gender,
    pub birthday: Option<NaiveDate>,
    pub relationship: Relationship,
    pub is_default: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Gender {
    #[serde(rename = "男")]
    Male,
    #[serde(rename = "女")]
    Female,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "lowercase")]
pub enum Relationship {
    #[serde(rename = "self")]
    MySelf,
    Family,
    Friend,
    Other,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreatePatientProfileDto {
    #[validate(length(min = 2, max = 50))]
    pub name: String,
    #[validate(length(min = 15, max = 18))]
    pub id_number: String,
    #[validate(length(min = 11, max = 11))]
    pub phone: String,
    pub gender: Gender,
    pub birthday: Option<NaiveDate>,
    pub relationship: Relationship,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct UpdatePatientProfileDto {
    #[validate(length(min = 2, max = 50))]
    pub name: Option<String>,
    #[validate(length(min = 11, max = 11))]
    pub phone: Option<String>,
    pub gender: Option<Gender>,
    pub birthday: Option<NaiveDate>,
    pub relationship: Option<Relationship>,
}

// Helper function to validate Chinese ID card number
pub fn validate_id_number(id_number: &str) -> bool {
    // Basic validation - should be 15 or 18 characters
    if id_number.len() != 15 && id_number.len() != 18 {
        return false;
    }
    
    // Extract date parts
    let (_year_str, month_str, day_str) = if id_number.len() == 18 {
        (&id_number[6..10], &id_number[10..12], &id_number[12..14])
    } else {
        // 15-digit ID uses 2-digit year
        (&id_number[6..8], &id_number[8..10], &id_number[10..12])
    };
    
    // Validate date parts
    let month = match month_str.parse::<u32>() {
        Ok(m) if m >= 1 && m <= 12 => m,
        _ => return false,
    };
    
    let day = match day_str.parse::<u32>() {
        Ok(d) if d >= 1 && d <= 31 => d,
        _ => return false,
    };
    
    // Simple date validation (doesn't check for month-specific days)
    if month == 2 && day > 29 {
        return false;
    }
    if (month == 4 || month == 6 || month == 9 || month == 11) && day > 30 {
        return false;
    }
    
    // For 18-digit ID, validate checksum
    if id_number.len() == 18 {
        let chars: Vec<char> = id_number.chars().collect();
        let weights = [7, 9, 10, 5, 8, 4, 2, 1, 6, 3, 7, 9, 10, 5, 8, 4, 2];
        let check_codes = ['1', '0', 'X', '9', '8', '7', '6', '5', '4', '3', '2'];
        
        let mut sum = 0;
        for i in 0..17 {
            if let Some(digit) = chars[i].to_digit(10) {
                sum += digit as usize * weights[i];
            } else {
                return false;
            }
        }
        
        let expected_check = check_codes[sum % 11];
        chars[17] == expected_check || (chars[17] == 'x' && expected_check == 'X')
    } else {
        // For 15-digit ID, just check if all are digits
        id_number.chars().all(|c| c.is_numeric())
    }
}