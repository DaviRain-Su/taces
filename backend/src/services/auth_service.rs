use crate::{
    config::{database::DbPool, Config},
    models::user::*,
    utils::{
        jwt::create_token,
        password::{hash_password, verify_password},
    },
};
use anyhow::{anyhow, Result};
use chrono::Utc;
use uuid::Uuid;

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

    let token = create_token(
        user.id,
        role_str.to_string(),
        &config.jwt_secret,
        config.jwt_expiration,
    )?;

    Ok(LoginResponse { token, user })
}

async fn get_user_by_id(pool: &DbPool, id: Uuid) -> Result<User> {
    let query = r#"
        SELECT id, account, name, password, gender, phone, email, birthday, role, status, created_at, updated_at
        FROM users
        WHERE id = ?
    "#;

    let row = sqlx::query(query)
        .bind(id.to_string())
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow!("User not found: {}", e))?;

    Ok(User {
        id: Uuid::parse_str(sqlx::Row::get(&row, "id")).unwrap(),
        account: sqlx::Row::get(&row, "account"),
        name: sqlx::Row::get(&row, "name"),
        password: sqlx::Row::get(&row, "password"),
        gender: sqlx::Row::get(&row, "gender"),
        phone: sqlx::Row::get(&row, "phone"),
        email: sqlx::Row::get(&row, "email"),
        birthday: sqlx::Row::get(&row, "birthday"),
        role: match sqlx::Row::get::<String, _>(&row, "role").as_str() {
            "admin" => UserRole::Admin,
            "doctor" => UserRole::Doctor,
            "patient" => UserRole::Patient,
            _ => return Err(anyhow!("Invalid user role")),
        },
        status: match sqlx::Row::get::<String, _>(&row, "status").as_str() {
            "active" => UserStatus::Active,
            "inactive" => UserStatus::Inactive,
            _ => return Err(anyhow!("Invalid user status")),
        },
        created_at: sqlx::Row::get(&row, "created_at"),
        updated_at: sqlx::Row::get(&row, "updated_at"),
    })
}

async fn get_user_by_account(pool: &DbPool, account: &str) -> Result<User> {
    let query = r#"
        SELECT id, account, name, password, gender, phone, email, birthday, role, status, created_at, updated_at
        FROM users
        WHERE account = ?
    "#;

    let row = sqlx::query(query)
        .bind(account)
        .fetch_one(pool)
        .await
        .map_err(|e| anyhow!("User not found: {}", e))?;

    Ok(User {
        id: Uuid::parse_str(sqlx::Row::get(&row, "id")).unwrap(),
        account: sqlx::Row::get(&row, "account"),
        name: sqlx::Row::get(&row, "name"),
        password: sqlx::Row::get(&row, "password"),
        gender: sqlx::Row::get(&row, "gender"),
        phone: sqlx::Row::get(&row, "phone"),
        email: sqlx::Row::get(&row, "email"),
        birthday: sqlx::Row::get(&row, "birthday"),
        role: match sqlx::Row::get::<String, _>(&row, "role").as_str() {
            "admin" => UserRole::Admin,
            "doctor" => UserRole::Doctor,
            "patient" => UserRole::Patient,
            _ => return Err(anyhow!("Invalid user role")),
        },
        status: match sqlx::Row::get::<String, _>(&row, "status").as_str() {
            "active" => UserStatus::Active,
            "inactive" => UserStatus::Inactive,
            _ => return Err(anyhow!("Invalid user status")),
        },
        created_at: sqlx::Row::get(&row, "created_at"),
        updated_at: sqlx::Row::get(&row, "updated_at"),
    })
}
