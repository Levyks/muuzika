use thiserror::Error;
use serde::Serialize;
use diesel_async::pooled_connection::deadpool;

#[derive(Error, Serialize, Debug)]
pub enum MuuzikaError {
    #[error("internal error")]
    InternalError(
        #[from]
        #[serde(skip)]
        MuuzikaInternalError
    ),
}

#[derive(Error, Debug)]
pub enum MuuzikaInternalError {
    #[error("database error")]
    DatabaseError(#[from] diesel::result::Error),
    #[error("database pool error")]
    DatabasePoolError(#[from] deadpool::PoolError),
    #[error("database pool build error")]
    DatabasePoolBuildError(#[from] deadpool::BuildError),
    #[error("unknown error")]
    Unknown,
}

pub type MuuzikaResult<T> = Result<T, MuuzikaError>;