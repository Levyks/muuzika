use crate::services::registry_service::create_registry_service_server;
use crate::state::State;
use std::sync::Arc;
use tonic::transport::server::Router;
use tonic::transport::Server;

mod registry_lobby_service;
pub mod registry_service;

pub trait RegistryGrpcServices {
    fn add_registry_services(&mut self, state: &Arc<State>) -> Router;
}

impl RegistryGrpcServices for Server {
    fn add_registry_services(&mut self, state: &Arc<State>) -> Router {
        self.add_service(create_registry_service_server(state))
    }
}
