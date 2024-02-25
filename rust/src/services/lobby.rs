use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};

use log::info;
use tokio::sync::mpsc::UnboundedReceiver;
use tonic::{Request, Response, Status};
use tonic::codegen::tokio_stream::Stream;
use tonic::codegen::tokio_stream::wrappers::UnboundedReceiverStream;

use crate::proto::{CodeList, Empty, Id, Message};
// use futures::{Stream, stream};
use crate::proto::lobby_service_server::{LobbyService, LobbyServiceServer};
use crate::state::State;

pub struct LobbyServiceImpl {
    state: Arc<State>,
}

struct MessageStream {
    inner: Pin<Box<dyn Stream<Item = Result<Message, Status>> + Send>>,
}

impl MessageStream {
    fn new(rx: UnboundedReceiver<Result<Message, Status>>) -> Self {
        MessageStream {
            inner: Box::pin(UnboundedReceiverStream::new(rx)),
        }
    }
}

impl Drop for MessageStream {
    fn drop(&mut self) {
        eprintln!("MessageStream dropped");
    }
}

impl Stream for MessageStream {
    type Item = Result<Message, Status>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.inner.as_mut().poll_next(cx)
    }
}

#[tonic::async_trait]
impl LobbyService for LobbyServiceImpl {
    async fn generate_id(&self, _: Request<Empty>) -> Result<Response<Id>, Status> {
        let id = self.state.generate_id().await;
        Ok(Response::new(Id { id }))
    }

    async fn log(&self, request: Request<Message>) -> Result<Response<Empty>, Status> {
        info!("Received message: {:?}", request.get_ref().message);
        Ok(Response::new(Empty {}))
    }

    type ListenMessagesStream = Pin<Box<dyn Stream<Item = Result<Message, Status>> + Send>>;

    async fn listen_messages(
        &self,
        _: Request<Empty>,
    ) -> Result<Response<Self::ListenMessagesStream>, Status> {
        let (tx, rx) = tokio::sync::mpsc::unbounded_channel();

        tokio::spawn(async move {
            loop {
                let message = Message {
                    message: "Hello, World!".to_string(),
                };
                if let Err(err) = tx.send(Ok(message)) {
                    eprintln!("Error sending message: {:?}", err);
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            }
        });

        let stream = MessageStream::new(rx);

        Ok(Response::new(Box::pin(stream)))
    }
}

pub fn create_lobby_service_server(state: &Arc<State>) -> LobbyServiceServer<impl LobbyService> {
    LobbyServiceServer::new(LobbyServiceImpl {
        state: state.clone(),
    })
}
