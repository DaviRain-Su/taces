use crate::config::database::DbPool;
use crate::models::video_consultation::*;
use crate::models::appointment::{Appointment, AppointmentStatus};
use crate::utils::errors::AppError;
use chrono::{DateTime, Duration, Utc};
use sqlx::{MySql, Transaction};
use uuid::Uuid;
use std::collections::HashMap;

pub struct VideoConsultationService;

impl VideoConsultationService {
    // Consultation Management
    pub async fn create_consultation(
        db: &DbPool,
        dto: CreateVideoConsultationDto,
    ) -> Result<VideoConsultation, AppError> {
        // Verify appointment exists and is confirmed
        let appointment = Self::get_appointment(db, dto.appointment_id).await?;
        if appointment.status != AppointmentStatus::Confirmed {
            return Err(AppError::BadRequest("预约未确认".to_string()));
        }

        let consultation_id = Uuid::new_v4();
        let room_id = format!("room_{}", Uuid::new_v4().to_string().replace("-", ""));
        let now = Utc::now();

        let query = r#"
            INSERT INTO video_consultations (
                id, appointment_id, doctor_id, patient_id, room_id,
                status, scheduled_start_time, chief_complaint,
                created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, 'waiting', ?, ?, ?, ?)
        "#;

        sqlx::query(query)
            .bind(&consultation_id)
            .bind(&dto.appointment_id)
            .bind(&dto.doctor_id)
            .bind(&dto.patient_id)
            .bind(&room_id)
            .bind(&dto.scheduled_start_time)
            .bind(&dto.chief_complaint)
            .bind(&now)
            .bind(&now)
            .execute(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Self::get_consultation(db, consultation_id).await
    }

    pub async fn get_consultation(
        db: &DbPool,
        consultation_id: Uuid,
    ) -> Result<VideoConsultation, AppError> {
        let query = r#"
            SELECT * FROM video_consultations WHERE id = ?
        "#;

        sqlx::query_as::<_, VideoConsultation>(query)
            .bind(&consultation_id)
            .fetch_one(db)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => AppError::NotFound("视频问诊记录不存在".to_string()),
                _ => AppError::DatabaseError(e.to_string()),
            })
    }

    pub async fn get_consultation_by_room_id(
        db: &DbPool,
        room_id: &str,
    ) -> Result<VideoConsultation, AppError> {
        let query = r#"
            SELECT * FROM video_consultations WHERE room_id = ?
        "#;

        sqlx::query_as::<_, VideoConsultation>(query)
            .bind(room_id)
            .fetch_one(db)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => AppError::NotFound("房间不存在".to_string()),
                _ => AppError::DatabaseError(e.to_string()),
            })
    }

    pub async fn list_consultations(
        db: &DbPool,
        query: ConsultationListQuery,
    ) -> Result<Vec<VideoConsultation>, AppError> {
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).min(100);
        let offset = (page - 1) * page_size;

        let mut sql_query = String::from(
            "SELECT * FROM video_consultations WHERE 1=1"
        );
        let mut bindings: Vec<Box<dyn sqlx::Encode<'_, MySql> + Send + Sync>> = vec![];

        if let Some(doctor_id) = &query.doctor_id {
            sql_query.push_str(" AND doctor_id = ?");
            bindings.push(Box::new(doctor_id.clone()));
        }

        if let Some(patient_id) = &query.patient_id {
            sql_query.push_str(" AND patient_id = ?");
            bindings.push(Box::new(patient_id.clone()));
        }

        if let Some(status) = &query.status {
            sql_query.push_str(" AND status = ?");
            bindings.push(Box::new(status.clone()));
        }

        if let Some(date_from) = &query.date_from {
            sql_query.push_str(" AND scheduled_start_time >= ?");
            bindings.push(Box::new(date_from.clone()));
        }

        if let Some(date_to) = &query.date_to {
            sql_query.push_str(" AND scheduled_start_time <= ?");
            bindings.push(Box::new(date_to.clone()));
        }

        sql_query.push_str(" ORDER BY scheduled_start_time DESC LIMIT ? OFFSET ?");

        let mut query_builder = sqlx::query_as::<_, VideoConsultation>(&sql_query);
        for binding in &bindings {
            query_builder = query_builder.bind(binding.as_ref());
        }

        query_builder
            .bind(page_size)
            .bind(offset)
            .fetch_all(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    // Room Management
    pub async fn join_room(
        db: &DbPool,
        room_id: &str,
        user_id: Uuid,
    ) -> Result<JoinRoomResponse, AppError> {
        let mut tx = db.begin().await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Get consultation
        let consultation = Self::get_consultation_by_room_id(db, room_id).await?;

        // Check if user is authorized
        let (role, token) = if user_id == consultation.doctor_id {
            ("doctor", Self::generate_token(&consultation.id, &user_id, "doctor"))
        } else if user_id == consultation.patient_id {
            ("patient", Self::generate_token(&consultation.id, &user_id, "patient"))
        } else {
            return Err(AppError::Forbidden);
        };

        // Update token in database
        let update_query = if role == "doctor" {
            "UPDATE video_consultations SET doctor_token = ?, updated_at = ? WHERE id = ?"
        } else {
            "UPDATE video_consultations SET patient_token = ?, updated_at = ? WHERE id = ?"
        };

        sqlx::query(update_query)
            .bind(&token)
            .bind(&Utc::now())
            .bind(&consultation.id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Log join event
        Self::log_event_tx(
            &mut tx,
            LogEventDto {
                consultation_id: consultation.id,
                event_type: VideoEventType::Joined,
                event_data: Some(serde_json::json!({ "role": role })),
            },
            user_id,
        ).await?;

        // Get ICE servers configuration
        let ice_servers = Self::get_ice_servers(db).await?;

        tx.commit().await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(JoinRoomResponse {
            room_id: room_id.to_string(),
            token,
            ice_servers,
            role: role.to_string(),
        })
    }

    pub async fn start_consultation(
        db: &DbPool,
        consultation_id: Uuid,
        doctor_id: Uuid,
    ) -> Result<(), AppError> {
        let consultation = Self::get_consultation(db, consultation_id).await?;

        // Verify doctor
        if consultation.doctor_id != doctor_id {
            return Err(AppError::Forbidden);
        }

        // Check status
        if consultation.status != ConsultationStatus::Waiting {
            return Err(AppError::BadRequest("问诊状态不正确".to_string()));
        }

        let now = Utc::now();
        let query = r#"
            UPDATE video_consultations
            SET status = 'in_progress', actual_start_time = ?, updated_at = ?
            WHERE id = ? AND status = 'waiting'
        "#;

        let result = sqlx::query(query)
            .bind(&now)
            .bind(&now)
            .bind(&consultation_id)
            .execute(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AppError::BadRequest("问诊已开始或已结束".to_string()));
        }

        // Log event
        Self::log_event(
            db,
            LogEventDto {
                consultation_id,
                event_type: VideoEventType::RecordingStart,
                event_data: None,
            },
            doctor_id,
        ).await?;

        Ok(())
    }

    pub async fn end_consultation(
        db: &DbPool,
        consultation_id: Uuid,
        doctor_id: Uuid,
        complete_dto: CompleteConsultationDto,
    ) -> Result<(), AppError> {
        let mut tx = db.begin().await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let consultation = Self::get_consultation(db, consultation_id).await?;

        // Verify doctor
        if consultation.doctor_id != doctor_id {
            return Err(AppError::Forbidden);
        }

        // Check status
        if consultation.status != ConsultationStatus::InProgress {
            return Err(AppError::BadRequest("问诊未开始".to_string()));
        }

        // Calculate duration
        let duration = consultation.actual_start_time
            .map(|start| (Utc::now() - start).num_seconds() as i32);

        let now = Utc::now();
        let query = r#"
            UPDATE video_consultations
            SET status = 'completed', end_time = ?, duration = ?,
                diagnosis = ?, treatment_plan = ?, notes = ?, updated_at = ?
            WHERE id = ? AND status = 'in_progress'
        "#;

        sqlx::query(query)
            .bind(&now)
            .bind(&duration)
            .bind(&complete_dto.diagnosis)
            .bind(&complete_dto.treatment_plan)
            .bind(&complete_dto.notes)
            .bind(&now)
            .bind(&consultation_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Update appointment status
        let query = r#"
            UPDATE appointments
            SET status = 'completed', updated_at = ?
            WHERE id = ?
        "#;

        sqlx::query(query)
            .bind(&now)
            .bind(&consultation.appointment_id)
            .execute(&mut *tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Log event
        Self::log_event_tx(
            &mut tx,
            LogEventDto {
                consultation_id,
                event_type: VideoEventType::RecordingEnd,
                event_data: Some(serde_json::json!({ "duration": duration })),
            },
            doctor_id,
        ).await?;

        tx.commit().await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn update_consultation(
        db: &DbPool,
        consultation_id: Uuid,
        doctor_id: Uuid,
        dto: UpdateConsultationDto,
    ) -> Result<(), AppError> {
        let consultation = Self::get_consultation(db, consultation_id).await?;

        // Verify doctor
        if consultation.doctor_id != doctor_id {
            return Err(AppError::Forbidden);
        }

        let query = r#"
            UPDATE video_consultations
            SET chief_complaint = COALESCE(?, chief_complaint),
                diagnosis = COALESCE(?, diagnosis),
                treatment_plan = COALESCE(?, treatment_plan),
                notes = COALESCE(?, notes),
                updated_at = ?
            WHERE id = ?
        "#;

        sqlx::query(query)
            .bind(&dto.chief_complaint)
            .bind(&dto.diagnosis)
            .bind(&dto.treatment_plan)
            .bind(&dto.notes)
            .bind(&Utc::now())
            .bind(&consultation_id)
            .execute(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn rate_consultation(
        db: &DbPool,
        consultation_id: Uuid,
        patient_id: Uuid,
        dto: RateConsultationDto,
    ) -> Result<(), AppError> {
        let consultation = Self::get_consultation(db, consultation_id).await?;

        // Verify patient
        if consultation.patient_id != patient_id {
            return Err(AppError::Forbidden);
        }

        // Check if consultation is completed
        if consultation.status != ConsultationStatus::Completed {
            return Err(AppError::BadRequest("问诊未完成".to_string()));
        }

        let query = r#"
            UPDATE video_consultations
            SET patient_rating = ?, patient_feedback = ?, updated_at = ?
            WHERE id = ?
        "#;

        sqlx::query(query)
            .bind(&dto.rating)
            .bind(&dto.feedback)
            .bind(&Utc::now())
            .bind(&consultation_id)
            .execute(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    // WebRTC Signaling
    pub async fn send_signal(
        db: &DbPool,
        from_user_id: Uuid,
        dto: SendSignalDto,
    ) -> Result<(), AppError> {
        // Verify user is in the room
        let consultation = Self::get_consultation_by_room_id(db, &dto.room_id).await?;
        
        if from_user_id != consultation.doctor_id && from_user_id != consultation.patient_id {
            return Err(AppError::Forbidden);
        }

        // Verify target user is in the room
        if dto.to_user_id != consultation.doctor_id && dto.to_user_id != consultation.patient_id {
            return Err(AppError::BadRequest("目标用户不在房间内".to_string()));
        }

        let signal_id = Uuid::new_v4();
        let query = r#"
            INSERT INTO webrtc_signals (
                id, room_id, from_user_id, to_user_id,
                signal_type, payload, delivered, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, false, ?)
        "#;

        sqlx::query(query)
            .bind(&signal_id)
            .bind(&dto.room_id)
            .bind(&from_user_id)
            .bind(&dto.to_user_id)
            .bind(&dto.signal_type)
            .bind(&dto.payload)
            .bind(&Utc::now())
            .execute(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn receive_signals(
        db: &DbPool,
        room_id: &str,
        user_id: Uuid,
    ) -> Result<Vec<WebRTCSignal>, AppError> {
        // Verify user is in the room
        let consultation = Self::get_consultation_by_room_id(db, room_id).await?;
        
        if user_id != consultation.doctor_id && user_id != consultation.patient_id {
            return Err(AppError::Forbidden);
        }

        let mut tx = db.begin().await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Get undelivered signals
        let query = r#"
            SELECT * FROM webrtc_signals
            WHERE room_id = ? AND to_user_id = ? AND delivered = false
            ORDER BY created_at ASC
        "#;

        let signals = sqlx::query_as::<_, WebRTCSignal>(query)
            .bind(room_id)
            .bind(&user_id)
            .fetch_all(&mut *tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Mark as delivered
        if !signals.is_empty() {
            let signal_ids: Vec<Uuid> = signals.iter().map(|s| s.id).collect();
            let placeholders = signal_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let update_query = format!(
                "UPDATE webrtc_signals SET delivered = true WHERE id IN ({})",
                placeholders
            );

            let mut query_builder = sqlx::query(&update_query);
            for id in signal_ids {
                query_builder = query_builder.bind(id);
            }

            query_builder
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        }

        tx.commit().await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(signals)
    }

    // Recording Management
    pub async fn start_recording(
        db: &DbPool,
        consultation_id: Uuid,
    ) -> Result<VideoRecording, AppError> {
        let consultation = Self::get_consultation(db, consultation_id).await?;

        if consultation.status != ConsultationStatus::InProgress {
            return Err(AppError::BadRequest("问诊未开始".to_string()));
        }

        let recording_id = Uuid::new_v4();
        let query = r#"
            INSERT INTO video_recordings (
                id, consultation_id, status, started_at, created_at
            ) VALUES (?, ?, 'recording', ?, ?)
        "#;

        let now = Utc::now();
        sqlx::query(query)
            .bind(&recording_id)
            .bind(&consultation_id)
            .bind(&now)
            .bind(&now)
            .execute(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Self::get_recording(db, recording_id).await
    }

    pub async fn complete_recording(
        db: &DbPool,
        recording_id: Uuid,
        recording_url: String,
        file_size: i64,
        duration: i32,
    ) -> Result<(), AppError> {
        let query = r#"
            UPDATE video_recordings
            SET status = 'completed', recording_url = ?, file_size = ?,
                duration = ?, completed_at = ?
            WHERE id = ? AND status IN ('recording', 'processing')
        "#;

        let result = sqlx::query(query)
            .bind(&recording_url)
            .bind(&file_size)
            .bind(&duration)
            .bind(&Utc::now())
            .bind(&recording_id)
            .execute(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        if result.rows_affected() == 0 {
            return Err(AppError::BadRequest("录制已完成或失败".to_string()));
        }

        Ok(())
    }

    pub async fn get_recording(
        db: &DbPool,
        recording_id: Uuid,
    ) -> Result<VideoRecording, AppError> {
        let query = r#"
            SELECT * FROM video_recordings WHERE id = ?
        "#;

        sqlx::query_as::<_, VideoRecording>(query)
            .bind(&recording_id)
            .fetch_one(db)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => AppError::NotFound("录制记录不存在".to_string()),
                _ => AppError::DatabaseError(e.to_string()),
            })
    }

    pub async fn get_consultation_recordings(
        db: &DbPool,
        consultation_id: Uuid,
    ) -> Result<Vec<VideoRecording>, AppError> {
        let query = r#"
            SELECT * FROM video_recordings
            WHERE consultation_id = ?
            ORDER BY started_at DESC
        "#;

        sqlx::query_as::<_, VideoRecording>(query)
            .bind(&consultation_id)
            .fetch_all(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    // Template Management
    pub async fn create_template(
        db: &DbPool,
        doctor_id: Uuid,
        dto: CreateConsultationTemplateDto,
    ) -> Result<VideoConsultationTemplate, AppError> {
        let template_id = Uuid::new_v4();
        let now = Utc::now();

        let query = r#"
            INSERT INTO video_consultation_templates (
                id, doctor_id, name, chief_complaint, diagnosis,
                treatment_plan, notes, usage_count, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, 0, ?, ?)
        "#;

        sqlx::query(query)
            .bind(&template_id)
            .bind(&doctor_id)
            .bind(&dto.name)
            .bind(&dto.chief_complaint)
            .bind(&dto.diagnosis)
            .bind(&dto.treatment_plan)
            .bind(&dto.notes)
            .bind(&now)
            .bind(&now)
            .execute(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Self::get_template(db, template_id).await
    }

    pub async fn get_template(
        db: &DbPool,
        template_id: Uuid,
    ) -> Result<VideoConsultationTemplate, AppError> {
        let query = r#"
            SELECT * FROM video_consultation_templates WHERE id = ?
        "#;

        sqlx::query_as::<_, VideoConsultationTemplate>(query)
            .bind(&template_id)
            .fetch_one(db)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => AppError::NotFound("模板不存在".to_string()),
                _ => AppError::DatabaseError(e.to_string()),
            })
    }

    pub async fn list_doctor_templates(
        db: &DbPool,
        doctor_id: Uuid,
    ) -> Result<Vec<VideoConsultationTemplate>, AppError> {
        let query = r#"
            SELECT * FROM video_consultation_templates
            WHERE doctor_id = ?
            ORDER BY usage_count DESC, created_at DESC
        "#;

        sqlx::query_as::<_, VideoConsultationTemplate>(query)
            .bind(&doctor_id)
            .fetch_all(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))
    }

    pub async fn use_template(
        db: &DbPool,
        template_id: Uuid,
        doctor_id: Uuid,
    ) -> Result<VideoConsultationTemplate, AppError> {
        let template = Self::get_template(db, template_id).await?;

        if template.doctor_id != doctor_id {
            return Err(AppError::Forbidden);
        }

        let query = r#"
            UPDATE video_consultation_templates
            SET usage_count = usage_count + 1, updated_at = ?
            WHERE id = ?
        "#;

        sqlx::query(query)
            .bind(&Utc::now())
            .bind(&template_id)
            .execute(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Self::get_template(db, template_id).await
    }

    // Statistics
    pub async fn get_consultation_statistics(
        db: &DbPool,
        doctor_id: Option<Uuid>,
        start_date: Option<DateTime<Utc>>,
        end_date: Option<DateTime<Utc>>,
    ) -> Result<ConsultationStatistics, AppError> {
        let mut where_clauses = vec!["status != 'cancelled'"];
        let mut bindings: Vec<Box<dyn sqlx::Encode<'_, MySql> + Send + Sync>> = vec![];

        if let Some(doc_id) = doctor_id {
            where_clauses.push("doctor_id = ?");
            bindings.push(Box::new(doc_id));
        }

        if let Some(start) = start_date {
            where_clauses.push("scheduled_start_time >= ?");
            bindings.push(Box::new(start));
        }

        if let Some(end) = end_date {
            where_clauses.push("scheduled_start_time <= ?");
            bindings.push(Box::new(end));
        }

        let where_clause = where_clauses.join(" AND ");

        let query = format!(
            r#"
            SELECT 
                COUNT(*) as total_consultations,
                COUNT(CASE WHEN status = 'completed' THEN 1 END) as completed_consultations,
                AVG(CASE WHEN status = 'completed' THEN duration END) as average_duration,
                AVG(patient_rating) as average_rating,
                COUNT(CASE WHEN status = 'no_show' THEN 1 END) * 100.0 / COUNT(*) as no_show_rate
            FROM video_consultations
            WHERE {}
            "#,
            where_clause
        );

        let mut query_builder = sqlx::query(&query);
        for binding in bindings {
            query_builder = query_builder.bind(binding.as_ref());
        }

        let row = query_builder
            .fetch_one(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        use sqlx::Row;
        Ok(ConsultationStatistics {
            total_consultations: row.get::<Option<i64>, _>("total_consultations").unwrap_or(0),
            completed_consultations: row.get::<Option<i64>, _>("completed_consultations").unwrap_or(0),
            average_duration: row.get("average_duration"),
            average_rating: row.get("average_rating"),
            no_show_rate: row.get::<Option<f64>, _>("no_show_rate").unwrap_or(0.0),
        })
    }

    // Helper methods
    async fn get_appointment(
        db: &DbPool,
        appointment_id: Uuid,
    ) -> Result<Appointment, AppError> {
        let query = r#"
            SELECT * FROM appointments WHERE id = ?
        "#;

        sqlx::query_as::<_, Appointment>(query)
            .bind(&appointment_id)
            .fetch_one(db)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => AppError::NotFound("预约不存在".to_string()),
                _ => AppError::DatabaseError(e.to_string()),
            })
    }

    async fn log_event(
        db: &DbPool,
        dto: LogEventDto,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        let event_id = Uuid::new_v4();
        let query = r#"
            INSERT INTO video_call_events (
                id, consultation_id, user_id, event_type,
                event_data, created_at
            ) VALUES (?, ?, ?, ?, ?, ?)
        "#;

        sqlx::query(query)
            .bind(&event_id)
            .bind(&dto.consultation_id)
            .bind(&user_id)
            .bind(&dto.event_type)
            .bind(&dto.event_data)
            .bind(&Utc::now())
            .execute(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn log_event_tx(
        tx: &mut Transaction<'_, MySql>,
        dto: LogEventDto,
        user_id: Uuid,
    ) -> Result<(), AppError> {
        let event_id = Uuid::new_v4();
        let query = r#"
            INSERT INTO video_call_events (
                id, consultation_id, user_id, event_type,
                event_data, created_at
            ) VALUES (?, ?, ?, ?, ?, ?)
        "#;

        sqlx::query(query)
            .bind(&event_id)
            .bind(&dto.consultation_id)
            .bind(&user_id)
            .bind(&dto.event_type)
            .bind(&dto.event_data)
            .bind(&Utc::now())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn get_ice_servers(db: &DbPool) -> Result<serde_json::Value, AppError> {
        let query = r#"
            SELECT config_value FROM system_configs
            WHERE category = 'video_call' AND config_key = 'ice_servers'
        "#;

        let ice_servers: String = sqlx::query_scalar(query)
            .fetch_optional(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?
            .unwrap_or_else(|| r#"[{"urls": ["stun:stun.l.google.com:19302"]}]"#.to_string());

        serde_json::from_str(&ice_servers)
            .map_err(|e| AppError::InternalServerError(format!("解析ICE服务器配置失败: {}", e)))
    }

    fn generate_token(consultation_id: &Uuid, user_id: &Uuid, role: &str) -> String {
        // In production, this should generate a proper JWT or secure token
        format!("{}_{}_{}", consultation_id, user_id, role)
    }

    pub async fn clean_expired_signals(db: &DbPool) -> Result<u64, AppError> {
        let query = r#"
            DELETE FROM webrtc_signals
            WHERE created_at < ? OR delivered = true
        "#;

        let one_hour_ago = Utc::now() - Duration::hours(1);
        let result = sqlx::query(query)
            .bind(&one_hour_ago)
            .execute(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected())
    }
}