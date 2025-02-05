use std::fmt::Debug;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RequestError<E: Debug> {
    #[error("Failed to send request")]
    FailedToSend,
    #[error("Failed to receive response")]
    FailedToReceive,
    #[error("Received empty response")]
    EmptyResponse,
    #[error("Received unexpected response")]
    UnexpectedResponse,
    #[error("Received error response: {:?}", .0)]
    ErrorResponse(#[from]E),
}

pub type RequestResult<T, E> = Result<T, RequestError<E>>;
