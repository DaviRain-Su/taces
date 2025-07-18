use axum::{
    extract::Request,
    response::Response,
    middleware::Next,
};
use std::sync::Arc;

#[derive(Clone)]
pub struct JwtConfig {
    pub secret: String,
}

pub async fn inject_jwt_config(
    mut req: Request,
    next: Next,
) -> Response {
    // Get JWT secret from environment or use default
    let jwt_secret = std::env::var("JWT_SECRET")
        .unwrap_or_else(|_| "default_jwt_secret".to_string());
    
    req.extensions_mut().insert(Arc::new(JwtConfig {
        secret: jwt_secret,
    }));
    
    next.run(req).await
}