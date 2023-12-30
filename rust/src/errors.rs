use diesel_async::pooled_connection::deadpool::PoolError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum MuuzikaError {
    #[error("Internal error: {0:?}")]
    Internal(Option<String>),
}
pub type MuuzikaResult<T> = Result<T, MuuzikaError>;

impl From<PoolError> for MuuzikaError {
    fn from(e: PoolError) -> Self {
        Self::Internal(Some(e.to_string()))
    }
}

impl From<diesel::result::Error> for MuuzikaError {
    fn from(e: diesel::result::Error) -> Self {
        Self::Internal(Some(e.to_string()))
    }
}

impl From<MuuzikaError> for tonic::Status {
    fn from(e: MuuzikaError) -> Self {
        match e {
            MuuzikaError::Internal(Some(msg)) => tonic::Status::internal(msg),
            MuuzikaError::Internal(None) => tonic::Status::internal("Internal error"),
        }
    }
}