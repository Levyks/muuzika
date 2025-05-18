use crate::proto::registry::registry_lobby_service_server::{RegistryLobbyService, RegistryLobbyServiceServer};
use crate::proto::registry::{get_server_info_error, CreateRoomError, CreateRoomRequest, GetServerInfoError, JoinRoomRequest, JoinRoomResponse, ServerId, ServerInfo};
use crate::registry::Registry;
use crate::utils::packing::into_any_bytes;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::{Code, Request, Response, Status};

pub struct RegistryLobbyServiceImpl {
    registry: Arc<RwLock<Registry>>,
}

impl RegistryLobbyServiceImpl {
    pub fn new(registry: Arc<RwLock<Registry>>) -> Self {
        Self { registry }
    }

    pub fn server(registry: Arc<RwLock<Registry>>) -> RegistryLobbyServiceServer<Self> {
        RegistryLobbyServiceServer::new(Self::new(registry))
    }
}

#[tonic::async_trait]
impl RegistryLobbyService for RegistryLobbyServiceImpl {
    async fn create_room(&self, request: Request<CreateRoomRequest>) -> Result<Response<JoinRoomResponse>, Status> {
        let response = Registry::create_room(&self.registry, request.into_inner()).await
            .map_err(|error| CreateRoomError {
                error: Some(error),
            })?;

        Ok(Response::new(response))
    }

    async fn join_room(&self, request: Request<JoinRoomRequest>) -> Result<Response<JoinRoomResponse>, Status> {
        todo!()
    }

    async fn get_server_info(&self, request: Request<ServerId>) -> Result<Response<ServerInfo>, Status> {
        let server_id = request.into_inner();
        let registry = self.registry.read().await;

        let server_info = registry.get_server_info(&server_id)
            .ok_or(GetServerInfoError {
                error: Some(get_server_info_error::Error::ServerNotFound(()))
            })?;

        Ok(Response::new(server_info))
    }
}

impl From<CreateRoomError> for Status {
    fn from(error: CreateRoomError) -> Self {
        use crate::proto::registry::create_room_error::Error;
        let (code, message) = match error.error {
            Some(error) => match error {
                Error::InternalError(_) => (Code::Internal, "Internal error"),
                Error::OutOfCodes(_) => (Code::ResourceExhausted, "No room codes available"),
                Error::NoServerAvailable(_) => (Code::Unavailable, "No server available"),
            },
            None => (Code::Unknown, "Unknown error"),
        };

        Status::with_details(
            code,
            message,
            into_any_bytes(error, "type.googleapis.com/com.muuzika.registry.CreateRoomError".into()),
        )
    }
}

impl From<GetServerInfoError> for Status {
    fn from(error: GetServerInfoError) -> Self {
        use get_server_info_error::Error;
        let (code, message) = match error.error {
            Some(error) => match error {
                Error::ServerNotFound(_) => (Code::NotFound, "Server not found"),
            }
            _ => (Code::Unknown, "Unknown error"),
        };

        Status::with_details(
            code,
            message,
            into_any_bytes(error, "type.googleapis.com/com.muuzika.registry.GetServerInfoError".into()),
        )
    }
}