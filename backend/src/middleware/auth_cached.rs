use crate::{
    services::session_service::SessionService,
    utils::jwt::decode_token,
    AppState,
};
use axum::{
    extract::{Request, State},
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

pub async fn auth_middleware_cached(
    State(app_state): State<AppState>,
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

    // Check session in Redis first
    if let Some(session) = SessionService::get_session(&app_state.redis, token).await {
        let auth_user = AuthUser {
            user_id: session.user_id,
            role: session.role,
        };
        req.extensions_mut().insert(auth_user);
        
        // Extend session TTL
        let _ = SessionService::extend_session(&app_state.redis, token).await;
        
        return Ok(next.run(req).await);
    }

    // Fall back to JWT validation if no session
    match decode_token(token, &app_state.config.jwt_secret) {
        Ok(claims) => {
            let auth_user = AuthUser {
                user_id: claims.sub,
                role: claims.role,
            };
            
            // Try to create session for valid JWT
            if SessionService::is_session_valid(&app_state.redis, token).await {
                req.extensions_mut().insert(auth_user);
                Ok(next.run(req).await)
            } else {
                // Valid JWT but no session - might be a new token
                req.extensions_mut().insert(auth_user);
                Ok(next.run(req).await)
            }
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

pub fn require_role_cached(allowed_roles: Vec<String>) -> impl Fn(Request, Next) -> BoxedFuture + Clone {
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