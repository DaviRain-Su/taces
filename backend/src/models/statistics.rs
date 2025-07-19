use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize)]
pub struct DashboardStats {
    pub total_users: i64,
    pub total_doctors: i64,
    pub total_patients: i64,
    pub total_appointments: i64,
    pub total_prescriptions: i64,
    pub today_appointments: i64,
    pub pending_appointments: i64,
    pub completed_appointments: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DoctorStats {
    pub total_appointments: i64,
    pub completed_appointments: i64,
    pub cancelled_appointments: i64,
    pub total_patients: i64,
    pub total_prescriptions: i64,
    pub average_rating: Option<f64>,
    pub total_reviews: i64,
    pub today_appointments: i64,
    pub this_week_appointments: i64,
    pub this_month_appointments: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PatientStats {
    pub total_appointments: i64,
    pub completed_appointments: i64,
    pub upcoming_appointments: i64,
    pub total_prescriptions: i64,
    pub total_doctors_visited: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AppointmentTrend {
    pub date: NaiveDate,
    pub count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DepartmentStats {
    pub department_id: Uuid,
    pub department_name: String,
    pub total_doctors: i64,
    pub total_appointments: i64,
    pub average_rating: Option<f64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimeSlotStats {
    pub time_slot: String,
    pub count: i64,
    pub percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RevenueStats {
    pub date: NaiveDate,
    pub appointment_count: i64,
    pub total_revenue: f64,
    pub average_revenue_per_appointment: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ContentStats {
    pub total_articles: i64,
    pub total_videos: i64,
    pub total_views: i64,
    pub published_articles: i64,
    pub draft_articles: i64,
    pub published_videos: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LiveStreamStats {
    pub total_streams: i64,
    pub scheduled_streams: i64,
    pub completed_streams: i64,
    pub total_viewers: i64,
    pub average_viewers_per_stream: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CircleStats {
    pub total_circles: i64,
    pub total_members: i64,
    pub total_posts: i64,
    pub active_circles: i64,  // Circles with posts in last 30 days
    pub average_members_per_circle: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserGrowthStats {
    pub date: NaiveDate,
    pub new_users: i64,
    pub new_doctors: i64,
    pub new_patients: i64,
    pub cumulative_users: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TopDoctor {
    pub doctor_id: Uuid,
    pub doctor_name: String,
    pub department: String,
    pub appointment_count: i64,
    pub average_rating: Option<f64>,
    pub review_count: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TopContent {
    pub content_id: Uuid,
    pub title: String,
    pub content_type: String,
    pub author_name: String,
    pub view_count: i64,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DateRangeQuery {
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportQuery {
    pub export_type: ExportType,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub format: ExportFormat,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ExportType {
    Appointments,
    Prescriptions,
    Users,
    Doctors,
    Revenue,
    Content,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ExportFormat {
    CSV,
    Excel,
    PDF,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HeatmapData {
    pub hour: i32,
    pub day_of_week: i32,  // 0 = Sunday, 6 = Saturday
    pub count: i64,
}