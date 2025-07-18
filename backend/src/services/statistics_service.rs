use crate::{
    config::database::DbPool,
    models::statistics::*,
};
use chrono::{Datelike, Duration, Local, NaiveDate, Utc};
use sqlx::query;
use uuid::Uuid;

pub struct StatisticsService;

impl StatisticsService {
    /// 获取管理员仪表盘统计数据
    pub async fn get_dashboard_stats(pool: &DbPool) -> Result<DashboardStats, sqlx::Error> {
        let stats = query!(
            r#"
            SELECT 
                (SELECT COUNT(*) FROM users WHERE status = 'active') as total_users,
                (SELECT COUNT(*) FROM users WHERE role = 'doctor' AND status = 'active') as total_doctors,
                (SELECT COUNT(*) FROM users WHERE role = 'patient' AND status = 'active') as total_patients,
                (SELECT COUNT(*) FROM appointments) as total_appointments,
                (SELECT COUNT(*) FROM prescriptions) as total_prescriptions,
                (SELECT COUNT(*) FROM appointments WHERE DATE(appointment_date) = CURDATE()) as today_appointments,
                (SELECT COUNT(*) FROM appointments WHERE status = 'pending') as pending_appointments,
                (SELECT COUNT(*) FROM appointments WHERE status = 'completed') as completed_appointments
            "#
        )
        .fetch_one(pool)
        .await?;

        Ok(DashboardStats {
            total_users: stats.total_users.unwrap_or(0),
            total_doctors: stats.total_doctors.unwrap_or(0),
            total_patients: stats.total_patients.unwrap_or(0),
            total_appointments: stats.total_appointments.unwrap_or(0),
            total_prescriptions: stats.total_prescriptions.unwrap_or(0),
            today_appointments: stats.today_appointments.unwrap_or(0),
            pending_appointments: stats.pending_appointments.unwrap_or(0),
            completed_appointments: stats.completed_appointments.unwrap_or(0),
        })
    }

    /// 获取医生统计数据
    pub async fn get_doctor_stats(
        pool: &DbPool,
        doctor_id: Uuid,
    ) -> Result<DoctorStats, sqlx::Error> {
        let stats = query!(
            r#"
            SELECT 
                COUNT(DISTINCT a.id) as total_appointments,
                COUNT(DISTINCT CASE WHEN a.status = 'completed' THEN a.id END) as completed_appointments,
                COUNT(DISTINCT CASE WHEN a.status = 'cancelled' THEN a.id END) as cancelled_appointments,
                COUNT(DISTINCT a.patient_id) as total_patients,
                COUNT(DISTINCT p.id) as total_prescriptions,
                AVG(r.rating) as average_rating,
                COUNT(DISTINCT r.id) as total_reviews,
                COUNT(DISTINCT CASE WHEN DATE(a.appointment_date) = CURDATE() THEN a.id END) as today_appointments,
                COUNT(DISTINCT CASE WHEN a.appointment_date >= DATE_SUB(CURDATE(), INTERVAL 7 DAY) THEN a.id END) as this_week_appointments,
                COUNT(DISTINCT CASE WHEN a.appointment_date >= DATE_SUB(CURDATE(), INTERVAL 30 DAY) THEN a.id END) as this_month_appointments
            FROM doctors d
            LEFT JOIN appointments a ON d.id = a.doctor_id
            LEFT JOIN prescriptions p ON d.id = p.doctor_id
            LEFT JOIN patient_reviews r ON d.id = r.doctor_id
            WHERE d.id = ?
            "#,
            doctor_id.to_string()
        )
        .fetch_one(pool)
        .await?;

        Ok(DoctorStats {
            total_appointments: stats.total_appointments,
            completed_appointments: stats.completed_appointments,
            cancelled_appointments: stats.cancelled_appointments,
            total_patients: stats.total_patients,
            total_prescriptions: stats.total_prescriptions,
            average_rating: stats.average_rating,
            total_reviews: stats.total_reviews,
            today_appointments: stats.today_appointments,
            this_week_appointments: stats.this_week_appointments,
            this_month_appointments: stats.this_month_appointments,
        })
    }

