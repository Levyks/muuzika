use crate::services::connection_handler::create_connection_handler_service_server;
use crate::state::State;
use std::sync::Arc;
use tonic::transport::server::Router;
use tonic::transport::Server;

mod connection_handler;

pub trait MuuzikaGrpcServices {
    fn add_services(&mut self, state: &Arc<State>) -> Router;
}

impl MuuzikaGrpcServices for Server {
    fn add_services(&mut self, state: &Arc<State>) -> Router {
        self.add_service(create_connection_handler_service_server(state))
    }
}