use log::error;
use thiserror::Error;
use tonic::Status;

#[derive(Error, Debug)]
pub enum MuuzikaError {
    #[error("proto defined error")]
    ProtoDefined(crate::proto::errors::error::Error),
    #[error("database error")]
    DatabaseError(#[from] sqlx::Error),
    #[error("mq open error")]
    MqOpenError(#[from] fe2o3_amqp::connection::OpenError),
    #[error("mq begin error")]
    MqBeginError(#[from] fe2o3_amqp::session::BeginError),
    #[error("mq receiver attach error")]
    MqReceiverAttachError(#[from] fe2o3_amqp::link::ReceiverAttachError),
    #[error("mq link state error")]
    MqLinkStateError(#[from] fe2o3_amqp::link::LinkStateError),
    #[error("mq illegal link state error")]
    MqIllegalLinkStateError(#[from] fe2o3_amqp::link::IllegalLinkStateError),
    #[error("unknown error")]
    Unknown,
}

pub type MuuzikaResult<T> = Result<T, MuuzikaError>;

impl From<MuuzikaError> for crate::proto::errors::Error {
    fn from(e: MuuzikaError) -> Self {
        match e {
            MuuzikaError::ProtoDefined(e) => crate::proto::errors::Error { error: Some(e) },
            _ => {
                error!("Unknown error: {:?}", e);
                crate::proto::errors::Error {
                    error: Some(crate::proto::errors::error::Error::Internal(
                        crate::proto::errors::InternalError {},
                    )),
                }
            }
        }
    }
}

impl From<crate::proto::errors::Error> for MuuzikaError {
    fn from(e: crate::proto::errors::Error) -> Self {
        println!("Converting proto error to MuuzikaError: {:?}", e);
        match e.error {
            Some(e) => MuuzikaError::ProtoDefined(e),
            None => MuuzikaError::Unknown,
        }
    }
}
