use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::sync::mpsc::Receiver;
use tonic::codegen::tokio_stream::Stream;

pub struct NotifiableReceiverStream<T> {
    inner: Receiver<T>,
    on_close: Option<Box<dyn FnOnce() + Send>>,
}

impl<T> NotifiableReceiverStream<T> {
    pub fn new(recv: Receiver<T>, on_close: impl FnOnce() + Send + 'static) -> Self {
        Self { inner: recv, on_close: Some(Box::new(on_close)) }
    }

    pub fn close(&mut self) {
        self.inner.close();
    }
}

impl<T> Stream for NotifiableReceiverStream<T> {
    type Item = T;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.poll_recv(cx)
    }
}

impl<T> Drop for NotifiableReceiverStream<T> {
    fn drop(&mut self) {
        if let Some(on_close) = self.on_close.take() {
            on_close();
        }
    }
}

#[macro_export]
macro_rules! handle_response_complete {
    ($rx:expr, $response_variant:path, $success_variant:path, $error_variant:path) => {
        match $rx.await {
            Ok(response) => match response.response {
                Some($response_variant(response)) => match response.response {
                    Some($success_variant(success)) => Ok(success),
                    Some($error_variant(error)) => match error.error {
                        Some(error) => Err(error.into()),
                        None => Err(RequestError::EmptyResponse),
                    },
                    None => Err(RequestError::EmptyResponse),
                },
                Some(_) => Err(RequestError::UnexpectedResponse),
                None => Err(RequestError::EmptyResponse),
            },
            Err(_) => Err(RequestError::FailedToReceive),
        }
    };
}

#[macro_export]
macro_rules! send_request {
    ($self:ident, $request:expr, $response_variant:path, $success_variant:path, $error_variant:path) => {{
        let (request_id, rx) = $self.add_pending();

        let message = $crate::packing::RegistryToServerMessagePack::pack($request, Some(request_id));

        $self.tx.send(Ok(message)).await.map_err(|_| RequestError::FailedToSend)?;

        handle_response_complete!(rx, $response_variant, $success_variant, $error_variant)
    }};
}

#[macro_export]
macro_rules! handle_response {
    ($rx:expr, $expected_response_variant:path) => {
        match $rx.await {
            Ok(response) => match response.response {
                Some($expected_response_variant(response)) => match response.response {
                    Some(response) => Ok(response),
                    None => Err(RequestError::EmptyResponse),
                },
                Some(_) => Err(RequestError::UnexpectedResponse),
                None => Err(RequestError::EmptyResponse),
            },
            Err(_) => Err(RequestError::FailedToReceive),
        }
    };
}