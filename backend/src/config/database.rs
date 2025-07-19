use sqlx::{MySql, MySqlPool, Pool};
use std::env;

pub type DbPool = Pool<MySql>;

pub async fn create_pool() -> Result<DbPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    MySqlPool::connect(&database_url).await
}

pub async fn run_migrations(pool: &DbPool) -> Result<(), sqlx::migrate::MigrateError> {
    sqlx::migrate!("./migrations").run(pool).await
}
