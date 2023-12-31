// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "room_status"))]
    pub struct RoomStatus;
}

diesel::table! {
    players (id) {
        id -> Int8,
        #[max_length = 63]
        username -> Varchar,
        room_id -> Int8,
        last_seen -> Nullable<Timestamptz>,
        enabled -> Bool,
        online -> Bool,
    }
}

diesel::table! {
    possible_codes (code) {
        #[max_length = 31]
        code -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::RoomStatus;

    rooms (id) {
        id -> Int8,
        #[max_length = 31]
        code -> Varchar,
        enabled -> Bool,
        status -> RoomStatus,
        leader_id -> Nullable<Int8>,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    players,
    possible_codes,
    rooms,
);
