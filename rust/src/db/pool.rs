use sqlx::postgres::PgPoolOptions;

pub type Pool = sqlx::Pool<sqlx::Postgres>;

pub async fn create_pool(database_url: &str, pool_size: u32) -> Result<Pool, sqlx::Error> {
    PgPoolOptions::new()
        .max_connections(pool_size)
        .connect(database_url)
        .await
}
