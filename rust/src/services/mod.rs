use std::sync::Arc;
use tonic::transport::Server;
use tonic::transport::server::Router;
use crate::services::lobby::create_lobby_service_server;
use crate::state::State;

mod lobby;

pub trait MuuzikaGrpcServices {
    fn add_services(&mut self, state: &Arc<State>) -> Router;
}

impl MuuzikaGrpcServices for Server {
    fn add_services(&mut self, state: &Arc<State>) -> Router {
        self.add_service(create_lobby_service_server(state))
    }
}