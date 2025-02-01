use crate::codes::RoomCodeGenerator;
use crate::proto::common::RoomCode;
use crate::proto::registry::{RoomRegistryDefinition, ServerId};
use crate::server;
use dashmap::DashMap;
use std::hash::{Hash, Hasher};
use std::sync::atomic::AtomicU32;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct Room {
    pub code: RoomCode,
    pub leader_username: String,
    pub num_players: u32,
    pub is_public: bool,
    pub has_password: bool,
    pub server_id: ServerId,
}

impl Room {
    pub fn from_def(room_def: RoomRegistryDefinition, code: RoomCode, server_id: ServerId) -> Self {
        Room {
            code,
            leader_username: room_def.leader_username,
            num_players: room_def.num_players,
            is_public: room_def.is_public,
            has_password: room_def.has_password,
            server_id,
        }
    }
}

pub struct State {
    pub server_id_counter: AtomicU32,
    pub servers: DashMap<ServerId, server::Server>,

    pub room_code_generator: Arc<Mutex<RoomCodeGenerator>>,
    pub rooms: DashMap<RoomCode, Arc<Room>>,
}

impl State {
    pub fn new() -> Self {
        State {
            server_id_counter: AtomicU32::new(0),
            servers: DashMap::new(),
            rooms: DashMap::new(),
            // TODO: extract these values from env
            room_code_generator: Arc::new(Mutex::new(RoomCodeGenerator::new(None, 3, 200))),
        }
    }

    pub async fn get_room_code(&self) -> RoomCode {
        let mut generator = self.room_code_generator.lock().await;

        let code = generator.get_code();

        if !generator.has_available_code() {
            let generator = self.room_code_generator.clone();
            tokio::spawn(async move {
                generator.lock().await.create_next_collection();
            });
        }

        code
    }
}

impl Eq for RoomCode {}

impl Hash for RoomCode {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.code.hash(state);
    }
}
