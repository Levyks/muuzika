use crate::proto::registry::registry_lobby_service_server::{
    RegistryLobbyService, RegistryLobbyServiceServer,
};
use crate::proto::registry::{CreateRoomRequest, CreateRoomResponse};
use crate::state::State;
use std::sync::Arc;
use tonic::{Request, Response, Status};

// impl conversion from RegistryError to tonic::Status
struct RegistryLobbyServiceImpl {
    state: Arc<State>,
}

#[tonic::async_trait]
impl RegistryLobbyService for RegistryLobbyServiceImpl {
    async fn create_room(
        &self,
        request: Request<CreateRoomRequest>,
    ) -> Result<Response<CreateRoomResponse>, Status> {
        todo!()
    }
}

pub fn create_registry_lobby_service_server(
    state: &Arc<State>,
) -> RegistryLobbyServiceServer<impl RegistryLobbyService> {
    RegistryLobbyServiceServer::new(RegistryLobbyServiceImpl {
        state: state.clone(),
    })
}
