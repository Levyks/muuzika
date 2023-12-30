use diesel::prelude::*;

#[derive(Debug, diesel_derive_enum::DbEnum)]
#[ExistingTypePath = "crate::db::schema::sql_types::RoomStatus"]
pub enum RoomStatus {
    Lobby,
    InGame,
    Finished,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::db::schema::players)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Player {
    pub id: i64,
    pub username: String,
    pub room_id: i64,
    pub last_seen: Option<chrono::DateTime<chrono::Utc>>,
    pub enabled: bool,
    pub online: bool
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::db::schema::rooms)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Room {
    pub id: i64,
    pub code: String,
    pub leader_id: Option<i64>,
    pub status: RoomStatus,
    pub enabled: bool,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = crate::db::schema::possible_codes)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct PossibleCode {
    pub code: String,
}