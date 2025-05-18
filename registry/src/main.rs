use std::env;
use std::net::SocketAddr;
use log::LevelFilter;
use pretty_env_logger::formatted_timed_builder;
use muuzika_registry::{serve, Options};

#[tokio::main]
async fn main() {
    init_logger();

    let listen_address = env::var("LISTEN_ADDRESS")
        .unwrap_or_else(|_| "0.0.0.0:50051".to_string())
        .parse::<SocketAddr>()
        .expect("LISTEN_ADDRESS is not a valid address");

    let options = Options::from_env();
    serve(listen_address, options).await;
}

fn init_logger() {
    let mut builder = formatted_timed_builder();

    if let Ok(rust_log) = env::var("RUST_LOG") {
        builder.parse_filters(&rust_log);
    } else {
        builder.filter(None, LevelFilter::Info);
    }

    builder.init();
}