pub mod services;
mod state;

pub mod proto {
    pub mod common {
        tonic::include_proto!("muuzika.common");
    }
    pub mod connection_handler {
        tonic::include_proto!("muuzika.connection_handler");
    }
}