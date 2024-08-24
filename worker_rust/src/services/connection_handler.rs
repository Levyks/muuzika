use crate::proto::common::Empty;
use crate::proto::connection_handler::connection_handler_service_server::{ConnectionHandlerService, ConnectionHandlerServiceServer};
use crate::proto::connection_handler::{PlayerWithServer, ServerIdentifier};
use tonic::{Request, Response, Status};

pub struct ConnectionHandlerServiceImpl;

#[tonic::async_trait]
impl ConnectionHandlerService for ConnectionHandlerServiceImpl {
    async fn on_connect(&self, request: Request<PlayerWithServer>) -> Result<Response<Empty>, Status> {
        todo!()
    }

    async fn on_disconnect(&self, request: Request<PlayerWithServer>) -> Result<Response<Empty>, Status> {
        todo!()
    }

    async fn im_alive(&self, request: Request<ServerIdentifier>) -> Result<Response<Empty>, Status> {
        todo!()
    }
}

pub fn create_connection_handler_service_server() -> ConnectionHandlerServiceServer<impl ConnectionHandlerService> {
    ConnectionHandlerServiceServer::new(ConnectionHandlerServiceImpl)
}