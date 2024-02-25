use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

use log::info;
use tonic::transport::Server;

use muuzika::services::MuuzikaGrpcServices;
use muuzika::state::{EnvParameters, State};

#[tokio::main]
pub async fn main() {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init_timed();

    info!("Initializing server state...");
    let env = EnvParameters::from_env();
    let state = State::init(&env)
        .await
        .expect("Failed to initialize server state");
    info!("Server state initialized");

    let listen_address = env::var("LISTEN_ADDRESS").unwrap_or("127.0.0.1".to_string());
    let listen_port = env::var("LISTEN_PORT").unwrap_or("50051".to_string());
    let addr = format!("{}:{}", listen_address, listen_port)
        .parse::<SocketAddr>()
        .expect("Invalid listen address/port");

    let server = Server::builder().add_services(&Arc::new(state));

    info!("Starting GRPC server on {}", addr);
    server.serve(addr).await.expect("Failed to start server");
}
