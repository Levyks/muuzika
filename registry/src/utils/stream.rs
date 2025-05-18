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