    /// 获取患者统计数据
    pub async fn get_patient_stats(
        pool: &DbPool,
        patient_id: Uuid,
    ) -> Result<PatientStats, sqlx::Error> {
        let stats = query!(
            r#"
            SELECT 
                COUNT(DISTINCT a.id) as total_appointments,
                COUNT(DISTINCT CASE WHEN a.status = 'completed' THEN a.id END) as completed_appointments,
                COUNT(DISTINCT CASE WHEN a.status IN ('pending', 'confirmed') AND a.appointment_date > NOW() THEN a.id END) as upcoming_appointments,
                COUNT(DISTINCT p.id) as total_prescriptions,
                COUNT(DISTINCT a.doctor_id) as total_doctors_visited
            FROM appointments a
            LEFT JOIN prescriptions p ON a.patient_id = p.patient_id
            WHERE a.patient_id = ?
            "#,
            patient_id.to_string()
        )
        .fetch_one(pool)
        .await?;

        Ok(PatientStats {
            total_appointments: stats.total_appointments,
            completed_appointments: stats.completed_appointments,
            upcoming_appointments: stats.upcoming_appointments,
            total_prescriptions: stats.total_prescriptions,
            total_doctors_visited: stats.total_doctors_visited,
        })
    }

    /// 获取预约趋势数据
    pub async fn get_appointment_trends(
        pool: &DbPool,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<AppointmentTrend>, sqlx::Error> {
        let trends = query!(
            r#"
            SELECT 
                DATE(appointment_date) as date,
                COUNT(*) as count
            FROM appointments
            WHERE DATE(appointment_date) BETWEEN ? AND ?
            GROUP BY DATE(appointment_date)
            ORDER BY date
            "#,
            start_date,
            end_date
        )
        .fetch_all(pool)
        .await?;

        Ok(trends
            .into_iter()
            .map(|row| AppointmentTrend {
                date: row.date.unwrap(),
                count: row.count,
            })
            .collect())
    }

    /// 获取科室统计数据
    pub async fn get_department_stats(pool: &DbPool) -> Result<Vec<DepartmentStats>, sqlx::Error> {
        let stats = query!(
            r#"
            SELECT 
                dep.id as department_id,
                dep.name as department_name,
                COUNT(DISTINCT d.id) as total_doctors,
                COUNT(DISTINCT a.id) as total_appointments,
                AVG(r.rating) as average_rating
            FROM departments dep
            LEFT JOIN doctors d ON dep.name = d.department
            LEFT JOIN appointments a ON d.id = a.doctor_id
            LEFT JOIN patient_reviews r ON d.id = r.doctor_id
            GROUP BY dep.id, dep.name
            ORDER BY total_appointments DESC
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(stats
            .into_iter()
            .map(|row| DepartmentStats {
                department_id: Uuid::parse_str(&row.department_id).unwrap(),
                department_name: row.department_name,
                total_doctors: row.total_doctors as i64,
                total_appointments: row.total_appointments as i64,
                average_rating: row.average_rating,
            })
            .collect())
    }

    /// 获取时间段分布统计
    pub async fn get_time_slot_stats(pool: &DbPool) -> Result<Vec<TimeSlotStats>, sqlx::Error> {
        let stats = query!(
            r#"
            SELECT 
                time_slot,
                COUNT(*) as count
            FROM appointments
            WHERE status IN ('completed', 'confirmed', 'pending')
            GROUP BY time_slot
            ORDER BY count DESC
            "#
        )
        .fetch_all(pool)
        .await?;

        let total: i64 = stats.iter().map(|s| s.count).sum();

        Ok(stats
            .into_iter()
            .map(|row| TimeSlotStats {
                time_slot: row.time_slot,
                count: row.count,
                percentage: if total > 0 {
                    (row.count as f64 / total as f64) * 100.0
                } else {
                    0.0
                },
            })
            .collect())
    }

    /// 获取内容统计数据
    pub async fn get_content_stats(pool: &DbPool) -> Result<ContentStats, sqlx::Error> {
        let stats = query!(
            r#"
            SELECT 
                (SELECT COUNT(*) FROM articles) as total_articles,
                (SELECT COUNT(*) FROM videos) as total_videos,
                (SELECT SUM(view_count) FROM articles) + (SELECT SUM(view_count) FROM videos) as total_views,
                (SELECT COUNT(*) FROM articles WHERE status = 'published') as published_articles,
                (SELECT COUNT(*) FROM articles WHERE status = 'draft') as draft_articles,
                (SELECT COUNT(*) FROM videos WHERE status = 'published') as published_videos
            "#
        )
        .fetch_one(pool)
        .await?;

        Ok(ContentStats {
            total_articles: stats.total_articles.unwrap_or(0),
            total_videos: stats.total_videos.unwrap_or(0),
            total_views: stats.total_views.unwrap_or(0),
            published_articles: stats.published_articles.unwrap_or(0),
            draft_articles: stats.draft_articles.unwrap_or(0),
            published_videos: stats.published_videos.unwrap_or(0),
        })
    }

    /// 获取直播统计数据
    pub async fn get_live_stream_stats(pool: &DbPool) -> Result<LiveStreamStats, sqlx::Error> {
        let stats = query!(
            r#"
            SELECT 
                COUNT(*) as total_streams,
                COUNT(CASE WHEN status = 'scheduled' THEN 1 END) as scheduled_streams,
                COUNT(CASE WHEN status = 'ended' THEN 1 END) as completed_streams,
                0 as total_viewers,  -- 需要实际的观看记录表
                0.0 as average_viewers_per_stream
            FROM live_streams
            "#
        )
        .fetch_one(pool)
        .await?;

        Ok(LiveStreamStats {
            total_streams: stats.total_streams,
            scheduled_streams: stats.scheduled_streams as i64,
            completed_streams: stats.completed_streams as i64,
            total_viewers: stats.total_viewers,
            average_viewers_per_stream: stats.average_viewers_per_stream,
        })
    }

    /// 获取圈子统计数据
    pub async fn get_circle_stats(pool: &DbPool) -> Result<CircleStats, sqlx::Error> {
        let stats = query!(
            r#"
            SELECT 
                COUNT(DISTINCT c.id) as total_circles,
                COUNT(DISTINCT cm.user_id) as total_members,
                COUNT(DISTINCT cp.id) as total_posts,
                COUNT(DISTINCT CASE 
                    WHEN EXISTS (
                        SELECT 1 FROM circle_posts cp2 
                        WHERE cp2.circle_id = c.id 
                        AND cp2.created_at >= DATE_SUB(NOW(), INTERVAL 30 DAY)
                    ) THEN c.id 
                END) as active_circles
            FROM circles c
            LEFT JOIN circle_members cm ON c.id = cm.circle_id
            LEFT JOIN circle_posts cp ON c.id = cp.circle_id
            "#
        )
        .fetch_one(pool)
        .await?;

        let average_members = if stats.total_circles > 0 {
            stats.total_members as f64 / stats.total_circles as f64
        } else {
            0.0
        };

        Ok(CircleStats {
            total_circles: stats.total_circles,
            total_members: stats.total_members as i64,
            total_posts: stats.total_posts as i64,
            active_circles: stats.active_circles as i64,
            average_members_per_circle: average_members,
        })
    }

    /// 获取用户增长统计
    pub async fn get_user_growth_stats(
        pool: &DbPool,
        start_date: NaiveDate,
        end_date: NaiveDate,
    ) -> Result<Vec<UserGrowthStats>, sqlx::Error> {
        let stats = query!(
            r#"
            SELECT 
                DATE(created_at) as date,
                COUNT(*) as new_users,
                COUNT(CASE WHEN role = 'doctor' THEN 1 END) as new_doctors,
                COUNT(CASE WHEN role = 'patient' THEN 1 END) as new_patients
            FROM users
            WHERE DATE(created_at) BETWEEN ? AND ?
            GROUP BY DATE(created_at)
            ORDER BY date
            "#,
            start_date,
            end_date
        )
        .fetch_all(pool)
        .await?;

        let mut cumulative_users = 0i64;
        let growth_stats: Vec<UserGrowthStats> = stats
            .into_iter()
            .map(|row| {
                cumulative_users += row.new_users;
                UserGrowthStats {
                    date: row.date.unwrap(),
                    new_users: row.new_users,
                    new_doctors: row.new_doctors as i64,
                    new_patients: row.new_patients as i64,
                    cumulative_users,
                }
            })
            .collect();

        Ok(growth_stats)
    }

    /// 获取热门医生
    pub async fn get_top_doctors(
        pool: &DbPool,
        limit: i64,
    ) -> Result<Vec<TopDoctor>, sqlx::Error> {
        let doctors = query!(
            r#"
            SELECT 
                d.id as doctor_id,
                u.name as doctor_name,
                dep.name as department,
                COUNT(DISTINCT a.id) as appointment_count,
                AVG(r.rating) as average_rating,
                COUNT(DISTINCT r.id) as review_count
            FROM doctors d
            JOIN users u ON d.user_id = u.id
            LEFT JOIN departments dep ON d.department = dep.name
            LEFT JOIN appointments a ON d.id = a.doctor_id
            LEFT JOIN patient_reviews r ON d.id = r.doctor_id
            WHERE u.status = 'active'
            GROUP BY d.id, u.name, dep.name
            ORDER BY appointment_count DESC
            LIMIT ?
            "#,
            limit
        )
        .fetch_all(pool)
        .await?;

        Ok(doctors
            .into_iter()
            .map(|row| TopDoctor {
                doctor_id: Uuid::parse_str(&row.doctor_id).unwrap(),
                doctor_name: row.doctor_name,
                department: row.department.unwrap_or_else(|| "未分配".to_string()),
                appointment_count: row.appointment_count as i64,
                average_rating: row.average_rating,
                review_count: row.review_count as i64,
            })
            .collect())
    }

    /// 获取热门内容
    pub async fn get_top_content(
        pool: &DbPool,
        limit: i64,
    ) -> Result<Vec<TopContent>, sqlx::Error> {
        // 获取热门文章
        let articles = query!(
            r#"
            SELECT 
                a.id as content_id,
                a.title,
                'article' as content_type,
                u.name as author_name,
                a.view_count,
                a.created_at
            FROM articles a
            JOIN users u ON a.author_id = u.id
            WHERE a.status = 'published'
            ORDER BY a.view_count DESC
            LIMIT ?
            "#,
            limit / 2
        )
        .fetch_all(pool)
        .await?;

        // 获取热门视频
        let videos = query!(
            r#"
            SELECT 
                v.id as content_id,
                v.title,
                'video' as content_type,
                u.name as author_name,
                v.view_count,
                v.created_at
            FROM videos v
            JOIN users u ON v.author_id = u.id
            WHERE v.status = 'published'
            ORDER BY v.view_count DESC
            LIMIT ?
            "#,
            limit / 2
        )
        .fetch_all(pool)
        .await?;

        let mut all_content: Vec<TopContent> = articles
            .into_iter()
            .chain(videos.into_iter())
            .map(|row| TopContent {
                content_id: Uuid::parse_str(&row.content_id).unwrap(),
                title: row.title,
                content_type: row.content_type,
                author_name: row.author_name,
                view_count: row.view_count,
                created_at: row.created_at,
            })
            .collect();

        // 按浏览量排序
        all_content.sort_by(|a, b| b.view_count.cmp(&a.view_count));
        all_content.truncate(limit as usize);

        Ok(all_content)
    }

    /// 获取预约热力图数据
    pub async fn get_appointment_heatmap(pool: &DbPool) -> Result<Vec<HeatmapData>, sqlx::Error> {
        let heatmap = query!(
            r#"
            SELECT 
                HOUR(appointment_date) as hour,
                DAYOFWEEK(appointment_date) - 1 as day_of_week,  -- MySQL: 1=Sunday, 7=Saturday
                COUNT(*) as count
            FROM appointments
            WHERE status IN ('completed', 'confirmed', 'pending')
            GROUP BY hour, day_of_week
            ORDER BY day_of_week, hour
            "#
        )
        .fetch_all(pool)
        .await?;

        Ok(heatmap
            .into_iter()
            .map(|row| HeatmapData {
                hour: row.hour as i32,
                day_of_week: row.day_of_week as i32,
                count: row.count,
            })
            .collect())
    }

    /// 导出数据到CSV（示例实现）
    pub async fn export_appointments_csv(
        pool: &DbPool,
        start_date: Option<NaiveDate>,
        end_date: Option<NaiveDate>,
    ) -> Result<String, sqlx::Error> {
        let mut query = String::from(
            r#"
            SELECT 
                a.id,
                u_patient.name as patient_name,
                u_doctor.name as doctor_name,
                dep.name as department,
                a.appointment_date,
                a.time_slot,
                a.visit_type,
                a.symptoms,
                a.status,
                a.created_at
            FROM appointments a
            JOIN users u_patient ON a.patient_id = u_patient.id
            JOIN doctors d ON a.doctor_id = d.id
            JOIN users u_doctor ON d.user_id = u_doctor.id
            LEFT JOIN departments dep ON d.department = dep.name
            WHERE 1=1
            "#
        );

        if let Some(start) = start_date {
            query.push_str(&format!(" AND DATE(a.appointment_date) >= '{}'", start));
        }
        if let Some(end) = end_date {
            query.push_str(&format!(" AND DATE(a.appointment_date) <= '{}'", end));
        }
        query.push_str(" ORDER BY a.appointment_date DESC");

        // 这里只是示例，实际实现需要执行查询并格式化为CSV
        Ok("id,patient_name,doctor_name,department,appointment_date,time_slot,visit_type,symptoms,status,created_at\n".to_string())
    }
}