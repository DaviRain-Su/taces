use crate::middleware::auth::AuthUser;
use crate::models::{
    ApiResponse, CreateReviewDto, CreateTagDto, ReplyReviewDto, ReviewQuery, UpdateReviewDto,
    UpdateReviewVisibilityDto,
};
use crate::services::review_service::{ReviewQueryParams, ReviewService};
use crate::AppState;
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use uuid::Uuid;
use validator::Validate;

// ========== 评价相关接口 ==========

pub async fn create_review(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(dto): Json<CreateReviewDto>,
) -> impl IntoResponse {
    // 只有患者可以创建评价
    if auth_user.role != "patient" {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<serde_json::Value>::error(
                "Only patients can create reviews",
            )),
        );
    }

    if let Err(e) = dto.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<serde_json::Value>::error(&format!(
                "Validation error: {}",
                e
            ))),
        );
    }

    match ReviewService::create_review(&state.pool, auth_user.user_id, dto).await {
        Ok(review) => (
            StatusCode::CREATED,
            Json(ApiResponse::success(
                "Review created successfully",
                serde_json::to_value(review).unwrap(),
            )),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<serde_json::Value>::error(&e.to_string())),
        ),
    }
}

pub async fn get_reviews(
    State(state): State<AppState>,
    Query(query): Query<ReviewQuery>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(10);

    let params = ReviewQueryParams {
        doctor_id: query.doctor_id,
        patient_id: query.patient_id,
        rating: query.rating,
        has_comment: query.has_comment,
        has_reply: query.has_reply,
        is_anonymous: query.is_anonymous,
        page,
        page_size,
    };

    match ReviewService::get_reviews(&state.pool, params).await {
        Ok((reviews, total)) => {
            let response = serde_json::json!({
                "reviews": reviews,
                "pagination": {
                    "page": page,
                    "page_size": page_size,
                    "total": total,
                    "total_pages": (total as f64 / page_size as f64).ceil() as i64,
                }
            });
            (
                StatusCode::OK,
                Json(ApiResponse::success(
                    "Reviews retrieved successfully",
                    response,
                )),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<serde_json::Value>::error(&e.to_string())),
        ),
    }
}

pub async fn get_review_by_id(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {
    match ReviewService::get_review_detail(&state.pool, id).await {
        Ok(review) => (
            StatusCode::OK,
            Json(ApiResponse::success(
                "Review retrieved successfully",
                serde_json::to_value(review).unwrap(),
            )),
        ),
        Err(e) => (
            StatusCode::NOT_FOUND,
            Json(ApiResponse::<serde_json::Value>::error(&e.to_string())),
        ),
    }
}

pub async fn update_review(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateReviewDto>,
) -> impl IntoResponse {
    // 只有患者可以更新自己的评价
    if auth_user.role != "patient" {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<serde_json::Value>::error(
                "Only patients can update reviews",
            )),
        );
    }

    if let Err(e) = dto.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<serde_json::Value>::error(&format!(
                "Validation error: {}",
                e
            ))),
        );
    }

    match ReviewService::update_review(&state.pool, id, auth_user.user_id, dto).await {
        Ok(review) => (
            StatusCode::OK,
            Json(ApiResponse::success(
                "Review updated successfully",
                serde_json::to_value(review).unwrap(),
            )),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<serde_json::Value>::error(&e.to_string())),
        ),
    }
}

pub async fn reply_to_review(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(dto): Json<ReplyReviewDto>,
) -> impl IntoResponse {
    // 只有医生可以回复评价
    if auth_user.role != "doctor" {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<serde_json::Value>::error(
                "Only doctors can reply to reviews",
            )),
        );
    }

    if let Err(e) = dto.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<serde_json::Value>::error(&format!(
                "Validation error: {}",
                e
            ))),
        );
    }

    match ReviewService::reply_to_review(&state.pool, id, auth_user.user_id, dto).await {
        Ok(review) => (
            StatusCode::OK,
            Json(ApiResponse::success(
                "Reply added successfully",
                serde_json::to_value(review).unwrap(),
            )),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<serde_json::Value>::error(&e.to_string())),
        ),
    }
}

pub async fn update_review_visibility(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(id): Path<Uuid>,
    Json(dto): Json<UpdateReviewVisibilityDto>,
) -> impl IntoResponse {
    // 只有管理员可以管理评价可见性
    if auth_user.role != "admin" {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<serde_json::Value>::error(
                "Only admins can manage review visibility",
            )),
        );
    }

    match ReviewService::update_review_visibility(&state.pool, id, dto).await {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiResponse::success(
                "Review visibility updated successfully",
                serde_json::json!(()),
            )),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<serde_json::Value>::error(&e.to_string())),
        ),
    }
}

