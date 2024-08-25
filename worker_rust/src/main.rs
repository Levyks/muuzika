use fe2o3_amqp::{Sender, Session};
use log::info;
use muuzika_worker::services::MuuzikaGrpcServices;
use muuzika_worker::state::State;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Mutex;
use tonic::transport::Server;

#[tokio::main]
async fn main() {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init_timed();

    info!("Initializing state");
    let state = State::init().await.expect("Failed to initialize state");
    info!("State initialized");

    let listen_address = env::var("LISTEN_ADDRESS")
        .unwrap_or("0.0.0.0:50051".to_string())
        .parse::<SocketAddr>().expect("LISTEN_ADDRESS is not a valid address");

    let mut server = Server::builder().add_services(&Arc::new(state));

    server.serve(listen_address).await.expect("Server failed");
}
