use crate::{
    models::{statistics::*, ApiResponse},
    services::statistics_service::StatisticsService,
    utils::jwt::Claims,
    AppState,
};
use axum::{
    extract::{Extension, Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};
use chrono::{Duration, Local, NaiveDate};
use serde_json::json;
use uuid::Uuid;

/// 获取管理员仪表盘统计（仅管理员）
pub async fn get_dashboard_stats(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> impl IntoResponse {
    // 检查权限
    if claims.role != "admin" {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error("无权限访问")),
        )
            .into_response();
    }

    match StatisticsService::get_dashboard_stats(&state.pool).await {
        Ok(stats) => Json(ApiResponse::success("获取仪表盘统计成功", stats)).into_response(),
        Err(e) => {
            eprintln!("获取仪表盘统计失败: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("获取仪表盘统计失败")),
            )
                .into_response()
        }
    }
}

/// 获取医生统计数据
pub async fn get_doctor_statistics(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Path(doctor_id): Path<Uuid>,
) -> impl IntoResponse {
    // 医生只能查看自己的统计，管理员可以查看所有医生
    if claims.role == "doctor" {
        // 获取医生ID
        let doctor_result: Result<Option<(String,)>, sqlx::Error> = sqlx::query_as(
            "SELECT id FROM doctors WHERE user_id = ?"
        )
        .bind(claims.user_id.to_string())
        .fetch_optional(&state.pool)
        .await;

        match doctor_result {
            Ok(Some((id,))) => {
                let doc_id = Uuid::parse_str(&id).unwrap();
                if doc_id != doctor_id {
                    return (
                        StatusCode::FORBIDDEN,
                        Json(ApiResponse::<()>::error("无权限查看其他医生的统计")),
                    )
                        .into_response();
                }
            }
            _ => {
                return (
                    StatusCode::FORBIDDEN,
                    Json(ApiResponse::<()>::error("医生信息不存在")),
                )
                    .into_response();
            }
        }
    } else if claims.role != "admin" {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error("无权限访问")),
        )
            .into_response();
    }

    match StatisticsService::get_doctor_stats(&state.pool, doctor_id).await {
        Ok(stats) => Json(ApiResponse::success("获取医生统计成功", stats)).into_response(),
        Err(e) => {
            eprintln!("获取医生统计失败: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("获取医生统计失败")),
            )
                .into_response()
        }
    }
}

/// 获取患者统计数据
pub async fn get_patient_statistics(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> impl IntoResponse {
    // 患者只能查看自己的统计
    if claims.role != "patient" {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error("无权限访问")),
        )
            .into_response();
    }

    match StatisticsService::get_patient_stats(&state.pool, claims.user_id).await {
        Ok(stats) => Json(ApiResponse::success("获取患者统计成功", stats)).into_response(),
        Err(e) => {
            eprintln!("获取患者统计失败: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("获取患者统计失败")),
            )
                .into_response()
        }
    }
}

/// 获取预约趋势（管理员）
pub async fn get_appointment_trends(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(date_range): Query<DateRangeQuery>,
) -> impl IntoResponse {
    if claims.role != "admin" {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error("无权限访问")),
        )
            .into_response();
    }

    // 设置默认日期范围（最近30天）
    let end_date = date_range.end_date.unwrap_or_else(|| Local::now().naive_local().date());
    let start_date = date_range.start_date.unwrap_or_else(|| end_date - Duration::days(29));

    match StatisticsService::get_appointment_trends(&state.pool, start_date, end_date).await {
        Ok(trends) => Json(ApiResponse::success("获取预约趋势成功", trends)).into_response(),
        Err(e) => {
            eprintln!("获取预约趋势失败: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("获取预约趋势失败")),
            )
                .into_response()
        }
    }
}

/// 获取科室统计（公开）
pub async fn get_department_statistics(State(state): State<AppState>) -> impl IntoResponse {
    match StatisticsService::get_department_stats(&state.pool).await {
        Ok(stats) => Json(ApiResponse::success("获取科室统计成功", stats)).into_response(),
        Err(e) => {
            eprintln!("获取科室统计失败: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("获取科室统计失败")),
            )
                .into_response()
        }
    }
}

/// 获取时间段分布统计（管理员）
pub async fn get_time_slot_statistics(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> impl IntoResponse {
    if claims.role != "admin" {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error("无权限访问")),
        )
            .into_response();
    }

    match StatisticsService::get_time_slot_stats(&state.pool).await {
        Ok(stats) => Json(ApiResponse::success("获取时间段统计成功", stats)).into_response(),
        Err(e) => {
            eprintln!("获取时间段统计失败: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("获取时间段统计失败")),
            )
                .into_response()
        }
    }
}

