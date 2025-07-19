use crate::config::database::DbPool;
use crate::models::{
    Circle, CircleListItem, CircleMemberInfo, CircleWithMemberInfo,
    CreateCircleDto, MemberRole, UpdateCircleDto, UpdateMemberRoleDto,
};
use anyhow::{anyhow, Result};
use sqlx::{MySql, Row, Transaction};
use uuid::Uuid;

pub struct CircleService;

impl CircleService {
    pub async fn create_circle(
        pool: &DbPool,
        creator_id: Uuid,
        dto: CreateCircleDto,
    ) -> Result<Circle> {
        let mut tx = pool.begin().await?;

        // Create the circle
        let circle_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO circles (id, name, description, avatar, category, creator_id, member_count)
            VALUES (?, ?, ?, ?, ?, ?, 1)
            "#,
        )
        .bind(circle_id.to_string())
        .bind(&dto.name)
        .bind(&dto.description)
        .bind(&dto.avatar)
        .bind(&dto.category)
        .bind(creator_id.to_string())
        .execute(&mut *tx)
        .await?;

        // Add creator as owner
        let member_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO circle_members (id, circle_id, user_id, role)
            VALUES (?, ?, ?, 'owner')
            "#,
        )
        .bind(member_id.to_string())
        .bind(circle_id.to_string())
        .bind(creator_id.to_string())
        .execute(&mut *tx)
        .await?;

        // Fetch the created circle
        let circle = sqlx::query(
            r#"
            SELECT id, name, description, avatar, category, creator_id, 
                   member_count, post_count, is_active, created_at, updated_at
            FROM circles
            WHERE id = ?
            "#,
        )
        .bind(circle_id.to_string())
        .fetch_one(&mut *tx)
        .await?;

        let circle = parse_circle_row(&circle)?;

        tx.commit().await?;

        Ok(circle)
    }

    pub async fn get_circles(
        pool: &DbPool,
        user_id: Option<Uuid>,
        category: Option<String>,
        keyword: Option<String>,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<CircleListItem>, i64)> {
        let offset = (page - 1) * page_size;

        // Build the query with filters
        let mut count_query = String::from("SELECT COUNT(*) FROM circles WHERE is_active = TRUE");
        let mut list_query = String::from(
            r#"
            SELECT c.id, c.name, c.description, c.avatar, c.category, 
                   c.member_count, c.post_count,
                   CASE WHEN cm.id IS NOT NULL THEN TRUE ELSE FALSE END as is_joined
            FROM circles c
            LEFT JOIN circle_members cm ON c.id = cm.circle_id AND cm.user_id = ?
            WHERE c.is_active = TRUE
            "#,
        );

        let mut params = vec![];
        let mut count_params = vec![];

        if let Some(ref cat) = category {
            count_query.push_str(" AND category = ?");
            list_query.push_str(" AND c.category = ?");
            count_params.push(cat.clone());
            params.push(cat.clone());
        }

        if let Some(ref kw) = keyword {
            count_query.push_str(" AND (name LIKE ? OR description LIKE ?)");
            list_query.push_str(" AND (c.name LIKE ? OR c.description LIKE ?)");
            let like_pattern = format!("%{}%", kw);
            count_params.push(like_pattern.clone());
            count_params.push(like_pattern.clone());
            params.push(like_pattern.clone());
            params.push(like_pattern);
        }

        list_query.push_str(" ORDER BY c.member_count DESC, c.created_at DESC LIMIT ? OFFSET ?");

        // Get total count
        let mut count_query_builder = sqlx::query(&count_query);
        for param in count_params {
            count_query_builder = count_query_builder.bind(param);
        }
        let total: i64 = count_query_builder
            .fetch_one(pool)
            .await?
            .try_get(0)?;

        // Get circles list
        let mut list_query_builder = sqlx::query(&list_query)
            .bind(user_id.unwrap_or(Uuid::nil()).to_string());
        for param in params {
            list_query_builder = list_query_builder.bind(param);
        }
        list_query_builder = list_query_builder.bind(page_size).bind(offset);

        let rows = list_query_builder.fetch_all(pool).await?;

        let circles = rows
            .into_iter()
            .map(|row| {
                let id_str: String = row.try_get("id")?;
                Ok(CircleListItem {
                    id: Uuid::parse_str(&id_str).map_err(|e| anyhow!("Invalid UUID: {}", e))?,
                    name: row.try_get("name")?,
                    description: row.try_get("description")?,
                    avatar: row.try_get("avatar")?,
                    category: row.try_get("category")?,
                    member_count: row.try_get("member_count")?,
                    post_count: row.try_get("post_count")?,
                    is_joined: row.try_get("is_joined")?,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok((circles, total))
    }

    pub async fn get_circle_by_id(
        pool: &DbPool,
        id: Uuid,
        user_id: Option<Uuid>,
    ) -> Result<CircleWithMemberInfo> {
        let row = sqlx::query(
            r#"
            SELECT c.id, c.name, c.description, c.avatar, c.category, c.creator_id,
                   c.member_count, c.post_count, c.is_active, c.created_at, c.updated_at,
                   cm.id as member_id, cm.role as member_role
            FROM circles c
            LEFT JOIN circle_members cm ON c.id = cm.circle_id AND cm.user_id = ?
            WHERE c.id = ?
            "#,
        )
        .bind(user_id.unwrap_or(Uuid::nil()).to_string())
        .bind(id.to_string())
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| anyhow!("Circle not found"))?;

        let id_str: String = row.try_get("id")?;
        let creator_id_str: String = row.try_get("creator_id")?;

        let circle = Circle {
            id: Uuid::parse_str(&id_str)?,
            name: row.try_get("name")?,
            description: row.try_get("description")?,
            avatar: row.try_get("avatar")?,
            category: row.try_get("category")?,
            creator_id: Uuid::parse_str(&creator_id_str)?,
            member_count: row.try_get("member_count")?,
            post_count: row.try_get("post_count")?,
            is_active: row.try_get("is_active")?,
            created_at: row.try_get("created_at")?,
            updated_at: row.try_get("updated_at")?,
        };

        let member_id: Option<String> = row.try_get("member_id")?;
        let is_joined = member_id.is_some();
        let member_role = if is_joined {
            let role_str: String = row.try_get("member_role")?;
            Some(parse_member_role(&role_str)?)
        } else {
            None
        };

        Ok(CircleWithMemberInfo {
            circle,
            is_joined,
            member_role,
        })
    }

    pub async fn update_circle(
        pool: &DbPool,
        id: Uuid,
        user_id: Uuid,
        is_admin: bool,
        dto: UpdateCircleDto,
    ) -> Result<Circle> {
        // Check if user is owner or admin
        if !is_admin {
            let is_owner = Self::is_circle_owner(pool, id, user_id).await?;
            if !is_owner {
                return Err(anyhow!("Only circle owner or admin can update"));
            }
        }

        // Build dynamic query
        let mut query = String::from("UPDATE circles SET ");
        let mut first = true;
        
        if dto.name.is_some() {
            if !first { query.push_str(", "); }
            query.push_str("name = ?");
            first = false;
        }
        
        if dto.description.is_some() {
            if !first { query.push_str(", "); }
            query.push_str("description = ?");
            first = false;
        }
        
        if dto.avatar.is_some() {
            if !first { query.push_str(", "); }
            query.push_str("avatar = ?");
            first = false;
        }
        
        if dto.is_active.is_some() {
            if !first { query.push_str(", "); }
            query.push_str("is_active = ?");
            first = false;
        }
        
        if first {
            return Err(anyhow!("No fields to update"));
        }
        
        query.push_str(", updated_at = CURRENT_TIMESTAMP WHERE id = ?");
        
        // Bind parameters
        let mut query_builder = sqlx::query(&query);
        
        if let Some(name) = dto.name {
            query_builder = query_builder.bind(name);
        }
        
        if let Some(description) = dto.description {
            query_builder = query_builder.bind(description);
        }
        
        if let Some(avatar) = dto.avatar {
            query_builder = query_builder.bind(avatar);
        }
        
        if let Some(is_active) = dto.is_active {
            query_builder = query_builder.bind(is_active);
        }
        
        query_builder = query_builder.bind(id.to_string());
        
        query_builder.execute(pool).await?;

        Self::get_circle_simple(pool, id).await
    }

    pub async fn delete_circle(
        pool: &DbPool,
        id: Uuid,
        user_id: Uuid,
        is_admin: bool,
    ) -> Result<()> {
        // Check if user is owner or admin
        if !is_admin {
            let is_owner = Self::is_circle_owner(pool, id, user_id).await?;
            if !is_owner {
                return Err(anyhow!("Only circle owner or admin can delete"));
            }
        }

        // Soft delete
        sqlx::query("UPDATE circles SET is_active = FALSE WHERE id = ?")
            .bind(id.to_string())
            .execute(pool)
            .await?;

        Ok(())
    }

    pub async fn join_circle(pool: &DbPool, circle_id: Uuid, user_id: Uuid) -> Result<()> {
        // Check if already joined
        let existing = sqlx::query(
            "SELECT id FROM circle_members WHERE circle_id = ? AND user_id = ?"
        )
        .bind(circle_id.to_string())
        .bind(user_id.to_string())
        .fetch_optional(pool)
        .await?;

        if existing.is_some() {
            return Err(anyhow!("Already joined this circle"));
        }

        let mut tx = pool.begin().await?;

        // Add member
        let member_id = Uuid::new_v4();
        sqlx::query(
            r#"
            INSERT INTO circle_members (id, circle_id, user_id, role)
            VALUES (?, ?, ?, 'member')
            "#,
        )
        .bind(member_id.to_string())
        .bind(circle_id.to_string())
        .bind(user_id.to_string())
        .execute(&mut *tx)
        .await?;

        // Update member count
        sqlx::query("UPDATE circles SET member_count = member_count + 1 WHERE id = ?")
            .bind(circle_id.to_string())
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn leave_circle(pool: &DbPool, circle_id: Uuid, user_id: Uuid) -> Result<()> {
        // Check if user is owner
        let member = sqlx::query(
            "SELECT role FROM circle_members WHERE circle_id = ? AND user_id = ?"
        )
        .bind(circle_id.to_string())
        .bind(user_id.to_string())
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| anyhow!("Not a member of this circle"))?;

        let role: String = member.try_get("role")?;
        if role == "owner" {
            return Err(anyhow!("Owner cannot leave the circle"));
        }

        let mut tx = pool.begin().await?;

        // Remove member
        sqlx::query("DELETE FROM circle_members WHERE circle_id = ? AND user_id = ?")
            .bind(circle_id.to_string())
            .bind(user_id.to_string())
            .execute(&mut *tx)
            .await?;

        // Update member count
        sqlx::query("UPDATE circles SET member_count = member_count - 1 WHERE id = ?")
            .bind(circle_id.to_string())
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn get_circle_members(
        pool: &DbPool,
        circle_id: Uuid,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<CircleMemberInfo>, i64)> {
        let offset = (page - 1) * page_size;

        // Get total count
        let total: i64 = sqlx::query("SELECT COUNT(*) FROM circle_members WHERE circle_id = ?")
            .bind(circle_id.to_string())
            .fetch_one(pool)
            .await?
            .try_get(0)?;

        // Get members
        let rows = sqlx::query(
            r#"
            SELECT cm.id, cm.user_id, cm.role, cm.joined_at,
                   u.name as user_name
            FROM circle_members cm
            JOIN users u ON cm.user_id = u.id
            WHERE cm.circle_id = ?
            ORDER BY 
                CASE cm.role 
                    WHEN 'owner' THEN 1
                    WHEN 'admin' THEN 2
                    ELSE 3
                END,
                cm.joined_at
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(circle_id.to_string())
        .bind(page_size)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let members = rows
            .into_iter()
            .map(|row| {
                let id_str: String = row.try_get("id")?;
                let user_id_str: String = row.try_get("user_id")?;
                let role_str: String = row.try_get("role")?;
                Ok(CircleMemberInfo {
                    id: Uuid::parse_str(&id_str)?,
                    user_id: Uuid::parse_str(&user_id_str)?,
                    user_name: row.try_get("user_name")?,
                    user_avatar: None,
                    role: parse_member_role(&role_str)?,
                    joined_at: row.try_get("joined_at")?,
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok((members, total))
    }

    pub async fn update_member_role(
        pool: &DbPool,
        circle_id: Uuid,
        member_user_id: Uuid,
        operator_id: Uuid,
        is_admin: bool,
        dto: UpdateMemberRoleDto,
    ) -> Result<()> {
        // Check if operator has permission
        if !is_admin {
            let operator_role = Self::get_member_role(pool, circle_id, operator_id).await?;
            match operator_role {
                MemberRole::Owner => {}, // Owner can update any role
                MemberRole::Admin => {
                    // Admin can only update members
                    if matches!(dto.role, MemberRole::Owner | MemberRole::Admin) {
                        return Err(anyhow!("Admin can only set member role"));
                    }
                },
                MemberRole::Member => {
                    return Err(anyhow!("No permission to update member roles"));
                }
            }
        }

        // Cannot change owner role
        let current_role = Self::get_member_role(pool, circle_id, member_user_id).await?;
        if matches!(current_role, MemberRole::Owner) {
            return Err(anyhow!("Cannot change owner role"));
        }

        let role_str = match dto.role {
            MemberRole::Owner => "owner",
            MemberRole::Admin => "admin",
            MemberRole::Member => "member",
        };

        sqlx::query(
            "UPDATE circle_members SET role = ? WHERE circle_id = ? AND user_id = ?"
        )
        .bind(role_str)
        .bind(circle_id.to_string())
        .bind(member_user_id.to_string())
        .execute(pool)
        .await?;

        Ok(())
    }

    pub async fn remove_member(
        pool: &DbPool,
        circle_id: Uuid,
        member_user_id: Uuid,
        operator_id: Uuid,
        is_admin: bool,
    ) -> Result<()> {
        // Check if operator has permission
        if !is_admin {
            let operator_role = Self::get_member_role(pool, circle_id, operator_id).await?;
            match operator_role {
                MemberRole::Owner | MemberRole::Admin => {}, // Can remove members
                MemberRole::Member => {
                    return Err(anyhow!("No permission to remove members"));
                }
            }
        }

        // Cannot remove owner
        let member_role = Self::get_member_role(pool, circle_id, member_user_id).await?;
        if matches!(member_role, MemberRole::Owner) {
            return Err(anyhow!("Cannot remove owner"));
        }

        let mut tx = pool.begin().await?;

        // Remove member
        sqlx::query("DELETE FROM circle_members WHERE circle_id = ? AND user_id = ?")
            .bind(circle_id.to_string())
            .bind(member_user_id.to_string())
            .execute(&mut *tx)
            .await?;

        // Update member count
        sqlx::query("UPDATE circles SET member_count = member_count - 1 WHERE id = ?")
            .bind(circle_id.to_string())
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(())
    }

    pub async fn get_user_circles(
        pool: &DbPool,
        user_id: Uuid,
        page: i64,
        page_size: i64,
    ) -> Result<(Vec<CircleListItem>, i64)> {
        let offset = (page - 1) * page_size;

        // Get total count
        let total: i64 = sqlx::query(
            r#"
            SELECT COUNT(*) 
            FROM circle_members cm
            JOIN circles c ON cm.circle_id = c.id
            WHERE cm.user_id = ? AND c.is_active = TRUE
            "#
        )
        .bind(user_id.to_string())
        .fetch_one(pool)
        .await?
        .try_get(0)?;

        // Get circles
        let rows = sqlx::query(
            r#"
            SELECT c.id, c.name, c.description, c.avatar, c.category,
                   c.member_count, c.post_count
            FROM circle_members cm
            JOIN circles c ON cm.circle_id = c.id
            WHERE cm.user_id = ? AND c.is_active = TRUE
            ORDER BY cm.joined_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(user_id.to_string())
        .bind(page_size)
        .bind(offset)
        .fetch_all(pool)
        .await?;

        let circles = rows
            .into_iter()
            .map(|row| {
                let id_str: String = row.try_get("id")?;
                Ok(CircleListItem {
                    id: Uuid::parse_str(&id_str)?,
                    name: row.try_get("name")?,
                    description: row.try_get("description")?,
                    avatar: row.try_get("avatar")?,
                    category: row.try_get("category")?,
                    member_count: row.try_get("member_count")?,
                    post_count: row.try_get("post_count")?,
                    is_joined: true, // User is already a member
                })
            })
            .collect::<Result<Vec<_>>>()?;

        Ok((circles, total))
    }

    // Helper methods
    async fn is_circle_owner(pool: &DbPool, circle_id: Uuid, user_id: Uuid) -> Result<bool> {
        let result = sqlx::query(
            "SELECT role FROM circle_members WHERE circle_id = ? AND user_id = ?"
        )
        .bind(circle_id.to_string())
        .bind(user_id.to_string())
        .fetch_optional(pool)
        .await?;

        if let Some(row) = result {
            let role: String = row.try_get("role")?;
            Ok(role == "owner")
        } else {
            Ok(false)
        }
    }

    async fn get_member_role(
        pool: &DbPool,
        circle_id: Uuid,
        user_id: Uuid,
    ) -> Result<MemberRole> {
        let row = sqlx::query(
            "SELECT role FROM circle_members WHERE circle_id = ? AND user_id = ?"
        )
        .bind(circle_id.to_string())
        .bind(user_id.to_string())
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| anyhow!("Not a member of this circle"))?;

        let role_str: String = row.try_get("role")?;
        parse_member_role(&role_str)
    }

    async fn get_circle_simple(pool: &DbPool, id: Uuid) -> Result<Circle> {
        let row = sqlx::query(
            r#"
            SELECT id, name, description, avatar, category, creator_id,
                   member_count, post_count, is_active, created_at, updated_at
            FROM circles
            WHERE id = ?
            "#,
        )
        .bind(id.to_string())
        .fetch_one(pool)
        .await?;

        parse_circle_row(&row)
    }

    pub async fn update_post_count(
        tx: &mut Transaction<'_, MySql>,
        circle_id: Uuid,
        increment: i32,
    ) -> Result<()> {
        if increment > 0 {
            sqlx::query("UPDATE circles SET post_count = post_count + ? WHERE id = ?")
                .bind(increment)
                .bind(circle_id.to_string())
                .execute(&mut **tx)
                .await?;
        } else {
            sqlx::query("UPDATE circles SET post_count = post_count - ? WHERE id = ? AND post_count > 0")
                .bind(-increment)
                .bind(circle_id.to_string())
                .execute(&mut **tx)
                .await?;
        }
        Ok(())
    }
}

fn parse_circle_row(row: &sqlx::mysql::MySqlRow) -> Result<Circle> {
    let id_str: String = row.try_get("id")?;
    let creator_id_str: String = row.try_get("creator_id")?;
    
    Ok(Circle {
        id: Uuid::parse_str(&id_str)?,
        name: row.try_get("name")?,
        description: row.try_get("description")?,
        avatar: row.try_get("avatar")?,
        category: row.try_get("category")?,
        creator_id: Uuid::parse_str(&creator_id_str)?,
        member_count: row.try_get("member_count")?,
        post_count: row.try_get("post_count")?,
        is_active: row.try_get("is_active")?,
        created_at: row.try_get("created_at")?,
        updated_at: row.try_get("updated_at")?,
    })
}

fn parse_member_role(role_str: &str) -> Result<MemberRole> {
    match role_str {
        "owner" => Ok(MemberRole::Owner),
        "admin" => Ok(MemberRole::Admin),
        "member" => Ok(MemberRole::Member),
        _ => Err(anyhow!("Invalid member role: {}", role_str)),
    }
}