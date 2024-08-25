use crate::proto::broadcast::{broadcast_message_data, BroadcastMessage, BroadcastMessageData, PlayerIdentifier};
use crate::proto::common::Empty;
use crate::proto::connection_handler::connection_handler_service_server::{ConnectionHandlerService, ConnectionHandlerServiceServer};
use crate::proto::connection_handler::{PlayerWithServer, ServerIdentifier};
use crate::state::State;
use fe2o3_amqp::types::messaging::Data;
use fe2o3_amqp::types::primitives::Binary;
use fe2o3_amqp::{Sender, Session};
use log::error;
use prost::Message;
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

        let mut buf = Vec::new();

        let message = BroadcastMessage {
            room_code: request.room_code,
            // TODO: implement into for this enum
            data: Some(BroadcastMessageData {
                data: Some(broadcast_message_data::Data::PlayerConnected(PlayerIdentifier {
                    username: request.username,
                }))
            }),
        };
        if let Err(e) = message.encode(&mut buf) {
            error!("Failed to encode message: {}", e);
            return Err(Status::internal("Failed to encode message"));
        }

        let mut sender = self.state.broadcast_sender.lock().await;
        sender
            .send(Data(Binary::from(buf)))
            .await
            .map_err(|e| {
                error!("Failed to send message: {}", e);
                Status::internal("Failed to send message")
            })?;

        Ok(Response::new(Empty {}))
    }

    async fn on_disconnect(&self, request: Request<PlayerWithServer>) -> Result<Response<Empty>, Status> {
        let request = request.into_inner();

        let mut buf = Vec::new();
        let message = BroadcastMessage {
            room_code: request.room_code,
            data: Some(BroadcastMessageData {
                data: Some(broadcast_message_data::Data::PlayerDisconnected(PlayerIdentifier {
                    username: request.username,
                }))
            }),
        };

        if let Err(e) = message.encode(&mut buf) {
            error!("Failed to encode message: {}", e);
            return Err(Status::internal("Failed to encode message"));
        }

        let mut sender = self.state.broadcast_sender.lock().await;
        sender
            .send(Data(Binary::from(buf)))
            .await
            .map_err(|e| {
                error!("Failed to send message: {}", e);
                Status::internal("Failed to send message")
            })?;

        Ok(Response::new(Empty {}))
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