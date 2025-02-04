pub mod codes;
pub mod services;
pub mod registry;
pub mod server;
pub mod room;
mod packing;
mod utils;

pub mod proto {
    pub mod common {
        tonic::include_proto!("com.muuzika.common");
    }
    pub mod registry {
        tonic::include_proto!("com.muuzika.registry");
    }
    pub mod issue {
        tonic::include_proto!("com.muuzika.issue");
    }
}
