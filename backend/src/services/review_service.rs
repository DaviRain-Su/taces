use crate::config::database::DbPool;
use crate::models::{
    CreateReviewDto, CreateTagDto, DoctorReviewStatistics, PatientReview, RatingDistribution,
    ReplyReviewDto, ReviewDetail, ReviewTag, TagCategory, UpdateReviewDto,
    UpdateReviewVisibilityDto,
};
use anyhow::{anyhow, Result};
use chrono::Utc;
use sqlx::{MySql, Row, Transaction};
use uuid::Uuid;

pub struct ReviewService;

impl ReviewService {
    // ========== 评价相关方法 ==========

    pub async fn create_review(
        pool: &DbPool,
        patient_id: Uuid,
        dto: CreateReviewDto,
    ) -> Result<PatientReview> {
        let mut tx = pool.begin().await?;

        // 验证预约是否存在且属于该患者
        let appointment =
            sqlx::query("SELECT patient_id, doctor_id, status FROM appointments WHERE id = ?")
                .bind(dto.appointment_id.to_string())
                .fetch_optional(&mut *tx)
                .await?
                .ok_or_else(|| anyhow!("Appointment not found"))?;

        let appointment_patient_id: String = appointment.get("patient_id");
        let doctor_id: String = appointment.get("doctor_id");
        let status: String = appointment.get("status");

        if appointment_patient_id != patient_id.to_string() {
            return Err(anyhow!("You can only review your own appointments"));
        }

        if status != "completed" {
            return Err(anyhow!("Can only review completed appointments"));
        }

        // 检查是否已经评价过
        let existing = sqlx::query("SELECT id FROM patient_reviews WHERE appointment_id = ?")
            .bind(dto.appointment_id.to_string())
            .fetch_optional(&mut *tx)
            .await?;

        if existing.is_some() {
            return Err(anyhow!("This appointment has already been reviewed"));
        }

        // 创建评价
        let review_id = Uuid::new_v4();
        let is_anonymous = dto.is_anonymous.unwrap_or(false);

        sqlx::query(
            r#"
            INSERT INTO patient_reviews 
            (id, appointment_id, doctor_id, patient_id, rating, attitude_rating, 
             professionalism_rating, efficiency_rating, comment, is_anonymous)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(review_id.to_string())
        .bind(dto.appointment_id.to_string())
        .bind(&doctor_id)
        .bind(patient_id.to_string())
        .bind(dto.rating)
        .bind(dto.attitude_rating)
        .bind(dto.professionalism_rating)
        .bind(dto.efficiency_rating)
        .bind(&dto.comment)
        .bind(is_anonymous)
        .execute(&mut *tx)
        .await?;

        // 添加标签关联
        if let Some(tag_ids) = dto.tag_ids {
            for tag_id in tag_ids {
                sqlx::query(
                    "INSERT INTO review_tag_relations (id, review_id, tag_id) VALUES (?, ?, ?)",
                )
                .bind(Uuid::new_v4().to_string())
                .bind(review_id.to_string())
                .bind(tag_id.to_string())
                .execute(&mut *tx)
                .await?;

                // 更新标签使用次数
                sqlx::query("UPDATE review_tags SET usage_count = usage_count + 1 WHERE id = ?")
                    .bind(tag_id.to_string())
                    .execute(&mut *tx)
                    .await?;
            }
        }

        // 更新医生评价统计
        Self::update_doctor_statistics(&mut tx, Uuid::parse_str(&doctor_id)?).await?;

        tx.commit().await?;

        Self::get_review_by_id(pool, review_id).await
    }

    pub async fn get_reviews(
        pool: &DbPool,
        doctor_id: Option<Uuid>,
        patient_id: Option<Uuid>,
        rating: Option<i32>,
        has_comment: Option<bool>,
        has_reply: Option<bool>,
        is_anonymous: Option<bool>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<ReviewDetail>, i64)> {
        let offset = (page - 1) * page_size;

        // 构建查询条件
        let mut count_query =
            String::from("SELECT COUNT(*) FROM patient_reviews pr WHERE pr.is_visible = TRUE");
        let mut list_query = String::from(
            r#"
            SELECT pr.*, 
                   d.user_id as doctor_user_id,
                   du.name as doctor_name,
                   p.name as patient_name,
                   a.appointment_date
            FROM patient_reviews pr
            JOIN doctors d ON pr.doctor_id = d.id
            JOIN users du ON d.user_id = du.id
            JOIN users p ON pr.patient_id = p.id
            JOIN appointments a ON pr.appointment_id = a.id
            WHERE pr.is_visible = TRUE
            "#,
        );

        let mut params = vec![];

        if let Some(did) = doctor_id {
            count_query.push_str(" AND pr.doctor_id = ?");
            list_query.push_str(" AND pr.doctor_id = ?");
            params.push(did.to_string());
        }

        if let Some(pid) = patient_id {
            count_query.push_str(" AND pr.patient_id = ?");
            list_query.push_str(" AND pr.patient_id = ?");
            params.push(pid.to_string());
        }

        if let Some(r) = rating {
            count_query.push_str(" AND pr.rating = ?");
            list_query.push_str(" AND pr.rating = ?");
            params.push(r.to_string());
        }

        if let Some(has_c) = has_comment {
            if has_c {
                count_query.push_str(" AND pr.comment IS NOT NULL AND pr.comment != ''");
                list_query.push_str(" AND pr.comment IS NOT NULL AND pr.comment != ''");
            } else {
                count_query.push_str(" AND (pr.comment IS NULL OR pr.comment = '')");
                list_query.push_str(" AND (pr.comment IS NULL OR pr.comment = '')");
            }
        }

        if let Some(has_r) = has_reply {
            if has_r {
                count_query.push_str(" AND pr.reply IS NOT NULL");
                list_query.push_str(" AND pr.reply IS NOT NULL");
            } else {
                count_query.push_str(" AND pr.reply IS NULL");
                list_query.push_str(" AND pr.reply IS NULL");
            }
        }

        if let Some(anon) = is_anonymous {
            count_query.push_str(" AND pr.is_anonymous = ?");
            list_query.push_str(" AND pr.is_anonymous = ?");
            params.push(anon.to_string());
        }

        list_query.push_str(" ORDER BY pr.created_at DESC LIMIT ? OFFSET ?");

        // 获取总数
        let mut count_query_builder = sqlx::query(&count_query);
        for param in &params {
            count_query_builder = count_query_builder.bind(param);
        }
        let total: i64 = count_query_builder.fetch_one(pool).await?.get(0);

        // 获取列表
        let mut list_query_builder = sqlx::query(&list_query);
        for param in params {
            list_query_builder = list_query_builder.bind(param);
        }
        list_query_builder = list_query_builder.bind(page_size).bind(offset);

        let rows = list_query_builder.fetch_all(pool).await?;

        let mut reviews = vec![];
        for row in rows {
            let review_id: String = row.get("id");
            let review_id = Uuid::parse_str(&review_id)?;

            // 获取标签
            let tags = Self::get_review_tags(pool, review_id).await?;

            let detail = Self::parse_review_detail_row(&row, tags)?;
            reviews.push(detail);
        }

        Ok((reviews, total))
    }

    pub async fn get_review_by_id(pool: &DbPool, id: Uuid) -> Result<PatientReview> {
        let row = sqlx::query(
            r#"
            SELECT * FROM patient_reviews WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| anyhow!("Review not found"))?;

        Self::parse_review_row(&row)
    }

    pub async fn get_review_detail(pool: &DbPool, id: Uuid) -> Result<ReviewDetail> {
        let row = sqlx::query(
            r#"
            SELECT pr.*, 
                   d.user_id as doctor_user_id,
                   du.name as doctor_name,
                   p.name as patient_name,
                   a.appointment_date
            FROM patient_reviews pr
            JOIN doctors d ON pr.doctor_id = d.id
            JOIN users du ON d.user_id = du.id
            JOIN users p ON pr.patient_id = p.id
            JOIN appointments a ON pr.appointment_id = a.id
            WHERE pr.id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| anyhow!("Review not found"))?;

        let tags = Self::get_review_tags(pool, id).await?;
        Self::parse_review_detail_row(&row, tags)
    }

    pub async fn update_review(
        pool: &DbPool,
        id: Uuid,
        patient_id: Uuid,
        dto: UpdateReviewDto,
    ) -> Result<PatientReview> {
        let mut tx = pool.begin().await?;

        // 验证所有权
        let review = Self::get_review_by_id(pool, id).await?;
        if review.patient_id != patient_id {
            return Err(anyhow!("You can only update your own reviews"));
        }

        // 只能在24小时内修改
        let hours_since_creation = (Utc::now() - review.created_at).num_hours();
        if hours_since_creation > 24 {
            return Err(anyhow!("Reviews can only be updated within 24 hours"));
        }

        // 构建动态更新查询
        let mut updates = vec![];

        if dto.rating.is_some() {
            updates.push("rating = ?");
        }
        if dto.attitude_rating.is_some() {
            updates.push("attitude_rating = ?");
        }
        if dto.professionalism_rating.is_some() {
            updates.push("professionalism_rating = ?");
        }
        if dto.efficiency_rating.is_some() {
            updates.push("efficiency_rating = ?");
        }
        if dto.comment.is_some() {
            updates.push("comment = ?");
        }
        if dto.is_anonymous.is_some() {
            updates.push("is_anonymous = ?");
        }

        if !updates.is_empty() {
            let query = format!(
                "UPDATE patient_reviews SET {}, updated_at = CURRENT_TIMESTAMP WHERE id = ?",
                updates.join(", ")
            );

            let mut query_builder = sqlx::query(&query);

            if let Some(r) = dto.rating {
                query_builder = query_builder.bind(r);
            }
            if let Some(ar) = dto.attitude_rating {
                query_builder = query_builder.bind(ar);
            }
            if let Some(pr) = dto.professionalism_rating {
                query_builder = query_builder.bind(pr);
            }
            if let Some(er) = dto.efficiency_rating {
                query_builder = query_builder.bind(er);
            }
            if let Some(c) = dto.comment {
                query_builder = query_builder.bind(c);
            }
            if let Some(anon) = dto.is_anonymous {
                query_builder = query_builder.bind(anon);
            }

            query_builder = query_builder.bind(id.to_string());
            query_builder.execute(&mut *tx).await?;
        }

        // 更新标签
        if let Some(tag_ids) = dto.tag_ids {
            // 删除旧标签关联
            sqlx::query("DELETE FROM review_tag_relations WHERE review_id = ?")
                .bind(id.to_string())
                .execute(&mut *tx)
                .await?;

            // 添加新标签关联
            for tag_id in tag_ids {
                sqlx::query(
                    "INSERT INTO review_tag_relations (id, review_id, tag_id) VALUES (?, ?, ?)",
                )
                .bind(Uuid::new_v4().to_string())
                .bind(id.to_string())
                .bind(tag_id.to_string())
                .execute(&mut *tx)
                .await?;
            }
        }

        // 更新医生统计
        Self::update_doctor_statistics(&mut tx, review.doctor_id).await?;

        tx.commit().await?;

        Self::get_review_by_id(pool, id).await
    }

    pub async fn reply_to_review(
        pool: &DbPool,
        id: Uuid,
        doctor_user_id: Uuid,
        dto: ReplyReviewDto,
    ) -> Result<PatientReview> {
        // 验证医生身份
        let review = Self::get_review_by_id(pool, id).await?;

        let doctor_check = sqlx::query("SELECT user_id FROM doctors WHERE id = ?")
            .bind(review.doctor_id.to_string())
            .fetch_one(pool)
            .await?;

        let user_id: String = doctor_check.get::<String, _>("user_id");
        if user_id != doctor_user_id.to_string() {
            return Err(anyhow!("You can only reply to reviews for yourself"));
        }

        // 更新回复
        sqlx::query(
            "UPDATE patient_reviews SET reply = ?, reply_at = CURRENT_TIMESTAMP WHERE id = ?",
        )
        .bind(&dto.reply)
        .bind(id.to_string())
        .execute(pool)
        .await?;

        Self::get_review_by_id(pool, id).await
    }

    pub async fn update_review_visibility(
        pool: &DbPool,
        id: Uuid,
        dto: UpdateReviewVisibilityDto,
    ) -> Result<()> {
        let mut tx = pool.begin().await?;

        // 获取医生ID
        let review = Self::get_review_by_id(pool, id).await?;

        // 更新可见性
        sqlx::query("UPDATE patient_reviews SET is_visible = ? WHERE id = ?")
            .bind(dto.is_visible)
            .bind(id.to_string())
            .execute(&mut *tx)
            .await?;

        // 更新医生统计
        Self::update_doctor_statistics(&mut tx, review.doctor_id).await?;

        tx.commit().await?;

        Ok(())
    }

    // ========== 标签相关方法 ==========

    pub async fn create_tag(pool: &DbPool, dto: CreateTagDto) -> Result<ReviewTag> {
        let category = match dto.category.as_str() {
            "positive" | "negative" => &dto.category,
            _ => return Err(anyhow!("Invalid tag category")),
        };

        let tag_id = Uuid::new_v4();

        sqlx::query("INSERT INTO review_tags (id, name, category) VALUES (?, ?, ?)")
            .bind(tag_id.to_string())
            .bind(&dto.name)
            .bind(category)
            .execute(pool)
            .await?;

        Self::get_tag_by_id(pool, tag_id).await
    }

    pub async fn get_tags(
        pool: &DbPool,
        category: Option<String>,
        is_active: Option<bool>,
    ) -> Result<Vec<ReviewTag>> {
        let mut query = String::from("SELECT * FROM review_tags WHERE 1=1");
        let mut params = vec![];

        if let Some(cat) = category {
            query.push_str(" AND category = ?");
            params.push(cat);
        }

        if let Some(active) = is_active {
            query.push_str(" AND is_active = ?");
            params.push(active.to_string());
        }

        query.push_str(" ORDER BY usage_count DESC, name ASC");

        let mut query_builder = sqlx::query(&query);
        for param in params {
            query_builder = query_builder.bind(param);
        }

        let rows = query_builder.fetch_all(pool).await?;
        rows.into_iter()
            .map(|row| Self::parse_tag_row(&row))
            .collect()
    }

    pub async fn get_tag_by_id(pool: &DbPool, id: Uuid) -> Result<ReviewTag> {
        let row = sqlx::query("SELECT * FROM review_tags WHERE id = ?")
            .bind(id.to_string())
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| anyhow!("Tag not found"))?;

        Self::parse_tag_row(&row)
    }

