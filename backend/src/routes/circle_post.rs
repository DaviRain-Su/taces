use crate::AppState;
use crate::controllers::circle_post_controller::*;
use crate::middleware::auth::auth_middleware;
use axum::{
    middleware,
    routing::{delete, get, post, put},
    Router,
};

pub fn circle_post_routes() -> Router<AppState> {
    Router::new()
        // Post routes
        .route("/posts", post(create_post))
        .route("/posts", get(get_posts))
        .route("/posts/:id", get(get_post_by_id))
        .route("/posts/:id", put(update_post))
        .route("/posts/:id", delete(delete_post))
        .route("/users/:user_id/posts", get(get_user_posts))
        .route("/circles/:circle_id/posts", get(get_circle_posts))
        
        // Like routes
        .route("/posts/:post_id/like", post(toggle_like))
        
        // Comment routes
        .route("/posts/:post_id/comments", post(create_comment))
        .route("/posts/:post_id/comments", get(get_comments))
        .route("/comments/:comment_id", delete(delete_comment))
        
        .layer(middleware::from_fn(auth_middleware))
}