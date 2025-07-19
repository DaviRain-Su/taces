use crate::{controllers::content_controller, middleware::auth::auth_middleware, AppState};
use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

pub fn routes() -> Router<AppState> {
    Router::new()
        // Article routes
        .route("/articles", get(content_controller::list_articles))
        .route("/articles/:id", get(content_controller::get_article))
        .route(
            "/articles",
            post(content_controller::create_article)
                .layer(middleware::from_fn(auth_middleware)),
        )
        .route(
            "/articles/:id",
            put(content_controller::update_article)
                .layer(middleware::from_fn(auth_middleware)),
        )
        .route(
            "/articles/:id/publish",
            post(content_controller::publish_article)
                .layer(middleware::from_fn(auth_middleware)),
        )
        .route(
            "/articles/:id/unpublish",
            post(content_controller::unpublish_article)
                .layer(middleware::from_fn(auth_middleware)),
        )
        .route(
            "/articles/:id",
            delete(content_controller::delete_article)
                .layer(middleware::from_fn(auth_middleware)),
        )
        // Video routes
        .route("/videos", get(content_controller::list_videos))
        .route("/videos/:id", get(content_controller::get_video))
        .route(
            "/videos",
            post(content_controller::create_video)
                .layer(middleware::from_fn(auth_middleware)),
        )
        .route(
            "/videos/:id",
            put(content_controller::update_video)
                .layer(middleware::from_fn(auth_middleware)),
        )
        .route(
            "/videos/:id/publish",
            post(content_controller::publish_video)
                .layer(middleware::from_fn(auth_middleware)),
        )
        .route(
            "/videos/:id",
            delete(content_controller::delete_video)
                .layer(middleware::from_fn(auth_middleware)),
        )
        // Category routes
        .route("/categories", get(content_controller::list_categories))
        .route(
            "/categories",
            post(content_controller::create_category)
                .layer(middleware::from_fn(auth_middleware)),
        )
}