    // ========== 统计相关方法 ==========

    pub async fn get_doctor_statistics(
        pool: &DbPool,
        doctor_id: Uuid,
    ) -> Result<DoctorReviewStatistics> {
        let row = sqlx::query(
            r#"
            SELECT 
                COUNT(pr.id) as total_reviews,
                COALESCE(AVG(pr.rating), 0) as average_rating,
                COALESCE(AVG(pr.attitude_rating), 0) as average_attitude,
                COALESCE(AVG(pr.professionalism_rating), 0) as average_professionalism,
                COALESCE(AVG(pr.efficiency_rating), 0) as average_efficiency,
                SUM(CASE WHEN pr.rating = 5 THEN 1 ELSE 0 END) as five_star,
                SUM(CASE WHEN pr.rating = 4 THEN 1 ELSE 0 END) as four_star,
                SUM(CASE WHEN pr.rating = 3 THEN 1 ELSE 0 END) as three_star,
                SUM(CASE WHEN pr.rating = 2 THEN 1 ELSE 0 END) as two_star,
                SUM(CASE WHEN pr.rating = 1 THEN 1 ELSE 0 END) as one_star
            FROM patient_reviews pr
            WHERE pr.doctor_id = ? AND pr.is_visible = TRUE
            "#,
        )
        .bind(doctor_id.to_string())
        .fetch_one(pool)
        .await?;

        Ok(DoctorReviewStatistics {
            doctor_id,
            total_reviews: row.get("total_reviews"),
            average_rating: row.get("average_rating"),
            average_attitude: row.get("average_attitude"),
            average_professionalism: row.get("average_professionalism"),
            average_efficiency: row.get("average_efficiency"),
            rating_distribution: RatingDistribution {
                five_star: row.get("five_star"),
                four_star: row.get("four_star"),
                three_star: row.get("three_star"),
                two_star: row.get("two_star"),
                one_star: row.get("one_star"),
            },
        })
    }