// ========== 标签相关接口 ==========

pub async fn create_tag(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Json(dto): Json<CreateTagDto>,
) -> impl IntoResponse {
    // 只有管理员可以创建标签
    if auth_user.role != "admin" {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<serde_json::Value>::error(
                "Only admins can create tags",
            )),
        );
    }

    if let Err(e) = dto.validate() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<serde_json::Value>::error(&format!(
                "Validation error: {}",
                e
            ))),
        );
    }

    match ReviewService::create_tag(&state.pool, dto).await {
        Ok(tag) => (
            StatusCode::CREATED,
            Json(ApiResponse::success(
                "Tag created successfully",
                serde_json::to_value(tag).unwrap(),
            )),
        ),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(ApiResponse::<serde_json::Value>::error(&e.to_string())),
        ),
    }
}

pub async fn get_tags(
    State(state): State<AppState>,
    Query(params): Query<serde_json::Value>,
) -> impl IntoResponse {
    let category = params
        .get("category")
        .and_then(|v| v.as_str())
        .map(String::from);
    let is_active = params.get("is_active").and_then(|v| v.as_bool());

    match ReviewService::get_tags(&state.pool, category, is_active).await {
        Ok(tags) => (
            StatusCode::OK,
            Json(ApiResponse::success(
                "Tags retrieved successfully",
                serde_json::to_value(tags).unwrap(),
            )),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<serde_json::Value>::error(&e.to_string())),
        ),
    }
}

// ========== 统计相关接口 ==========

pub async fn get_doctor_statistics(
    State(state): State<AppState>,
    Path(doctor_id): Path<Uuid>,
) -> impl IntoResponse {
    match ReviewService::get_doctor_statistics(&state.pool, doctor_id).await {
        Ok(stats) => (
            StatusCode::OK,
            Json(ApiResponse::success(
                "Statistics retrieved successfully",
                serde_json::to_value(stats).unwrap(),
            )),
        ),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<serde_json::Value>::error(&e.to_string())),
        ),
    }
}

// 获取患者的评价列表
pub async fn get_patient_reviews(
    State(state): State<AppState>,
    Extension(auth_user): Extension<AuthUser>,
    Path(patient_id): Path<Uuid>,
    Query(query): Query<ReviewQuery>,
) -> impl IntoResponse {
    // 只能查看自己的评价或者管理员可以查看所有
    if auth_user.role != "admin" && auth_user.user_id != patient_id {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<serde_json::Value>::error(
                "Cannot view other patient's reviews",
            )),
        )
            .into_response();
    }

    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(10);

    let params = ReviewQueryParams {
        doctor_id: None,
        patient_id: Some(patient_id),
        rating: query.rating,
        has_comment: query.has_comment,
        has_reply: query.has_reply,
        is_anonymous: query.is_anonymous,
        page,
        page_size,
    };

    match ReviewService::get_reviews(&state.pool, params).await {
        Ok((reviews, total)) => {
            let response = serde_json::json!({
                "reviews": reviews,
                "pagination": {
                    "page": page,
                    "page_size": page_size,
                    "total": total,
                    "total_pages": (total as f64 / page_size as f64).ceil() as i64,
                }
            });
            (
                StatusCode::OK,
                Json(ApiResponse::success(
                    "Reviews retrieved successfully",
                    response,
                )),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<serde_json::Value>::error(&e.to_string())),
        ),
    }
    .into_response()
}

// 获取医生的评价列表
pub async fn get_doctor_reviews(
    State(state): State<AppState>,
    Path(doctor_id): Path<Uuid>,
    Query(query): Query<ReviewQuery>,
) -> impl IntoResponse {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(10);

    let params = ReviewQueryParams {
        doctor_id: Some(doctor_id),
        patient_id: None,
        rating: query.rating,
        has_comment: query.has_comment,
        has_reply: query.has_reply,
        is_anonymous: query.is_anonymous,
        page,
        page_size,
    };

    match ReviewService::get_reviews(&state.pool, params).await {
        Ok((reviews, total)) => {
            let response = serde_json::json!({
                "reviews": reviews,
                "pagination": {
                    "page": page,
                    "page_size": page_size,
                    "total": total,
                    "total_pages": (total as f64 / page_size as f64).ceil() as i64,
                }
            });
            (
                StatusCode::OK,
                Json(ApiResponse::success(
                    "Reviews retrieved successfully",
                    response,
                )),
            )
        }
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ApiResponse::<serde_json::Value>::error(&e.to_string())),
        ),
    }
}
