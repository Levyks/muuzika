use std::collections::HashMap;
use std::fmt::Debug;
use std::str::FromStr;
use std::time::{Duration, UNIX_EPOCH};

use log::debug;
use snowflake::SnowflakeIdGenerator;
use tokio::sync::Mutex;
use url::Url;

use crate::db::pool::{create_pool, Pool};
use crate::db::queries::fetch_all_parameters;
use crate::errors::MuuzikaResult;
use crate::misc::U5;

pub struct State {
    pub params: DbParameters,
    pub pool: Pool,
    pub id_generator: Mutex<SnowflakeIdGenerator>,
}

impl State {
    pub async fn init(env: &EnvParameters) -> MuuzikaResult<Self> {
        let pool = create_pool(&env.database_url, env.pool_size).await?;

        let db_params = DbParameters::load(&pool).await?;
        debug!("Loaded database parameters: {:?}", db_params);
        let epoch = UNIX_EPOCH + Duration::from_millis(db_params.snowflake_epoch_offset_ms);
        let id_generator = SnowflakeIdGenerator::with_epoch(
            (&env.snowflake_machine_id).into(),
            (&env.snowflake_node_id).into(),
            epoch,
        );

        Ok(Self {
            params: db_params,
            pool,
            id_generator: Mutex::new(id_generator),
        })
    }

    pub async fn generate_id(&self) -> i64 {
        let mut id_generator = self.id_generator.lock().await;
        id_generator.real_time_generate()
    }
}

#[derive(Debug)]
pub struct EnvParameters {
    pub database_url: String,
    pub pool_size: u32,
    pub snowflake_machine_id: U5,
    pub snowflake_node_id: U5,
    pub mq_url: Url,
    pub mq_username: String,
    pub mq_password: String,
    pub mq_jobs_queue: String,
    pub mq_broadcast_exchange: String,
}

impl EnvParameters {
    pub fn from_env() -> Self {
        Self {
            database_url: read_required_env_var_parse("DATABASE_URL"),
            pool_size: read_required_env_var_parse("POOL_SIZE"),
            snowflake_machine_id: read_required_env_var_parse("SNOWFLAKE_MACHINE_ID"),
            snowflake_node_id: read_required_env_var_parse("SNOWFLAKE_NODE_ID"),
            mq_url: read_required_env_var_parse("MQ_URL"),
            mq_username: read_required_env_var_parse("MQ_USERNAME"),
            mq_password: read_required_env_var_parse("MQ_PASSWORD"),
            mq_jobs_queue: read_env_var_or_else("MQ_JOBS_QUEUE", || "muuzika_jobs".to_string()),
            mq_broadcast_exchange: read_env_var_or_else("MQ_BROADCAST_EXCHANGE", || {
                "muuzika_broadcast".to_string()
            }),
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
    pub jwt_secret: String,
}

impl DbParameters {
    pub async fn load(pool: &Pool) -> MuuzikaResult<Self> {
        let map = fetch_all_parameters(pool).await?;

        Ok(Self {
            snowflake_epoch_offset_ms: get_and_parse_db_parameter(
                &map,
                "SNOWFLAKE_EPOCH_OFFSET_MS",
            ),
            round_duration_leeway_ms: get_and_parse_db_parameter(&map, "ROUND_DURATION_LEEWAY_MS"),
            round_start_delay_ms: get_and_parse_db_parameter(&map, "ROUND_START_DELAY_MS"),
            min_username_length: get_and_parse_db_parameter(&map, "MIN_USERNAME_LENGTH"),
            max_username_length: get_and_parse_db_parameter(&map, "MAX_USERNAME_LENGTH"),
            max_password_length: get_and_parse_db_parameter(&map, "MAX_PASSWORD_LENGTH"),
            max_rounds: get_and_parse_db_parameter(&map, "MAX_ROUNDS"),
            max_players: get_and_parse_db_parameter(&map, "MAX_PLAYERS"),
            max_playlist_size: get_and_parse_db_parameter(&map, "MAX_PLAYLIST_SIZE"),
            jwt_secret: get_and_parse_db_parameter(&map, "JWT_SECRET"),
        })
    }
}

fn read_env_var_parse<T: FromStr>(name: &str) -> Option<T>
where
    T::Err: Debug,
{
    std::env::var(name).ok().map(|s| {
        s.parse()
            .expect(&format!("Invalid environment variable: {}", name))
    })
}

pub fn read_required_env_var_parse<T: FromStr>(name: &str) -> T
where
    T::Err: Debug,
{
    read_env_var_parse(name).expect(&format!("Missing environment variable: {}", name))
}

pub fn read_env_var_or_else<T: FromStr, F: FnOnce() -> T>(name: &str, f: F) -> T
where
    T::Err: Debug,
{
    read_env_var_parse(name).unwrap_or_else(f)
}

fn get_and_parse_db_parameter<T: FromStr>(map: &HashMap<String, String>, key: &str) -> T
where
    T::Err: Debug,
{
    map.get(key)
        .expect(&format!("Missing database parameter: {}", key))
        .parse()
        .expect(&format!("Invalid database parameter: {}", key))
}
