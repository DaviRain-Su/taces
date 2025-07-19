use crate::utils::jwt::decode_token;
use axum::{
    extract::Request,
    http::{header, StatusCode},
    middleware::Next,
    response::Response,
    Json,
};
use serde_json::json;

#[derive(Clone)]
pub struct AuthUser {
    pub user_id: uuid::Uuid,
    pub role: String,
}

pub async fn auth_middleware(
    mut req: Request,
    next: Next,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let auth_header = req
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok());

    let token = match auth_header {
        Some(auth_value) if auth_value.starts_with("Bearer ") => {
            auth_value.trim_start_matches("Bearer ")
        }
        _ => {
            return Err((
                StatusCode::UNAUTHORIZED,
                Json(json!({
                    "success": false,
                    "message": "Missing or invalid authorization header"
                })),
            ));
        }
    };

    // Get JWT secret from environment
    let jwt_secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "default_jwt_secret".to_string());

    match decode_token(token, &jwt_secret) {
        Ok(claims) => {
            let auth_user = AuthUser {
                user_id: claims.sub,
                role: claims.role,
            };
            req.extensions_mut().insert(auth_user);
            Ok(next.run(req).await)
        }
        Err(_) => Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({
                "success": false,
                "message": "Invalid or expired token"
            })),
        )),
    }
}

type BoxedFuture = std::pin::Pin<
    Box<
        dyn std::future::Future<Output = Result<Response, (StatusCode, Json<serde_json::Value>)>>
            + Send,
    >,
>;

pub fn require_role(allowed_roles: Vec<String>) -> impl Fn(Request, Next) -> BoxedFuture + Clone {
    move |req: Request, next: Next| {
        let allowed_roles = allowed_roles.clone();
        Box::pin(async move {
            let auth_user = req.extensions().get::<AuthUser>().ok_or_else(|| {
                (
                    StatusCode::UNAUTHORIZED,
                    Json(json!({
                        "success": false,
                        "message": "Unauthorized"
                    })),
                )
            })?;

            if allowed_roles.contains(&auth_user.role) {
                Ok(next.run(req).await)
            } else {
                Err((
                    StatusCode::FORBIDDEN,
                    Json(json!({
                        "success": false,
                        "message": "Insufficient permissions"
                    })),
                ))
            }
        })
    }
}
