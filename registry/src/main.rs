#[macro_use]
extern crate log;

use muuzika_registry::db::init_db;
use muuzika_registry::proto::registry::registry_service_server::RegistryServiceServer;
use muuzika_registry::service::RegistryServiceImpl;
use muuzika_registry::state::State;
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

    let db = init_db().await?;
    let state = Arc::new(State { db });

    println!("Registry server listening on {}", listen_address);

    Server::builder()
        .add_service(RegistryServiceServer::new(RegistryServiceImpl { state }))
        .serve(listen_address)
        .await?;

    Ok(())
}
