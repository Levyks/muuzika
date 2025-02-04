use muuzika_registry::codes::RoomCodeGeneratorImpl;
use muuzika_registry::registry::Registry;
use muuzika_registry::services::RegistryGrpcServices;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init_timed();

    let listen_address = env::var("LISTEN_ADDRESS")
        .unwrap_or("0.0.0.0:50051".to_string())
        .parse::<SocketAddr>()
        .expect("LISTEN_ADDRESS is not a valid address");

    let generator = RoomCodeGeneratorImpl::new(None, 3, 200);
    let registry = Arc::new(Registry::new(generator));

    log::info!("Registry server listening on {}", listen_address);
    Server::builder()
        .add_registry_services(&registry)
        .serve(listen_address)
        .await?;

    Ok(())
}
