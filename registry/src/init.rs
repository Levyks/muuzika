use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tonic::transport::Server;
use crate::generator::RoomCodeGeneratorImpl;
use crate::options::Options;
use crate::registry::Registry;
use crate::services::RegistryGrpcServices;

pub async fn serve(listen_address: SocketAddr, options: Options) {
    let generator = RoomCodeGeneratorImpl::new(None, 3, 200);
    let registry = Arc::new(RwLock::new(Registry::new(generator)));
    log::info!("Starting registry server on {listen_address} with options: {options:?}");
    Server::builder()
        .add_registry_services(registry, options)
        .serve(listen_address)
        .await
        .expect("Failed to start server");
}