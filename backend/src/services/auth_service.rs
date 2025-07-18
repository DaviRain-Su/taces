use anyhow::{anyhow, Result};
use uuid::Uuid;
use chrono::Utc;
use sqlx::MySqlPool;
use crate::{
    config::{Config, database::DbPool},
    models::user::*,
    utils::{
        jwt::create_token,
        password::{hash_password, verify_password},
    },
};

pub async fn register_user(pool: &DbPool, dto: CreateUserDto) -> Result<User> {
    let hashed_password = hash_password(&dto.password)?;
    
    let user_id = Uuid::new_v4();
    let now = Utc::now();
    
    let query = r#"
        INSERT INTO users (id, account, name, password, gender, phone, email, birthday, role, status, created_at, updated_at)
        VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 'active', ?, ?)
    "#;
    
    sqlx::query(query)
        .bind(user_id.to_string())
        .bind(&dto.account)
        .bind(&dto.name)
        .bind(&hashed_password)
        .bind(&dto.gender)
        .bind(&dto.phone)
        .bind(&dto.email)
        .bind(dto.birthday)
        .bind(match dto.role {
            UserRole::Admin => "admin",
            UserRole::Doctor => "doctor",
            UserRole::Patient => "patient",
        })
        .bind(now)
        .bind(now)
        .execute(pool)
        .await
        .map_err(|e| anyhow!("Failed to create user: {}", e))?;
    
    get_user_by_id(pool, user_id).await
}

pub async fn login(pool: &DbPool, config: &Config, dto: LoginDto) -> Result<LoginResponse> {
    let user = get_user_by_account(pool, &dto.account).await?;
    
    if !verify_password(&dto.password, &user.password)? {
        return Err(anyhow!("Invalid credentials"));
    }
    
    if matches!(user.status, UserStatus::Inactive) {
        return Err(anyhow!("Account is inactive"));
    }
    
    let role_str = match user.role {
        UserRole::Admin => "admin",
        UserRole::Doctor => "doctor",
        UserRole::Patient => "patient",
    };
    
    let token = create_token(user.id, role_str.to_string(), &config.jwt_secret, config.jwt_expiration)?;
    
    Ok(LoginResponse { token, user })
}

async fn get_user_by_id(pool: &DbPool, id: Uuid) -> Result<User> {
    let query = r#"
        SELECT id, account, name, password, gender, phone, email, birthday, role, status, created_at, updated_at
        FROM users
        WHERE id = ?
    "#;
    
    let user = sqlx::query_as::<_, User>(query)
        .bind(id.to_string())
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow!("User not found: {}", e))?;
    
    Ok(user)
}

async fn get_user_by_account(pool: &DbPool, account: &str) -> Result<User> {
    let query = r#"
        SELECT id, account, name, password, gender, phone, email, birthday, role, status, created_at, updated_at
        FROM users
        WHERE account = ?
    "#;
    
    let user = sqlx::query_as::<_, User>(query)
        .bind(account)
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow!("User not found: {}", e))?;
    
    Ok(user)
}