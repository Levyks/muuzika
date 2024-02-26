use std::sync::Arc;

use snowflake::SnowflakeIdGenerator;
use sqlx::{Postgres, Transaction};
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};

use crate::db::queries::{create_player, create_room_with_available_code, update_room_leader};
use crate::errors::{MuuzikaError, MuuzikaResult};
use crate::proto::lobby::{JoinRoomRequest, RoomResponse, RoomSync, UsernameAndCaptcha};
use crate::proto::lobby::lobby_service_server::{LobbyService, LobbyServiceServer};
use crate::state::State;

pub struct LobbyServiceImpl {
    state: Arc<State>,
}

#[tonic::async_trait]
impl LobbyService for LobbyServiceImpl {
    async fn create_room(
        &self,
        request: Request<UsernameAndCaptcha>,
    ) -> Result<Response<RoomResponse>, Status> {
        // TODO: fix this
        let mut transaction = self.state.pool.begin().await.unwrap();
        match create_room(
            &mut transaction,
            &self.state.id_generator,
            request.into_inner().username,
        )
        .await
        {
            Ok(room) => {
                transaction.commit().await.unwrap();
                Ok(Response::new(RoomResponse {
                    response: Some(crate::proto::lobby::room_response::Response::RoomSync(room)),
                }))
            }
            Err(e) => {
                transaction.rollback().await.unwrap();
                Ok(Response::new(RoomResponse {
                    response: Some(crate::proto::lobby::room_response::Response::Error(
                        e.into(),
                    )),
                }))
            }
        }
    }

    async fn join_room(
        &self,
        request: Request<JoinRoomRequest>,
    ) -> Result<Response<RoomResponse>, Status> {
        todo!()
    }
}

pub fn create_lobby_service_server(state: &Arc<State>) -> LobbyServiceServer<impl LobbyService> {
    LobbyServiceServer::new(LobbyServiceImpl {
        state: state.clone(),
    })
}

async fn create_room(
    transaction: &mut Transaction<'_, Postgres>,
    id_generator: &Mutex<SnowflakeIdGenerator>,
    username: String,
) -> MuuzikaResult<RoomSync> {
    let room_id = id_generator.lock().await.real_time_generate();
    let code = create_room_with_available_code(&mut **transaction, room_id).await?;
    let player_id = id_generator.lock().await.real_time_generate();
    create_player(&mut **transaction, player_id, &username, room_id).await?;
    update_room_leader(&mut **transaction, room_id, player_id).await?;
    return Err(MuuzikaError::ProtoDefined(
        crate::proto::errors::error::Error::InvalidUsername(
            crate::proto::errors::InvalidUsernameError {},
        ),
    ));
    let room = RoomSync {
        id: room_id,
        code,
        players: vec![],
        playlist: None,
        options: None,
        your_player_id: 0,
    };
    Ok(room)
}
