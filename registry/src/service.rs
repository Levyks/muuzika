use crate::db::Server;
use crate::proto::registry::registry_service_server::RegistryService;
use crate::proto::registry::{
    registry_to_server_message, server_to_registry_message, CreateRoomRequest, CreateRoomResponse,
    ListRoomsRequest, ListRoomsResponse, RegistryToServerMessage, ServerId,
    ServerToRegistryMessage,
};
use crate::state::State;
use anyhow::Context;
use std::sync::Arc;
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};

pub struct RegistryServiceImpl {
    pub state: Arc<State>,
}

#[tonic::async_trait]
impl RegistryService for RegistryServiceImpl {
    type RegisterServerStream = ReceiverStream<Result<RegistryToServerMessage, Status>>;

    async fn register_server(
        &self,
        request: Request<Streaming<ServerToRegistryMessage>>,
    ) -> Result<Response<Self::RegisterServerStream>, Status> {
        let (tx, rx) = tokio::sync::mpsc::channel::<Result<RegistryToServerMessage, Status>>(1);

        tokio::spawn(async move {
            let mut stream = request.into_inner();
            while let Ok(Some(msg)) = stream.message().await {
                match msg.message {
                    Some(server_to_registry_message::Message::Registration(registration)) => {
                        let message = RegistryToServerMessage {
                            message_id: msg.message_id,
                            message: Some(registry_to_server_message::Message::Registration(
                                ServerId { server_id: 42 },
                            )),
                        };
                        if let Err(e) = tx.send(Ok(message)).await {
                            eprintln!("Failed to send message: {:?}", e);
                        }
                    }
                    _ => {
                        eprintln!("Received an unexpected message");
                    }
                }
            }
        });

        Ok(Response::new(ReceiverStream::new(rx)))
    }

    async fn list_rooms(
        &self,
        request: Request<ListRoomsRequest>,
    ) -> Result<Response<ListRoomsResponse>, Status> {
        let servers = sqlx::query_as::<_, Server>("SELECT * FROM servers")
            .fetch_all(&self.state.db)
            .await
            .expect("Failed to fetch servers");

        log::info!("Fetched servers: {:?}", servers);

        todo!()
    }

    async fn create_room(
        &self,
        request: Request<CreateRoomRequest>,
    ) -> Result<Response<CreateRoomResponse>, Status> {
        todo!()
    }
}
