use crate::proto::common::Error;

#[derive(thiserror::Error, Debug)]
pub enum MuuzikaError {
    #[error("Internal error")]
    InternalError,
    #[error("AMQP open error")]
    AmqpOpenError(#[from] fe2o3_amqp::connection::OpenError),
    #[error("AMQP begin error")]
    AmqpBeginError(#[from] fe2o3_amqp::session::BeginError),
    #[error("AMQP sender attach error")]
    AmqpSenderAttachError(#[from] fe2o3_amqp::link::SenderAttachError),
}

pub type AppResult<T> = Result<T, MuuzikaError>;

impl From<MuuzikaError> for Error {
    fn from(error: MuuzikaError) -> Self {
        match error {
            _ => Error::Internal,
        }
    }
}