use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::oneshot::Receiver;
use tokio::sync::RwLock;
use tonic::transport::Server;
use crate::generator::{RoomCodeCollection, RoomCodeGeneratorImpl};
use crate::options::Options;
use crate::registry::Registry;
use crate::services::RegistryGrpcServices;

async fn serve_base(listen_address: SocketAddr, options: Options, generator: Option<RoomCodeGeneratorImpl>, shutdown_rx: Option<Receiver<()>>) {
    let generator = generator.unwrap_or_else(|| RoomCodeGeneratorImpl::new(None, 3, 200));
    let registry = Arc::new(RwLock::new(Registry::new(generator)));
    log::info!("Starting registry server on {listen_address} with options: {options:?}");

    let router = Server::builder().add_registry_services(registry, options);

    let result = if let Some(shutdown_rx) = shutdown_rx {
        router.serve_with_shutdown(listen_address, async {
            shutdown_rx.await.ok();
        }).await
    } else {
        router.serve(listen_address).await
    };

    result.expect("Failed to start server");
}

pub async fn serve(listen_address: SocketAddr, options: Options) {
    serve_base(listen_address, options, None, None).await;
}

pub async fn serve_with_shutdown_and_codes(
    listen_address: SocketAddr,
    options: Options,
    shutdown_rx: Receiver<()>,
    codes: Option<Vec<u32>>,
) {
    let generator = codes.map(|codes| {
        let power = codes.iter().max().map(|&code| code.ilog10()).unwrap_or(0);
        let code_collection = RoomCodeCollection::new_from_vec(power, codes);
        RoomCodeGeneratorImpl::new_with_collection(code_collection, power as usize)
    });
    serve_base(listen_address, options, generator, Some(shutdown_rx)).await;
}