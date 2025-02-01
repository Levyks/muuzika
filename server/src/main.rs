use muuzika_server::proto::registry::registry_service_client::RegistryServiceClient;
use muuzika_server::proto::registry::{
    server_to_registry_message, Greeting, ServerRegistration, ServerToRegistryMessage,
};
use std::env;
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use tonic::codegen::tokio_stream::StreamExt;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    if env::var_os("RUST_LOG").is_none() {
        env::set_var("RUST_LOG", "info");
    }
    pretty_env_logger::init_timed();

    let mut client = RegistryServiceClient::connect("http://127.0.0.1:50051").await?;

    log::info!("Connected to registry server");

    let (tx, rx) = tokio::sync::mpsc::channel::<ServerToRegistryMessage>(1);
    let stream = ReceiverStream::new(rx);

    let response = client.register_server(stream).await?;

    let registration = ServerRegistration {
        address: "localhost:8080".to_string(),
        rooms: vec![],
        capacity: Some(1024),
        description: Some("Test server".to_string()),
    };

    let message = ServerToRegistryMessage {
        message_id: 42,
        message: Some(server_to_registry_message::Message::Registration(
            registration,
        )),
    };

    tx.send(message).await?;
    std::mem::forget(tx);

    let mut resp_stream = response.into_inner();
    while let Some(received) = resp_stream.message().await? {
        println!("\treceived message: `{:?}`", received);
    }

    Ok(())
}
