use crate::codes::RoomCodeGenerator;
use crate::proto::registry::registry_lobby_service_server::{
    RegistryLobbyService, RegistryLobbyServiceServer,
};
use crate::proto::registry::{CreateRoomRequest, CreateRoomResponse};
use crate::registry::Registry;
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct RegistryLobbyServiceImpl<G: RoomCodeGenerator + Send + 'static> {
    registry: Arc<Registry<G>>,
}

#[tonic::async_trait]
impl<G: RoomCodeGenerator + Send + 'static> RegistryLobbyService for RegistryLobbyServiceImpl<G> {
    async fn create_room(
        &self,
        request: Request<CreateRoomRequest>,
    ) -> Result<Response<CreateRoomResponse>, Status> {
        todo!()
    }
}

impl<G: RoomCodeGenerator + Send + 'static> RegistryLobbyServiceImpl<G> {
    pub fn server(registry: &Arc<Registry<G>>) -> RegistryLobbyServiceServer<impl RegistryLobbyService> {
        RegistryLobbyServiceServer::new(Self {
            registry: registry.clone(),
        })
    }
}