    // ========== 辅助方法 ==========

    async fn get_review_tags(pool: &DbPool, review_id: Uuid) -> Result<Vec<ReviewTag>> {
        let rows = sqlx::query(
            r#"
            SELECT rt.* FROM review_tags rt
            JOIN review_tag_relations rtr ON rt.id = rtr.tag_id
            WHERE rtr.review_id = ?
            "#,
        )
        .bind(review_id.to_string())
        .fetch_all(pool)
        .await?;

        rows.into_iter()
            .map(|row| Self::parse_tag_row(&row))
            .collect()
    }

    async fn update_doctor_statistics(
        tx: &mut Transaction<'_, MySql>,
        doctor_id: Uuid,
    ) -> Result<()> {
        let stats = sqlx::query(
            r#"
            SELECT 
                COUNT(pr.id) as total,
                COALESCE(AVG(pr.rating), 0) as avg_rating,
                COALESCE(AVG(pr.attitude_rating), 0) as avg_attitude,
                COALESCE(AVG(pr.professionalism_rating), 0) as avg_professionalism,
                COALESCE(AVG(pr.efficiency_rating), 0) as avg_efficiency
            FROM patient_reviews pr
            WHERE pr.doctor_id = ? AND pr.is_visible = TRUE
            "#,
        )
        .bind(doctor_id.to_string())
        .fetch_one(&mut **tx)
        .await?;

        sqlx::query(
            r#"
            UPDATE doctors 
            SET total_reviews = ?,
                average_rating = ?,
                average_attitude = ?,
                average_professionalism = ?,
                average_efficiency = ?
            WHERE id = ?
            "#,
        )
        .bind(stats.try_get::<i64, _>("total")?)
        .bind(stats.try_get::<f64, _>("avg_rating")?)
        .bind(stats.try_get::<f64, _>("avg_attitude")?)
        .bind(stats.try_get::<f64, _>("avg_professionalism")?)
        .bind(stats.try_get::<f64, _>("avg_efficiency")?)
        .bind(doctor_id.to_string())
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    fn parse_review_row(row: &sqlx::mysql::MySqlRow) -> Result<PatientReview> {
        let id_str: String = row.get("id");
        let appointment_id_str: String = row.get("appointment_id");
        let doctor_id_str: String = row.get("doctor_id");
        let patient_id_str: String = row.get("patient_id");

        Ok(PatientReview {
            id: Uuid::parse_str(&id_str)?,
            appointment_id: Uuid::parse_str(&appointment_id_str)?,
            doctor_id: Uuid::parse_str(&doctor_id_str)?,
            patient_id: Uuid::parse_str(&patient_id_str)?,
            rating: row.get("rating"),
            attitude_rating: row.get("attitude_rating"),
            professionalism_rating: row.get("professionalism_rating"),
            efficiency_rating: row.get("efficiency_rating"),
            comment: row.get("comment"),
            reply: row.get("reply"),
            reply_at: row.get("reply_at"),
            is_anonymous: row.get("is_anonymous"),
            is_visible: row.get("is_visible"),
            created_at: row.get("created_at"),
            updated_at: row.get("updated_at"),
        })
    }

    fn parse_review_detail_row(
        row: &sqlx::mysql::MySqlRow,
        tags: Vec<ReviewTag>,
    ) -> Result<ReviewDetail> {
        let id_str: String = row.get("id");
        let appointment_id_str: String = row.get("appointment_id");
        let doctor_id_str: String = row.get("doctor_id");
        let patient_id_str: String = row.get("patient_id");
        let is_anonymous: bool = row.get("is_anonymous");

        Ok(ReviewDetail {
            id: Uuid::parse_str(&id_str)?,
            appointment_id: Uuid::parse_str(&appointment_id_str)?,
            doctor_id: Uuid::parse_str(&doctor_id_str)?,
            doctor_name: row.get("doctor_name"),
            patient_id: Uuid::parse_str(&patient_id_str)?,
            patient_name: if is_anonymous {
                "匿名用户".to_string()
            } else {
                row.get("patient_name")
            },
            rating: row.get("rating"),
            attitude_rating: row.get("attitude_rating"),
            professionalism_rating: row.get("professionalism_rating"),
            efficiency_rating: row.get("efficiency_rating"),
            comment: row.get("comment"),
            reply: row.get("reply"),
            reply_at: row.get("reply_at"),
            is_anonymous,
            tags,
            appointment_date: row.get("appointment_date"),
            created_at: row.get("created_at"),
        })
    }

    fn parse_tag_row(row: &sqlx::mysql::MySqlRow) -> Result<ReviewTag> {
        let id_str: String = row.get("id");
        let category_str: String = row.get("category");

        Ok(ReviewTag {
            id: Uuid::parse_str(&id_str)?,
            name: row.get("name"),
            category: match category_str.as_str() {
                "positive" => TagCategory::Positive,
                "negative" => TagCategory::Negative,
                _ => return Err(anyhow!("Invalid tag category")),
            },
            usage_count: row.get("usage_count"),
            is_active: row.get("is_active"),
        })
    }
}
