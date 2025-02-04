use crate::codes::RoomCodeGenerator;
use crate::registry::Registry;
use crate::services::registry_lobby_service::RegistryLobbyServiceImpl;
use crate::services::registry_service::RegistryServiceImpl;
use std::sync::Arc;
use tonic::transport::server::Router;
use tonic::transport::Server;

mod registry_lobby_service;
mod registry_service;

pub trait RegistryGrpcServices<G: RoomCodeGenerator + Send + 'static> {
    fn add_registry_services(&mut self, registry: &Arc<Registry<G>>) -> Router;
}

impl<G: RoomCodeGenerator + Send + 'static> RegistryGrpcServices<G> for Server {
    fn add_registry_services(&mut self, registry: &Arc<Registry<G>>) -> Router {
        self.add_service(RegistryServiceImpl::server(registry))
            .add_service(RegistryLobbyServiceImpl::server(registry))
    }
}
