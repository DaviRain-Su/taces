use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub role: String,
    pub exp: i64,
    pub iat: i64,
}

impl Claims {
    pub fn new(user_id: Uuid, role: String, expiration_seconds: i64) -> Self {
        let now = Utc::now();
        let exp = (now + Duration::seconds(expiration_seconds)).timestamp();

        Self {
            sub: user_id,
            role,
            exp,
            iat: now.timestamp(),
        }
    }
}

pub fn create_token(
    user_id: Uuid,
    role: String,
    secret: &str,
    expiration: i64,
) -> Result<String, jsonwebtoken::errors::Error> {
    let claims = Claims::new(user_id, role, expiration);
    let encoding_key = EncodingKey::from_secret(secret.as_ref());

    encode(&Header::default(), &claims, &encoding_key)
}

pub fn decode_token(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let decoding_key = DecodingKey::from_secret(secret.as_ref());
    let validation = Validation::default();

    let token_data = decode::<Claims>(token, &decoding_key, &validation)?;
    Ok(token_data.claims)
}
