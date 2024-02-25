use diesel_async::pooled_connection::deadpool;
use fe2o3_amqp::types::definitions::{AmqpError, Error, ErrorCondition};
use serde::Serialize;
use thiserror::Error;

#[derive(Error, Serialize, Debug)]
pub enum MuuzikaError {
    #[error("internal error")]
    InternalError(
        #[from]
        #[serde(skip)]
        MuuzikaInternalError,
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

impl From<MuuzikaError> for Error {
    fn from(e: MuuzikaError) -> Self {
        let condition = ErrorCondition::AmqpError(AmqpError::InternalError);
        Error::new(condition, Some(e.to_string()), None)
    }
}