/// 获取内容统计（管理员）
pub async fn get_content_statistics(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> impl IntoResponse {
    if claims.role != "admin" {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error("无权限访问")),
        )
            .into_response();
    }

    match StatisticsService::get_content_stats(&state.pool).await {
        Ok(stats) => Json(ApiResponse::success("获取内容统计成功", stats)).into_response(),
        Err(e) => {
            eprintln!("获取内容统计失败: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("获取内容统计失败")),
            )
                .into_response()
        }
    }
}

/// 获取直播统计（管理员）
pub async fn get_live_stream_statistics(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> impl IntoResponse {
    if claims.role != "admin" {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error("无权限访问")),
        )
            .into_response();
    }

    match StatisticsService::get_live_stream_stats(&state.pool).await {
        Ok(stats) => Json(ApiResponse::success("获取直播统计成功", stats)).into_response(),
        Err(e) => {
            eprintln!("获取直播统计失败: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("获取直播统计失败")),
            )
                .into_response()
        }
    }
}

/// 获取圈子统计（管理员）
pub async fn get_circle_statistics(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> impl IntoResponse {
    if claims.role != "admin" {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error("无权限访问")),
        )
            .into_response();
    }

    match StatisticsService::get_circle_stats(&state.pool).await {
        Ok(stats) => Json(ApiResponse::success("获取圈子统计成功", stats)).into_response(),
        Err(e) => {
            eprintln!("获取圈子统计失败: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("获取圈子统计失败")),
            )
                .into_response()
        }
    }
}

/// 获取用户增长统计（管理员）
pub async fn get_user_growth_statistics(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(date_range): Query<DateRangeQuery>,
) -> impl IntoResponse {
    if claims.role != "admin" {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error("无权限访问")),
        )
            .into_response();
    }

    // 设置默认日期范围（最近30天）
    let end_date = date_range.end_date.unwrap_or_else(|| Local::now().naive_local().date());
    let start_date = date_range.start_date.unwrap_or_else(|| end_date - Duration::days(29));

    match StatisticsService::get_user_growth_stats(&state.pool, start_date, end_date).await {
        Ok(stats) => Json(ApiResponse::success("获取用户增长统计成功", stats)).into_response(),
        Err(e) => {
            eprintln!("获取用户增长统计失败: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("获取用户增长统计失败")),
            )
                .into_response()
        }
    }
}

/// 获取热门医生（公开）
pub async fn get_top_doctors(State(state): State<AppState>) -> impl IntoResponse {
    match StatisticsService::get_top_doctors(&state.pool, 10).await {
        Ok(doctors) => Json(ApiResponse::success("获取热门医生成功", doctors)).into_response(),
        Err(e) => {
            eprintln!("获取热门医生失败: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("获取热门医生失败")),
            )
                .into_response()
        }
    }
}

/// 获取热门内容（公开）
pub async fn get_top_content(State(state): State<AppState>) -> impl IntoResponse {
    match StatisticsService::get_top_content(&state.pool, 10).await {
        Ok(content) => Json(ApiResponse::success("获取热门内容成功", content)).into_response(),
        Err(e) => {
            eprintln!("获取热门内容失败: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("获取热门内容失败")),
            )
                .into_response()
        }
    }
}

/// 获取预约热力图（管理员）
pub async fn get_appointment_heatmap(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> impl IntoResponse {
    if claims.role != "admin" {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error("无权限访问")),
        )
            .into_response();
    }

    match StatisticsService::get_appointment_heatmap(&state.pool).await {
        Ok(heatmap) => Json(ApiResponse::success("获取预约热力图成功", heatmap)).into_response(),
        Err(e) => {
            eprintln!("获取预约热力图失败: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiResponse::<()>::error("获取预约热力图失败")),
            )
                .into_response()
        }
    }
}

/// 导出数据（管理员）
pub async fn export_data(
    State(state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Query(export_query): Query<ExportQuery>,
) -> impl IntoResponse {
    if claims.role != "admin" {
        return (
            StatusCode::FORBIDDEN,
            Json(ApiResponse::<()>::error("无权限访问")),
        )
            .into_response();
    }

    // 这里只实现CSV导出的示例
    match export_query.export_type {
        ExportType::Appointments => {
            match StatisticsService::export_appointments_csv(
                &state.pool,
                export_query.start_date,
                export_query.end_date,
            )
            .await
            {
                Ok(csv_data) => {
                    // 实际实现中，这里应该返回文件下载响应
                    Json(ApiResponse::success(
                        "导出成功",
                        json!({ "data": csv_data, "format": "csv" }),
                    ))
                    .into_response()
                }
                Err(e) => {
                    eprintln!("导出数据失败: {:?}", e);
                    (
                        StatusCode::INTERNAL_SERVER_ERROR,
                        Json(ApiResponse::<()>::error("导出数据失败")),
                    )
                        .into_response()
                }
            }
        }
        _ => (
            StatusCode::NOT_IMPLEMENTED,
            Json(ApiResponse::<()>::error("该导出类型尚未实现")),
        )
            .into_response(),
    }
}