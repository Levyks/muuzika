use crate::generator::{RoomCodeCollection, RoomCodeGeneratorImpl};
use crate::options::Options;
use crate::registry::Registry;
use crate::services::RegistryGrpcServices;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::oneshot::Receiver;
use tokio::sync::RwLock;
use tonic::codegen::tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;

#[derive(Debug)]
enum ServeOpts {
    Addr(SocketAddr),
    ListenerAndShutdown(TcpListener, Receiver<()>),
}

async fn serve_base(opts: ServeOpts, options: Options, generator: Option<RoomCodeGeneratorImpl>) {
    let generator = generator.unwrap_or_else(|| RoomCodeGeneratorImpl::new(None, 3, 200));
    let registry = Arc::new(RwLock::new(Registry::new(generator)));
    log::info!("Starting registry server on {opts:?} with options: {options:?}");

    let router = Server::builder().add_registry_services(registry, options);

    let result = match opts {
        ServeOpts::Addr(listen_address) => {
            router.serve(listen_address).await
        }
        ServeOpts::ListenerAndShutdown(listener, shutdown_rx) => {
            router.serve_with_incoming_shutdown(TcpListenerStream::new(listener), async {
                shutdown_rx.await.ok();
            }).await
        }
    };

    result.expect("Failed to start server");
}

pub async fn serve(listen_address: SocketAddr, options: Options) {
    serve_base(ServeOpts::Addr(listen_address), options, None).await
}

pub async fn serve_with_shutdown_and_codes(
    listener: TcpListener,
    options: Options,
    shutdown_rx: Receiver<()>,
    codes: Option<Vec<u32>>,
) {
    let generator = codes.map(|codes| {
        let power = codes.iter().max().map(|&code| code.ilog10()).unwrap_or(0);
        let code_collection = RoomCodeCollection::new_from_vec(power, codes);
        RoomCodeGeneratorImpl::new_with_collection(code_collection, power as usize)
    });
    serve_base(
        ServeOpts::ListenerAndShutdown(listener, shutdown_rx),
        options,
        generator,
    ).await
}