use std::sync::Arc;
use tonic::{Request, Response, Status};
use crate::proto::{CodeList, Empty, Id};
use crate::proto::lobby_service_server::{LobbyService, LobbyServiceServer};
use crate::state::State;

pub struct LobbyServiceImpl {
    state: Arc<State>
}

#[tonic::async_trait]
impl LobbyService for LobbyServiceImpl {
    async fn list_possible_codes(&self, request: Request<Empty>) -> Result<Response<CodeList>, Status> {
        todo!()
    }

    async fn generate_id(&self, request: Request<Empty>) -> Result<Response<Id>, Status> {
        let id = self.state.id_bucket.lock().await.get_id();
        Ok(Response::new(Id { id }))
    }
}

pub fn create_lobby_service_server(state: &Arc<State>) -> LobbyServiceServer<impl LobbyService> {
    LobbyServiceServer::new(LobbyServiceImpl {
        state: state.clone()
    })
}