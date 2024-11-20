pub mod db;
pub mod server_handler;
pub mod service;
pub mod state;

pub mod proto {
    pub mod common {
        tonic::include_proto!("muuzika.common");
    }
    pub mod registry {
        tonic::include_proto!("muuzika.registry");
    }
}
