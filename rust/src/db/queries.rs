use std::collections::HashMap;

use log::error;
use sqlx::{Executor, Postgres};

use crate::db::pool::Pool;
use crate::errors::MuuzikaResult;

pub async fn create_room_with_available_code<'e, E>(executor: E, id: i64) -> MuuzikaResult<String>
where
    E: 'e + Executor<'e, Database = Postgres>,
{
    let (code,): (String,) = sqlx::query_as("SELECT create_room_with_available_code($1)")
        .bind(id)
        .fetch_one(executor)
        .await
        .map_err(|err| {
            error!("Error creating room: {:?}", err);
            err
        })?;

    Ok(code)
}

pub async fn create_player<'e, E>(
    executor: E,
    id: i64,
    username: &str,
    room_id: i64,
) -> MuuzikaResult<()>
where
    E: 'e + Executor<'e, Database = Postgres>,
{
    sqlx::query("INSERT INTO players (id, username, room_id) VALUES ($1, $2, $3)")
        .bind(id)
        .bind(username)
        .bind(room_id)
        .execute(executor)
        .await?;

    Ok(())
}

pub async fn update_room_leader<'e, E>(
    executor: E,
    room_id: i64,
    leader_id: i64,
) -> MuuzikaResult<bool>
where
    E: 'e + Executor<'e, Database = Postgres>,
{
    let result = sqlx::query("UPDATE rooms SET leader_id = $1 WHERE id = $2")
        .bind(leader_id)
        .bind(room_id)
        .execute(executor)
        .await?;

    Ok(result.rows_affected() > 0)
}

pub async fn fetch_all_parameters<'e, E>(executor: E) -> MuuzikaResult<HashMap<String, String>>
where
    E: 'e + Executor<'e, Database = Postgres>,
{
    let parameters: Vec<(String, String)> = sqlx::query_as("SELECT * FROM parameters")
        .fetch_all(executor)
        .await?;

    Ok(parameters.into_iter().collect::<HashMap<_, _>>())
}
