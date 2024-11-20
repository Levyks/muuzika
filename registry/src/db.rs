use anyhow::Context;
use sqlx::migrate::MigrateDatabase;
use sqlx::sqlite::SqlitePoolOptions;
use sqlx::{Pool, Sqlite};

pub async fn init_db() -> anyhow::Result<Pool<Sqlite>> {
    let sqlite_url = std::env::var("SQLITE_URL").unwrap_or_else(|_| ":memory:".to_string());

    let db_url = format!("sqlite:{}", sqlite_url);

    if sqlite_url != ":memory:" && Sqlite::database_exists(&db_url).await? {
        log::info!("Database already exists, dropping it");
        Sqlite::drop_database(&db_url).await?;
    }

    log::info!("Creating database {}", db_url);
    Sqlite::create_database(&db_url).await?;

    let max_connections = std::env::var("MAX_CONNECTIONS")
        .unwrap_or_else(|_| "5".to_string())
        .parse::<u32>()
        .context("Failed to parse env variable MAX_CONNECTIONS")?;

    let pool = SqlitePoolOptions::new()
        .max_connections(max_connections)
        .connect(&db_url)
        .await?;

    log::info!("Running migrations");
    sqlx::migrate!().run(&pool).await?;

    log::info!("Database initialized");

    Ok(pool)
}

#[derive(Debug, sqlx::FromRow)]
pub struct Server {
    pub id: i64,
    pub address: String,
    pub capacity: Option<i64>,
    pub callsign: Option<String>,
    pub description: Option<String>,
}
