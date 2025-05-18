mod init;
mod options;
mod services;
mod generator;
mod registry;
mod server;
mod utils;
mod room;
mod messages;

pub mod proto {
    pub mod common {
        tonic::include_proto!("com.muuzika.common");
    }
    pub mod registry {
        tonic::include_proto!("com.muuzika.registry");
    }
}

pub use init::{serve, serve_with_shutdown_and_codes};
pub use options::Options;
