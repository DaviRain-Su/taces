use crate::controllers::review_controller::*;
use crate::middleware::auth::auth_middleware;
use crate::AppState;
use axum::{
    middleware,
    routing::{get, post, put},
    Router,
};

pub fn routes() -> Router<AppState> {
    let public_routes = Router::new()
        // 公开路由 - 任何人都可以查看评价和标签
        .route("/doctor/:doctor_id/reviews", get(get_doctor_reviews))
        .route("/doctor/:doctor_id/statistics", get(get_doctor_statistics))
        .route("/tags", get(get_tags));

    let protected_routes = Router::new()
        // 需要认证的路由
        .route("/", post(create_review).get(get_reviews))
        .route("/:id", get(get_review_by_id).put(update_review))
        .route("/:id/reply", post(reply_to_review))
        .route("/:id/visibility", put(update_review_visibility))
        .route("/patient/:patient_id/reviews", get(get_patient_reviews))
        .route("/tags", post(create_tag))
        .layer(middleware::from_fn(auth_middleware));

    Router::new().merge(public_routes).merge(protected_routes)
}
