mod db;
mod errors;
mod misc;
pub mod mq;
mod queue;
pub mod services;
pub mod state;

pub mod proto {
    tonic::include_proto!("muuzika");
}
