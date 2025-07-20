use crate::{controllers::payment_controller::*, middleware::auth::auth_middleware, AppState};
use axum::{
    middleware,
    routing::{get, post, put},
    Router,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        // Order management routes
        .route("/orders", post(create_order))
        .route("/orders", get(list_orders))
        .route("/orders/:id", get(get_order))
        .route("/orders/:id/cancel", put(cancel_order))
        // Payment routes
        .route("/pay", post(initiate_payment))
        // Refund routes
        .route("/refunds", post(create_refund))
        .route("/refunds/:id", get(get_refund))
        // Balance routes
        .route("/balance/:user_id", get(get_user_balance))
        .route(
            "/balance/:user_id/transactions",
            get(get_balance_transactions),
        )
        // Statistics routes
        .route("/statistics", get(get_payment_statistics))
        // Admin only routes
        .route("/admin/refunds/:id/review", put(review_refund))
        .route("/admin/config/:payment_method", put(update_payment_config))
        // Apply auth middleware to most routes
        .layer(middleware::from_fn(auth_middleware))
}

pub fn public_routes() -> Router<AppState> {
    Router::new()
        // Payment callback route (no auth required)
        .route("/payment/callback", post(payment_callback))
        // Price configuration routes (public)
        .route("/prices/:service_type", get(get_price_config))
        .route("/prices", get(list_price_configs))
}
