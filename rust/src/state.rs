use std::collections::HashMap;
use std::str::FromStr;
use std::fmt::Debug;
use std::time::{Duration, UNIX_EPOCH};
use diesel::QueryDsl;
use diesel_async::RunQueryDsl;
use snowflake::SnowflakeIdBucket;
use tokio::sync::Mutex;
use crate::db::pool::{create_pool, Pool};
use crate::errors::{MuuzikaInternalError, MuuzikaResult};
use crate::misc::U5;

pub struct State {
    pub port: u16,
    pub params: DbParameters,
    pub pool: Pool,
    pub id_bucket: Mutex<SnowflakeIdBucket>,
}

impl State {
    pub async fn init(env: &EnvParameters) -> MuuzikaResult<Self> {
        let pool = create_pool(&env.database_url, env.pool_size).map_err(MuuzikaInternalError::from)?;
        let db_params = DbParameters::load(&pool).await?;
        let epoch = UNIX_EPOCH + Duration::from_millis(db_params.snowflake_epoch_offset_ms);
        let id_bucket = SnowflakeIdBucket::with_epoch((&env.snowflake_machine_id).into(), (&env.snowflake_node_id).into(), epoch);

        Ok(Self {
            port: env.listen_port,
            params: db_params,
            pool,
            id_bucket: Mutex::new(id_bucket),
        })
    }
}

#[derive(Debug)]
pub struct EnvParameters {
    pub database_url: String,
    pub pool_size: usize,
    pub snowflake_machine_id: U5,
    pub snowflake_node_id: U5,
    pub listen_address: String,
    pub listen_port: u16,
}

impl EnvParameters {
    pub fn from_env() -> Self {
        Self {
            database_url: read_required_env_var_parse("DATABASE_URL"),
            pool_size: read_required_env_var_parse("POOL_SIZE"),
            snowflake_machine_id: read_required_env_var_parse("SNOWFLAKE_MACHINE_ID"),
            snowflake_node_id: read_required_env_var_parse("SNOWFLAKE_NODE_ID"),
            listen_address: read_env_var_parse_default("LISTEN_ADDRESS", "127.0.0.1".into()),
            listen_port: read_env_var_parse_default("LISTEN_PORT", 50051),
        }
    }
}

#[derive(Debug)]
pub struct DbParameters {
    pub snowflake_epoch_offset_ms: u64,
    pub round_duration_leeway_ms: u64,
    pub round_start_delay_ms: u64,
    pub min_username_length: u8,
    pub max_username_length: u8,
    pub max_password_length: u8,
    pub max_rounds: u8,
    pub max_players: u8,
    pub max_playlist_size: u32,
}

impl DbParameters {
    pub async fn load(pool: &Pool) -> MuuzikaResult<Self> {
        let map = get_all_db_parameters(pool).await?;

        Ok(Self {
            snowflake_epoch_offset_ms: get_and_parse_db_parameter(&map, "SNOWFLAKE_EPOCH_OFFSET_MS"),
            round_duration_leeway_ms: get_and_parse_db_parameter(&map, "ROUND_DURATION_LEEWAY_MS"),
            round_start_delay_ms: get_and_parse_db_parameter(&map, "ROUND_START_DELAY_MS"),
            min_username_length: get_and_parse_db_parameter(&map, "MIN_USERNAME_LENGTH"),
            max_username_length: get_and_parse_db_parameter(&map, "MAX_USERNAME_LENGTH"),
            max_password_length: get_and_parse_db_parameter(&map, "MAX_PASSWORD_LENGTH"),
            max_rounds: get_and_parse_db_parameter(&map, "MAX_ROUNDS"),
            max_players: get_and_parse_db_parameter(&map, "MAX_PLAYERS"),
            max_playlist_size: get_and_parse_db_parameter(&map, "MAX_PLAYLIST_SIZE"),
        })
    }
}

fn read_env_var_parse<T: FromStr>(name: &str) -> Option<T>
    where
        T::Err: Debug,
{
    std::env::var(name).ok().map(|s| s.parse().expect(&format!("Invalid environment variable: {}", name)))
}

fn read_required_env_var_parse<T: FromStr>(name: &str) -> T
    where
        T::Err: Debug,
{
    read_env_var_parse(name).expect(&format!("Missing environment variable: {}", name))
}

fn read_env_var_parse_default<T: FromStr>(name: &str, default: T) -> T
    where
        T::Err: Debug,
{
    read_env_var_parse(name).unwrap_or(default)
}

async fn get_all_db_parameters(pool: &Pool) -> MuuzikaResult<HashMap<String, String>> {
    use crate::db::schema::parameters::dsl::*;

    let mut conn = pool.get().await.map_err(MuuzikaInternalError::from)?;

    let map = parameters
        .select((name, value))
        .load::<(String, String)>(&mut conn)
        .await
        .map_err(MuuzikaInternalError::from)?
        .into_iter()
        .collect::<HashMap<_, _>>();

    Ok(map)
}

pub fn get_and_parse_db_parameter<T: FromStr>(map: &HashMap<String, String>, key: &str) -> T
    where
        T::Err: Debug,
{
    map.get(key).expect(&format!("Missing database parameter: {}", key)).parse().expect(&format!("Invalid database parameter: {}", key))
}