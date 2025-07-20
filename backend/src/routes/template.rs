use crate::controllers::template_controller::*;
use crate::middleware::auth::auth_middleware;
use crate::AppState;
use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        // 常用语路由
        .route("/common-phrases", post(create_common_phrase))
        .route("/common-phrases", get(get_common_phrases))
        .route("/common-phrases/:id", get(get_common_phrase_by_id))
        .route("/common-phrases/:id", put(update_common_phrase))
        .route("/common-phrases/:id", delete(delete_common_phrase))
        .route("/common-phrases/:id/use", post(use_common_phrase))
        // 处方模板路由
        .route(
            "/prescription-templates",
            post(create_prescription_template),
        )
        .route("/prescription-templates", get(get_prescription_templates))
        .route(
            "/prescription-templates/:id",
            get(get_prescription_template_by_id),
        )
        .route(
            "/prescription-templates/:id",
            put(update_prescription_template),
        )
        .route(
            "/prescription-templates/:id",
            delete(delete_prescription_template),
        )
        .route(
            "/prescription-templates/:id/use",
            post(use_prescription_template),
        )
        // 所有路由都需要认证
        .layer(middleware::from_fn(auth_middleware))
}
