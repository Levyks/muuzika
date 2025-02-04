mod codes;
pub mod server;
pub mod services;
pub mod state;

pub mod proto {
    pub mod common {
        tonic::include_proto!("muuzika.common");
    }
    pub mod registry {
        tonic::include_proto!("muuzika.registry");
    }
}
