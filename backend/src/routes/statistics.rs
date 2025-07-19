use crate::{
    controllers::statistics_controller::*,
    middleware::auth::auth_middleware,
    AppState,
};
use axum::{
    middleware,
    routing::get,
    Router,
};

pub fn routes() -> Router<AppState> {
    let public_routes = Router::new()
        // 公开统计接口
        .route("/departments", get(get_department_statistics))
        .route("/top-doctors", get(get_top_doctors))
        .route("/top-content", get(get_top_content));

    let protected_routes = Router::new()
        // 管理员统计
        .route("/dashboard", get(get_dashboard_stats))
        .route("/appointment-trends", get(get_appointment_trends))
        .route("/time-slots", get(get_time_slot_statistics))
        .route("/content", get(get_content_statistics))
        .route("/live-streams", get(get_live_stream_statistics))
        .route("/circles", get(get_circle_statistics))
        .route("/user-growth", get(get_user_growth_statistics))
        .route("/appointment-heatmap", get(get_appointment_heatmap))
        .route("/export", get(export_data))
        
        // 医生统计
        .route("/doctor/:doctor_id", get(get_doctor_statistics))
        
        // 患者统计
        .route("/patient", get(get_patient_statistics))
        
        // 所有受保护的路由都需要认证
        .layer(middleware::from_fn(auth_middleware));

    Router::new()
        .merge(public_routes)
        .merge(protected_routes)
}