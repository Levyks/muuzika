use diesel_async::pooled_connection::AsyncDieselConnectionManager;
use diesel_async::AsyncPgConnection;
use diesel_async::pooled_connection::deadpool;

pub type Pool = deadpool::Pool<AsyncPgConnection>;

pub fn create_pool(database_url: &str, pool_size: usize) -> Result<Pool, deadpool::BuildError> {
    let manager = AsyncDieselConnectionManager::<AsyncPgConnection>::new(database_url);

    let pool: Pool = Pool::builder(manager).max_size(pool_size).build()?;

    Ok(pool)
}