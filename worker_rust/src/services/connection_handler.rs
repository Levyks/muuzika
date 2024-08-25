use crate::proto::common::Empty;
use crate::proto::connection_handler::connection_handler_service_server::{ConnectionHandlerService, ConnectionHandlerServiceServer};
use crate::proto::connection_handler::{PlayerWithServer, ServerIdentifier};
use crate::state::State;
use fe2o3_amqp::types::messaging::Data;
use fe2o3_amqp::types::primitives::Binary;
use fe2o3_amqp::{Sender, Session};
use log::error;
use serde_json::json;
use std::sync::Arc;
use tonic::{Request, Response, Status};

pub struct ConnectionHandlerServiceImpl {
    state: Arc<State>,
}

#[tonic::async_trait]
impl ConnectionHandlerService for ConnectionHandlerServiceImpl {
    async fn on_connect(&self, request: Request<PlayerWithServer>) -> Result<Response<Empty>, Status> {
        let request = request.into_inner();

        let bytes = request.username.as_bytes().to_vec();

        let mut sender = self.state.broadcast_sender.lock().await;
        sender
            /*.send(json!({
                "room_code": request.room_code,
                "data": {
                    "type": "PLAYER_CONNECTED",
                    "username": request.username,
                }
            }).to_string())*/
            .send(Data(Binary::from(bytes)))
            .await
            .map_err(|e| {
                error!("Failed to send message: {}", e);
                Status::internal("Failed to send message")
            })?;

        Ok(Response::new(Empty {}))
    }

    async fn on_disconnect(&self, request: Request<PlayerWithServer>) -> Result<Response<Empty>, Status> {
        todo!()
    }

    async fn im_alive(&self, request: Request<ServerIdentifier>) -> Result<Response<Empty>, Status> {
        todo!()
    }
}

pub fn create_connection_handler_service_server(state: &Arc<State>) -> ConnectionHandlerServiceServer<impl ConnectionHandlerService> {
    ConnectionHandlerServiceServer::new(ConnectionHandlerServiceImpl {
        state: state.clone()
    })
}