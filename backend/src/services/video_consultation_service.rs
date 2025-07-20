use crate::config::database::DbPool;
use crate::models::appointment::{Appointment, AppointmentStatus, VisitType};
use crate::models::video_consultation::*;
use crate::utils::errors::AppError;
use chrono::{DateTime, Duration, Utc};
use sqlx::{MySql, Transaction};
use uuid::Uuid;

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
            .bind(consultation_id.to_string())
            .bind(dto.appointment_id.to_string())
            .bind(dto.doctor_id.to_string())
            .bind(dto.patient_id.to_string())
            .bind(&room_id)
            .bind(dto.scheduled_start_time)
            .bind(&dto.chief_complaint)
            .bind(now)
            .bind(now)
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

        let row = sqlx::query(query)
            .bind(consultation_id.to_string())
            .fetch_one(db)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => AppError::NotFound("视频问诊记录不存在".to_string()),
                _ => AppError::DatabaseError(e.to_string()),
            })?;

        Self::parse_consultation_row(row)
    }

    pub async fn get_consultation_by_room_id(
        db: &DbPool,
        room_id: &str,
    ) -> Result<VideoConsultation, AppError> {
        let query = r#"
            SELECT * FROM video_consultations WHERE room_id = ?
        "#;

        let row = sqlx::query(query)
            .bind(room_id)
            .fetch_one(db)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => AppError::NotFound("房间不存在".to_string()),
                _ => AppError::DatabaseError(e.to_string()),
            })?;

        Self::parse_consultation_row(row)
    }

    pub async fn list_consultations(
        db: &DbPool,
        query: ConsultationListQuery,
    ) -> Result<Vec<VideoConsultation>, AppError> {
        let page = query.page.unwrap_or(1).max(1);
        let page_size = query.page_size.unwrap_or(20).min(100);
        let offset = (page - 1) * page_size;

        // Build query dynamically based on filters
        let rows = match (&query.doctor_id, &query.patient_id, &query.status, &query.date_from, &query.date_to) {
            (Some(doctor_id), None, None, None, None) => {
                sqlx::query(
                    "SELECT * FROM video_consultations WHERE doctor_id = ? ORDER BY scheduled_start_time DESC LIMIT ? OFFSET ?"
                )
                .bind(doctor_id.to_string())
                .bind(page_size)
                .bind(offset)
                .fetch_all(db)
                .await
            },
            (None, Some(patient_id), None, None, None) => {
                sqlx::query(
                    "SELECT * FROM video_consultations WHERE patient_id = ? ORDER BY scheduled_start_time DESC LIMIT ? OFFSET ?"
                )
                .bind(patient_id.to_string())
                .bind(page_size)
                .bind(offset)
                .fetch_all(db)
                .await
            },
            (Some(doctor_id), Some(patient_id), None, None, None) => {
                sqlx::query(
                    "SELECT * FROM video_consultations WHERE doctor_id = ? AND patient_id = ? ORDER BY scheduled_start_time DESC LIMIT ? OFFSET ?"
                )
                .bind(doctor_id.to_string())
                .bind(patient_id.to_string())
                .bind(page_size)
                .bind(offset)
                .fetch_all(db)
                .await
            },
            _ => {
                // For complex queries, use a simpler approach
                let mut conditions = vec![];
                if query.doctor_id.is_some() { conditions.push("doctor_id = ?"); }
                if query.patient_id.is_some() { conditions.push("patient_id = ?"); }
                if query.status.is_some() { conditions.push("status = ?"); }
                if query.date_from.is_some() { conditions.push("scheduled_start_time >= ?"); }
                if query.date_to.is_some() { conditions.push("scheduled_start_time <= ?"); }

                let where_clause = if conditions.is_empty() {
                    String::new()
                } else {
                    format!(" WHERE {}", conditions.join(" AND "))
                };

                let _sql = format!(
                    "SELECT * FROM video_consultations{} ORDER BY scheduled_start_time DESC LIMIT ? OFFSET ?",
                    where_clause
                );

                // For now, just return all consultations if filters are complex
                sqlx::query(
                    "SELECT * FROM video_consultations ORDER BY scheduled_start_time DESC LIMIT ? OFFSET ?"
                )
                .bind(page_size)
                .bind(offset)
                .fetch_all(db)
                .await
            }
        }
        .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        rows.into_iter()
            .map(|row| Self::parse_consultation_row(row))
            .collect::<Result<Vec<_>, _>>()
    }

    // Room Management
    pub async fn join_room(
        db: &DbPool,
        room_id: &str,
        user_id: Uuid,
    ) -> Result<JoinRoomResponse, AppError> {
        let mut tx = db
            .begin()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Get consultation
        let consultation = Self::get_consultation_by_room_id(db, room_id).await?;

        // Check if user is authorized
        // For doctors, we need to check if the user_id corresponds to the doctor_id
        let mut is_doctor = false;
        if let Ok(doctor) = crate::services::doctor_service::get_doctor_by_user_id(db, user_id).await {
            is_doctor = doctor.id == consultation.doctor_id;
        }

        let (role, token) = if is_doctor {
            (
                "doctor",
                Self::generate_token(&consultation.id, &user_id, "doctor"),
            )
        } else if user_id == consultation.patient_id {
            (
                "patient",
                Self::generate_token(&consultation.id, &user_id, "patient"),
            )
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
            .bind(Utc::now())
            .bind(consultation.id.to_string())
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
        )
        .await?;

        // Commit transaction first
        tx.commit()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Get ICE servers configuration (outside transaction)
        let ice_servers = Self::get_ice_servers(db).await?;

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
        user_id: Uuid,
    ) -> Result<(), AppError> {
        let consultation = Self::get_consultation(db, consultation_id).await?;

        // Verify doctor - check if the user_id corresponds to the doctor_id
        let doctor = match crate::services::doctor_service::get_doctor_by_user_id(db, user_id).await {
            Ok(doctor) => doctor,
            Err(_) => return Err(AppError::NotFound("医生信息不存在".to_string())),
        };
        
        if consultation.doctor_id != doctor.id {
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
            .bind(now)
            .bind(now)
            .bind(consultation_id.to_string())
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
            user_id,
        )
        .await?;

        Ok(())
    }

    pub async fn end_consultation(
        db: &DbPool,
        consultation_id: Uuid,
        user_id: Uuid,
        complete_dto: CompleteConsultationDto,
    ) -> Result<(), AppError> {
        let mut tx = db
            .begin()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let consultation = Self::get_consultation(db, consultation_id).await?;

        // Verify doctor - check if the user_id corresponds to the doctor_id
        let doctor = match crate::services::doctor_service::get_doctor_by_user_id(db, user_id).await {
            Ok(doctor) => doctor,
            Err(_) => return Err(AppError::NotFound("医生信息不存在".to_string())),
        };
        
        if consultation.doctor_id != doctor.id {
            return Err(AppError::Forbidden);
        }

        // Check status
        if consultation.status != ConsultationStatus::InProgress {
            return Err(AppError::BadRequest("问诊未开始".to_string()));
        }

        // Calculate duration
        let duration = consultation
            .actual_start_time
            .map(|start| (Utc::now() - start).num_seconds() as i32);

        let now = Utc::now();
        let query = r#"
            UPDATE video_consultations
            SET status = 'completed', end_time = ?, duration = ?,
                diagnosis = ?, treatment_plan = ?, notes = ?, updated_at = ?
            WHERE id = ? AND status = 'in_progress'
        "#;

        sqlx::query(query)
            .bind(now)
            .bind(duration)
            .bind(&complete_dto.diagnosis)
            .bind(&complete_dto.treatment_plan)
            .bind(&complete_dto.notes)
            .bind(now)
            .bind(consultation_id.to_string())
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
            .bind(now)
            .bind(consultation.appointment_id.to_string())
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
            user_id,
        )
        .await?;

        tx.commit()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    pub async fn update_consultation(
        db: &DbPool,
        consultation_id: Uuid,
        user_id: Uuid,
        dto: UpdateConsultationDto,
    ) -> Result<(), AppError> {
        let consultation = Self::get_consultation(db, consultation_id).await?;

        // Verify doctor - check if the user_id corresponds to the doctor_id
        let doctor = match crate::services::doctor_service::get_doctor_by_user_id(db, user_id).await {
            Ok(doctor) => doctor,
            Err(_) => return Err(AppError::NotFound("医生信息不存在".to_string())),
        };
        
        if consultation.doctor_id != doctor.id {
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
            .bind(Utc::now())
            .bind(consultation_id.to_string())
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
            .bind(dto.rating)
            .bind(&dto.feedback)
            .bind(Utc::now())
            .bind(consultation_id.to_string())
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

        // Check if from_user is authorized (doctor or patient)
        let mut is_authorized = false;
        if from_user_id == consultation.patient_id {
            is_authorized = true;
        } else if let Ok(doctor) = crate::services::doctor_service::get_doctor_by_user_id(db, from_user_id).await {
            is_authorized = doctor.id == consultation.doctor_id;
        }
        
        if !is_authorized {
            return Err(AppError::Forbidden);
        }

        // Verify target user is in the room (similar check)
        let mut target_authorized = false;
        if dto.to_user_id == consultation.patient_id {
            target_authorized = true;
        } else if let Ok(doctor) = crate::services::doctor_service::get_doctor_by_user_id(db, dto.to_user_id).await {
            target_authorized = doctor.id == consultation.doctor_id;
        }
        
        if !target_authorized {
            return Err(AppError::BadRequest("目标用户不在房间内".to_string()));
        }

        let signal_id = Uuid::new_v4();
        let query = r#"
            INSERT INTO webrtc_signals (
                id, room_id, from_user_id, to_user_id,
                signal_type, payload, delivered, created_at
            ) VALUES (?, ?, ?, ?, ?, ?, false, ?)
        "#;

        // Convert signal_type enum to string for database
        let signal_type_str = match dto.signal_type {
            SignalType::Offer => "offer",
            SignalType::Answer => "answer",
            SignalType::IceCandidate => "ice_candidate",
            SignalType::Join => "join",
            SignalType::Leave => "leave",
            SignalType::Error => "error",
        };

        sqlx::query(query)
            .bind(signal_id.to_string())
            .bind(&dto.room_id)
            .bind(from_user_id.to_string())
            .bind(dto.to_user_id.to_string())
            .bind(signal_type_str)
            .bind(&dto.payload)
            .bind(Utc::now())
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

        // Check if user is authorized (doctor or patient)
        let mut is_authorized = false;
        if user_id == consultation.patient_id {
            is_authorized = true;
        } else if let Ok(doctor) = crate::services::doctor_service::get_doctor_by_user_id(db, user_id).await {
            is_authorized = doctor.id == consultation.doctor_id;
        }
        
        if !is_authorized {
            return Err(AppError::Forbidden);
        }

        let mut tx = db
            .begin()
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        // Get undelivered signals
        let query = r#"
            SELECT * FROM webrtc_signals
            WHERE room_id = ? AND to_user_id = ? AND delivered = false
            ORDER BY created_at ASC
        "#;

        let rows = sqlx::query(query)
            .bind(room_id)
            .bind(user_id.to_string())
            .fetch_all(&mut *tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        let signals = rows
            .into_iter()
            .map(|row| Self::parse_webrtc_signal_row(row))
            .collect::<Result<Vec<_>, _>>()?;

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
                query_builder = query_builder.bind(id.to_string());
            }

            query_builder
                .execute(&mut *tx)
                .await
                .map_err(|e| AppError::DatabaseError(e.to_string()))?;
        }

        tx.commit()
            .await
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
            .bind(recording_id.to_string())
            .bind(consultation_id.to_string())
            .bind(now)
            .bind(now)
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
            .bind(file_size)
            .bind(duration)
            .bind(Utc::now())
            .bind(recording_id.to_string())
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

        let row = sqlx::query(query)
            .bind(recording_id.to_string())
            .fetch_one(db)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => AppError::NotFound("录制记录不存在".to_string()),
                _ => AppError::DatabaseError(e.to_string()),
            })?;

        Self::parse_recording_row(row)
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

        let rows = sqlx::query(query)
            .bind(consultation_id.to_string())
            .fetch_all(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        rows.into_iter()
            .map(|row| Self::parse_recording_row(row))
            .collect::<Result<Vec<_>, _>>()
    }

    // Template Management
    pub async fn create_template(
        db: &DbPool,
        user_id: Uuid,
        dto: CreateConsultationTemplateDto,
    ) -> Result<VideoConsultationTemplate, AppError> {
        // Get doctor_id from user_id
        let doctor = crate::services::doctor_service::get_doctor_by_user_id(db, user_id)
            .await
            .map_err(|_| AppError::NotFound("医生信息不存在".to_string()))?;
        
        let template_id = Uuid::new_v4();
        let now = Utc::now();

        let query = r#"
            INSERT INTO video_consultation_templates (
                id, doctor_id, name, chief_complaint, diagnosis,
                treatment_plan, notes, usage_count, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, 0, ?, ?)
        "#;

        sqlx::query(query)
            .bind(template_id.to_string())
            .bind(doctor.id.to_string())
            .bind(&dto.name)
            .bind(&dto.chief_complaint)
            .bind(&dto.diagnosis)
            .bind(&dto.treatment_plan)
            .bind(&dto.notes)
            .bind(now)
            .bind(now)
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

        let row = sqlx::query(query)
            .bind(template_id.to_string())
            .fetch_one(db)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => AppError::NotFound("模板不存在".to_string()),
                _ => AppError::DatabaseError(e.to_string()),
            })?;

        Self::parse_template_row(row)
    }

    pub async fn list_doctor_templates(
        db: &DbPool,
        user_id: Uuid,
    ) -> Result<Vec<VideoConsultationTemplate>, AppError> {
        // Get doctor_id from user_id
        let doctor = crate::services::doctor_service::get_doctor_by_user_id(db, user_id)
            .await
            .map_err(|_| AppError::NotFound("医生信息不存在".to_string()))?;
        
        let query = r#"
            SELECT * FROM video_consultation_templates
            WHERE doctor_id = ?
            ORDER BY usage_count DESC, created_at DESC
        "#;

        let rows = sqlx::query(query)
            .bind(doctor.id.to_string())
            .fetch_all(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        rows.into_iter()
            .map(|row| Self::parse_template_row(row))
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn use_template(
        db: &DbPool,
        template_id: Uuid,
        user_id: Uuid,
    ) -> Result<VideoConsultationTemplate, AppError> {
        // Get doctor_id from user_id
        let doctor = crate::services::doctor_service::get_doctor_by_user_id(db, user_id)
            .await
            .map_err(|_| AppError::NotFound("医生信息不存在".to_string()))?;
        
        let template = Self::get_template(db, template_id).await?;

        if template.doctor_id != doctor.id {
            return Err(AppError::Forbidden);
        }

        let query = r#"
            UPDATE video_consultation_templates
            SET usage_count = usage_count + 1, updated_at = ?
            WHERE id = ?
        "#;

        sqlx::query(query)
            .bind(Utc::now())
            .bind(template_id.to_string())
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
        let query = match (doctor_id, start_date, end_date) {
            (Some(doc_id), None, None) => {
                sqlx::query(
                    r#"
                    SELECT
                        COUNT(*) as total_consultations,
                        COUNT(CASE WHEN status = 'completed' THEN 1 END) as completed_consultations,
                        CAST(AVG(CASE WHEN status = 'completed' THEN duration END) AS DOUBLE) as average_duration,
                        CAST(AVG(patient_rating) AS DOUBLE) as average_rating,
                        CAST(COUNT(CASE WHEN status = 'no_show' THEN 1 END) * 100.0 / COUNT(*) AS DOUBLE) as no_show_rate
                    FROM video_consultations
                    WHERE doctor_id = ? AND status != 'cancelled'
                    "#
                )
                .bind(doc_id.to_string())
            },
            (None, None, None) => {
                sqlx::query(
                    r#"
                    SELECT
                        COUNT(*) as total_consultations,
                        COUNT(CASE WHEN status = 'completed' THEN 1 END) as completed_consultations,
                        CAST(AVG(CASE WHEN status = 'completed' THEN duration END) AS DOUBLE) as average_duration,
                        CAST(AVG(patient_rating) AS DOUBLE) as average_rating,
                        CAST(COUNT(CASE WHEN status = 'no_show' THEN 1 END) * 100.0 / COUNT(*) AS DOUBLE) as no_show_rate
                    FROM video_consultations
                    WHERE status != 'cancelled'
                    "#
                )
            },
            _ => {
                // For complex filters, just return simple stats
                sqlx::query(
                    r#"
                    SELECT
                        COUNT(*) as total_consultations,
                        COUNT(CASE WHEN status = 'completed' THEN 1 END) as completed_consultations,
                        CAST(AVG(CASE WHEN status = 'completed' THEN duration END) AS DOUBLE) as average_duration,
                        CAST(AVG(patient_rating) AS DOUBLE) as average_rating,
                        CAST(COUNT(CASE WHEN status = 'no_show' THEN 1 END) * 100.0 / COUNT(*) AS DOUBLE) as no_show_rate
                    FROM video_consultations
                    WHERE status != 'cancelled'
                    "#
                )
            }
        };

        let row = query
            .fetch_one(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        use sqlx::Row;
        Ok(ConsultationStatistics {
            total_consultations: row
                .get::<Option<i64>, _>("total_consultations")
                .unwrap_or(0),
            completed_consultations: row
                .get::<Option<i64>, _>("completed_consultations")
                .unwrap_or(0),
            average_duration: row.get("average_duration"),
            average_rating: row.get("average_rating"),
            no_show_rate: row.get::<Option<f64>, _>("no_show_rate").unwrap_or(0.0),
        })
    }

    // Helper methods
    async fn get_appointment(db: &DbPool, appointment_id: Uuid) -> Result<Appointment, AppError> {
        let query = r#"
            SELECT id, patient_id, doctor_id, appointment_date, time_slot, visit_type, 
                   symptoms, has_visited_before, status, created_at, updated_at
            FROM appointments WHERE id = ?
        "#;

        let row = sqlx::query(query)
            .bind(appointment_id.to_string())
            .fetch_one(db)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => AppError::NotFound("预约不存在".to_string()),
                _ => AppError::DatabaseError(e.to_string()),
            })?;

        use sqlx::Row;
        let visit_type_str: String = row.get("visit_type");
        let visit_type = match visit_type_str.as_str() {
            "online_video" => VisitType::OnlineVideo,
            "offline" => VisitType::Offline,
            _ => return Err(AppError::BadRequest("Invalid visit type".to_string())),
        };

        let status_str: String = row.get("status");
        let status = match status_str.as_str() {
            "pending" => AppointmentStatus::Pending,
            "confirmed" => AppointmentStatus::Confirmed,
            "completed" => AppointmentStatus::Completed,
            "cancelled" => AppointmentStatus::Cancelled,
            _ => {
                return Err(AppError::BadRequest(
                    "Invalid appointment status".to_string(),
                ))
            }
        };

        Ok(Appointment {
            id: Uuid::parse_str(row.get("id"))
                .map_err(|_| AppError::BadRequest("Invalid UUID".to_string()))?,
            patient_id: Uuid::parse_str(row.get("patient_id"))
                .map_err(|_| AppError::BadRequest("Invalid UUID".to_string()))?,
            doctor_id: Uuid::parse_str(row.get("doctor_id"))
                .map_err(|_| AppError::BadRequest("Invalid UUID".to_string()))?,
            appointment_date: row.get("appointment_date"),
            time_slot: row.get("time_slot"),
            visit_type,
            symptoms: row.get("symptoms"),
            has_visited_before: row.get("has_visited_before"),
            status,
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    fn parse_consultation_row(row: sqlx::mysql::MySqlRow) -> Result<VideoConsultation, AppError> {
        use sqlx::Row;

        let status_str: String = row.get("status");
        let status = match status_str.as_str() {
            "waiting" => ConsultationStatus::Waiting,
            "in_progress" => ConsultationStatus::InProgress,
            "completed" => ConsultationStatus::Completed,
            "cancelled" => ConsultationStatus::Cancelled,
            "no_show" => ConsultationStatus::NoShow,
            _ => {
                return Err(AppError::BadRequest(
                    "Invalid consultation status".to_string(),
                ))
            }
        };

        let connection_quality = if let Some(quality_str) = row
            .try_get::<Option<String>, _>("connection_quality")
            .ok()
            .flatten()
        {
            Some(match quality_str.as_str() {
                "excellent" => ConnectionQuality::Excellent,
                "good" => ConnectionQuality::Good,
                "fair" => ConnectionQuality::Fair,
                "poor" => ConnectionQuality::Poor,
                _ => {
                    return Err(AppError::BadRequest(
                        "Invalid connection quality".to_string(),
                    ))
                }
            })
        } else {
            None
        };

        Ok(VideoConsultation {
            id: Uuid::parse_str(row.get("id"))
                .map_err(|_| AppError::BadRequest("Invalid UUID".to_string()))?,
            appointment_id: Uuid::parse_str(row.get("appointment_id"))
                .map_err(|_| AppError::BadRequest("Invalid UUID".to_string()))?,
            doctor_id: Uuid::parse_str(row.get("doctor_id"))
                .map_err(|_| AppError::BadRequest("Invalid UUID".to_string()))?,
            patient_id: Uuid::parse_str(row.get("patient_id"))
                .map_err(|_| AppError::BadRequest("Invalid UUID".to_string()))?,
            room_id: row.get("room_id"),
            status,
            scheduled_start_time: row.get("scheduled_start_time"),
            actual_start_time: row.get("actual_start_time"),
            end_time: row.get("end_time"),
            duration: row.get("duration"),
            doctor_token: row.get("doctor_token"),
            patient_token: row.get("patient_token"),
            ice_servers: row.get("ice_servers"),
            chief_complaint: row.get("chief_complaint"),
            diagnosis: row.get("diagnosis"),
            treatment_plan: row.get("treatment_plan"),
            notes: row.get("notes"),
            connection_quality,
            patient_rating: row.get("patient_rating"),
            patient_feedback: row.get("patient_feedback"),
            metadata: row.get("metadata"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    fn parse_recording_row(row: sqlx::mysql::MySqlRow) -> Result<VideoRecording, AppError> {
        use sqlx::Row;

        let status_str: String = row.get("status");
        let status = match status_str.as_str() {
            "recording" => RecordingStatus::Recording,
            "processing" => RecordingStatus::Processing,
            "completed" => RecordingStatus::Completed,
            "failed" => RecordingStatus::Failed,
            _ => return Err(AppError::BadRequest("Invalid recording status".to_string())),
        };

        Ok(VideoRecording {
            id: Uuid::parse_str(row.get("id"))
                .map_err(|_| AppError::BadRequest("Invalid UUID".to_string()))?,
            consultation_id: Uuid::parse_str(row.get("consultation_id"))
                .map_err(|_| AppError::BadRequest("Invalid UUID".to_string()))?,
            recording_url: row.get("recording_url"),
            thumbnail_url: row.get("thumbnail_url"),
            file_size: row.get("file_size"),
            duration: row.get("duration"),
            format: row.get("format"),
            status,
            error_message: row.get("error_message"),
            started_at: row.get("started_at"),
            completed_at: row.get("completed_at"),
            expires_at: row.get("expires_at"),
            created_at: row.get("created_at"),
        })
    }

    fn parse_webrtc_signal_row(row: sqlx::mysql::MySqlRow) -> Result<WebRTCSignal, AppError> {
        use sqlx::Row;

        let signal_type_str: String = row.get("signal_type");
        let signal_type = match signal_type_str.as_str() {
            "offer" => SignalType::Offer,
            "answer" => SignalType::Answer,
            "ice_candidate" => SignalType::IceCandidate,
            "join" => SignalType::Join,
            "leave" => SignalType::Leave,
            "error" => SignalType::Error,
            _ => return Err(AppError::BadRequest("Invalid signal type".to_string())),
        };

        Ok(WebRTCSignal {
            id: Uuid::parse_str(row.get("id"))
                .map_err(|_| AppError::BadRequest("Invalid UUID".to_string()))?,
            room_id: row.get("room_id"),
            from_user_id: Uuid::parse_str(row.get("from_user_id"))
                .map_err(|_| AppError::BadRequest("Invalid UUID".to_string()))?,
            to_user_id: Uuid::parse_str(row.get("to_user_id"))
                .map_err(|_| AppError::BadRequest("Invalid UUID".to_string()))?,
            signal_type,
            payload: row.get("payload"),
            delivered: row.get("delivered"),
            created_at: row.get("created_at"),
        })
    }

    fn parse_template_row(
        row: sqlx::mysql::MySqlRow,
    ) -> Result<VideoConsultationTemplate, AppError> {
        use sqlx::Row;

        Ok(VideoConsultationTemplate {
            id: Uuid::parse_str(row.get("id"))
                .map_err(|_| AppError::BadRequest("Invalid UUID".to_string()))?,
            doctor_id: Uuid::parse_str(row.get("doctor_id"))
                .map_err(|_| AppError::BadRequest("Invalid UUID".to_string()))?,
            name: row.get("name"),
            chief_complaint: row.get("chief_complaint"),
            diagnosis: row.get("diagnosis"),
            treatment_plan: row.get("treatment_plan"),
            notes: row.get("notes"),
            usage_count: row.get("usage_count"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    async fn log_event(db: &DbPool, dto: LogEventDto, user_id: Uuid) -> Result<(), AppError> {
        let event_id = Uuid::new_v4();
        let query = r#"
            INSERT INTO video_call_events (
                id, consultation_id, user_id, event_type,
                event_data, created_at
            ) VALUES (?, ?, ?, ?, ?, ?)
        "#;

        // Convert event_type enum to string for database
        let event_type_str = match dto.event_type {
            VideoEventType::Joined => "joined",
            VideoEventType::Left => "left",
            VideoEventType::Reconnected => "reconnected",
            VideoEventType::Disconnected => "disconnected",
            VideoEventType::CameraOn => "camera_on",
            VideoEventType::CameraOff => "camera_off",
            VideoEventType::MicOn => "mic_on",
            VideoEventType::MicOff => "mic_off",
            VideoEventType::ScreenShareStart => "screen_share_start",
            VideoEventType::ScreenShareEnd => "screen_share_end",
            VideoEventType::RecordingStart => "recording_start",
            VideoEventType::RecordingEnd => "recording_end",
            VideoEventType::NetworkPoor => "network_poor",
            VideoEventType::NetworkRecovered => "network_recovered",
        };

        sqlx::query(query)
            .bind(event_id.to_string())
            .bind(dto.consultation_id.to_string())
            .bind(user_id.to_string())
            .bind(event_type_str)
            .bind(&dto.event_data)
            .bind(Utc::now())
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

        // Convert event_type enum to string for database
        let event_type_str = match dto.event_type {
            VideoEventType::Joined => "joined",
            VideoEventType::Left => "left",
            VideoEventType::Reconnected => "reconnected",
            VideoEventType::Disconnected => "disconnected",
            VideoEventType::CameraOn => "camera_on",
            VideoEventType::CameraOff => "camera_off",
            VideoEventType::MicOn => "mic_on",
            VideoEventType::MicOff => "mic_off",
            VideoEventType::ScreenShareStart => "screen_share_start",
            VideoEventType::ScreenShareEnd => "screen_share_end",
            VideoEventType::RecordingStart => "recording_start",
            VideoEventType::RecordingEnd => "recording_end",
            VideoEventType::NetworkPoor => "network_poor",
            VideoEventType::NetworkRecovered => "network_recovered",
        };

        sqlx::query(query)
            .bind(event_id.to_string())
            .bind(dto.consultation_id.to_string())
            .bind(user_id.to_string())
            .bind(event_type_str)
            .bind(&dto.event_data)
            .bind(Utc::now())
            .execute(&mut **tx)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(())
    }

    async fn get_ice_servers(_db: &DbPool) -> Result<serde_json::Value, AppError> {
        // For now, return a default ICE server configuration
        // In production, this would be fetched from system_configs table
        let default_ice_servers = r#"[{"urls": ["stun:stun.l.google.com:19302"]}]"#;
        
        serde_json::from_str(default_ice_servers)
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
            .bind(one_hour_ago)
            .execute(db)
            .await
            .map_err(|e| AppError::DatabaseError(e.to_string()))?;

        Ok(result.rows_affected())
    }
}
