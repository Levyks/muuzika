use std::sync::Arc;
use tonic::transport::Server;
use crate::services::lobby::create_lobby_service_server;
use crate::state::{EnvParameters, State};

pub async fn start() {
    println!("Initializing server state...");
    let env = EnvParameters::from_env();
    let state = State::init(&env).await.expect("Failed to initialize server state");
    println!("Server state initialized");


    let addr = format!("{}:{}", env.listen_address, env.listen_port).parse().expect("Invalid listen address/port");

    let state = Arc::new(state);
    let server = Server::builder().add_service(create_lobby_service_server(&state));

    println!("GRPC server listening on {}", addr);
    server.serve(addr).await.expect("Failed to start server");
}