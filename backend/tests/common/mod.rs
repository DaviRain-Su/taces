use axum::body::to_bytes;
use axum::http::{Request, StatusCode};
use axum::{body::Body, Router};
use backend::{
    config::{database::DbPool, Config},
    routes,
    utils::test_helpers::{create_test_pool, setup_test_db},
    AppState,
};
use serde_json::Value;
use tower::Service;

pub struct TestApp {
    pub app: Router,
    pub pool: DbPool,
    #[allow(dead_code)]
    pub config: Config,
}

impl TestApp {
    pub async fn new() -> Self {
        dotenv::dotenv().ok();

        let pool = create_test_pool().await;
        setup_test_db(&pool).await;

        let config = Config {
            database_url: std::env::var("TEST_DATABASE_URL").unwrap_or_else(|_| {
                "mysql://tcm_user:tcm_pass123@localhost:3307/tcm_telemedicine_test".to_string()
            }),
            jwt_secret: "test_jwt_secret".to_string(),
            jwt_expiration: 3600,
            server_port: 3001,
        };

        // Set JWT_SECRET environment variable for auth middleware
        std::env::set_var("JWT_SECRET", &config.jwt_secret);

        let state = AppState {
            config: config.clone(),
            pool: pool.clone(),
            redis: None,
            ws_manager: std::sync::Arc::new(
                backend::services::websocket_service::WebSocketManager::new(),
            ),
            s3_client: None,
        };

        let app = Router::new()
            .nest("/api/v1", routes::create_routes())
            .with_state(state);

        Self { app, pool, config }
    }

    pub async fn post<T>(&mut self, path: &str, body: T) -> (StatusCode, Value)
    where
        T: serde::Serialize,
    {
        let request = Request::builder()
            .method("POST")
            .uri(path)
            .header("content-type", "application/json")
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        let response = self.app.call(request).await.unwrap();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        (status, json)
    }

    pub async fn post_with_auth<T>(
        &mut self,
        path: &str,
        body: T,
        token: &str,
    ) -> (StatusCode, Value)
    where
        T: serde::Serialize,
    {
        let request = Request::builder()
            .method("POST")
            .uri(path)
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        let response = self.app.call(request).await.unwrap();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        (status, json)
    }

    pub async fn get(&mut self, path: &str) -> (StatusCode, Value) {
        let request = Request::builder()
            .method("GET")
            .uri(path)
            .body(Body::empty())
            .unwrap();

        let response = self.app.call(request).await.unwrap();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        (status, json)
    }

    pub async fn get_with_auth(&mut self, path: &str, token: &str) -> (StatusCode, Value) {
        let request = Request::builder()
            .method("GET")
            .uri(path)
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = self.app.call(request).await.unwrap();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        (status, json)
    }

    pub async fn put_with_auth<T>(
        &mut self,
        path: &str,
        body: T,
        token: &str,
    ) -> (StatusCode, Value)
    where
        T: serde::Serialize,
    {
        let request = Request::builder()
            .method("PUT")
            .uri(path)
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        let response = self.app.call(request).await.unwrap();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        (status, json)
    }

    pub async fn delete_with_auth(&mut self, path: &str, token: &str) -> (StatusCode, Value) {
        let request = Request::builder()
            .method("DELETE")
            .uri(path)
            .header("authorization", format!("Bearer {}", token))
            .body(Body::empty())
            .unwrap();

        let response = self.app.call(request).await.unwrap();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        (status, json)
    }

    pub async fn delete_with_auth_body<T>(
        &mut self,
        path: &str,
        body: T,
        token: &str,
    ) -> (StatusCode, Value)
    where
        T: serde::Serialize,
    {
        let request = Request::builder()
            .method("DELETE")
            .uri(path)
            .header("content-type", "application/json")
            .header("authorization", format!("Bearer {}", token))
            .body(Body::from(serde_json::to_string(&body).unwrap()))
            .unwrap();

        let response = self.app.call(request).await.unwrap();
        let status = response.status();
        let body = to_bytes(response.into_body(), usize::MAX).await.unwrap();
        let json: Value = serde_json::from_slice(&body).unwrap();

        (status, json)
    }
}
