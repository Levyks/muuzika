use diesel::{ExpressionMethods, JoinOnDsl, QueryDsl};
use diesel_async::RunQueryDsl;
use tonic::{Request, Response, Status};
use crate::db::{DbConn, DbPool};
use crate::errors::{MuuzikaError, MuuzikaResult};
use crate::proto::{Empty, CodeList};
use crate::proto::lobby_service_server::LobbyService;

pub struct LobbyServiceImpl {
    pool: DbPool
}

impl LobbyServiceImpl {
    pub fn new(pool: DbPool) -> Self {
        Self {
            pool
        }
    }

    async fn get_conn(&self) -> MuuzikaResult<DbConn> {
        Ok(self.pool.get().await?)
    }
}

#[tonic::async_trait]
impl LobbyService for LobbyServiceImpl {
    async fn list_possible_codes(&self, _: Request<Empty>) -> Result<Response<CodeList>, Status> {
        use crate::db::schema::*;

        let mut conn = self.get_conn().await?;

        let codes = possible_codes::table
            .select(possible_codes::code)
            .load::<String>(&mut conn)
            .await
            .map_err(|e| MuuzikaError::from(e))?; // there's really no better way than this?

        Ok(Response::new(CodeList { codes }))
    }

    async fn list_available_codes(&self, _: Request<Empty>) -> Result<Response<CodeList>, Status> {
        use crate::db::schema::*;

        let mut conn = self.get_conn().await?;

        let codes = possible_codes::table
            .left_join(rooms::table.on(possible_codes::code.eq(rooms::code)))
            .filter(rooms::id.is_null())
            .select(possible_codes::code)
            .load::<String>(&mut conn)
            .await
            .map_err(|e| MuuzikaError::from(e))?; // there's really no better way than this?

        Ok(Response::new(CodeList { codes }))
    }
}