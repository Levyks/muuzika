mod db;
pub mod errors;
mod misc;
pub mod mq;
mod queue;
pub mod services;
pub mod state;

pub mod proto {
    pub mod common {
        tonic::include_proto!("muuzika.common");
    }
    pub mod lobby {
        tonic::include_proto!("muuzika.lobby");
    }
    pub mod jobs {
        tonic::include_proto!("muuzika.jobs");
    }
    pub mod broadcast {
        tonic::include_proto!("muuzika.broadcast");
    }
    pub mod errors {
        tonic::include_proto!("muuzika.errors");
    }
}
