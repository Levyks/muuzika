mod state;
pub mod server;
mod misc;
mod db;
mod errors;
mod services;

pub mod proto { tonic::include_proto!("muuzika"); }