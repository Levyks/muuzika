use crate::registry::Registry;
use crate::options::Options;
use crate::services::registry_service::RegistryServiceImpl;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::transport::server::Router;
use tonic::transport::Server;

mod registry_service;

pub trait RegistryGrpcServices {
    fn add_registry_services(&mut self, registry: Arc<RwLock<Registry>>, options: Options) -> Router;
}

impl RegistryGrpcServices for Server {
    fn add_registry_services(&mut self, registry: Arc<RwLock<Registry>>, options: Options) -> Router {
        let options = Arc::new(options);
        self.add_service(RegistryServiceImpl::server(registry, options))
    }
}
