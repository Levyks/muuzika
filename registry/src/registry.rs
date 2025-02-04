use crate::codes::RoomCodeGenerator;
use crate::proto::common::RoomCode;
use crate::proto::registry::{RegistryToServerMessage, RoomCodeChange, ServerId, ServerRegistrationRequest, ServerRegistrationResponse};
use crate::room::Room;
use crate::server::{Server, ServerRunner};
use dashmap::mapref::multiple::RefMulti;
use dashmap::{DashMap, Entry};
use std::collections::HashSet;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::Mutex;
use tonic::Status;

pub struct Registry<G: RoomCodeGenerator + Send + 'static> {
    servers: DashMap<ServerId, Server<G>>,
    rooms: DashMap<RoomCode, Room>,
    server_id_count: AtomicU32,
    room_code_generator: Arc<Mutex<G>>,
}

impl<G: RoomCodeGenerator + Send + 'static> Registry<G> {
    pub async fn register_server(&self, registration: ServerRegistrationRequest, tx: Sender<Result<RegistryToServerMessage, Status>>, registry: Arc<Registry<G>>) -> (ServerId, ServerRegistrationResponse, Arc<ServerRunner<G>>) {
        let server_id: ServerId = self.server_id_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst).into();
        let runner = Arc::new(ServerRunner::new(server_id, registration.description, tx, registry));
        let mut server = Server::new(runner.clone(), registration.address, registration.capacity, HashSet::new());

        let mut conflicts: Vec<RoomCodeChange> = Vec::new();

        for room in registration.rooms {
            let room_code = if let Some(room_code) = room.code {
                room_code
            } else {
                continue
            };

            match self.rooms.entry(room_code) {
                Entry::Occupied(_) => {
                    let new_code = self.new_room_code().await;
                    conflicts.push(RoomCodeChange {
                        before: Some(room_code),
                        after: Some(new_code),
                    });
                    server.add_room(new_code);
                }
                Entry::Vacant(entry) => {
                    entry.insert(Room::new(room_code, server_id));
                    server.add_room(room_code);
                }
            };
        }

        self.servers.insert(server_id, server);

        (
            server_id,
            ServerRegistrationResponse {
                server_id: Some(server_id),
                conflicts,
            },
            runner
        )
    }

    pub fn remove_server(&self, server_id: ServerId) {
        if let Some((_, server)) = self.servers.remove(&server_id) {
            for room in server.rooms {
                self.rooms.remove(&room);
            }
        }
    }

    pub fn new(room_code_generator: G) -> Self {
        Self {
            servers: DashMap::new(),
            rooms: DashMap::new(),
            server_id_count: AtomicU32::new(0),
            room_code_generator: Arc::new(Mutex::new(room_code_generator)),
        }
    }

    fn get_server_with_capacity(&self) -> Option<RefMulti<ServerId, Server<G>>> {
        self.servers.iter()
            .filter(|server| server.has_capacity())
            .min_by_key(|server| server.room_count())
    }

    async fn new_room_code(&self) -> RoomCode {
        let mut generator = self.room_code_generator.lock().await;
        let code = generator.new_code();

        if !generator.has_available_code() {
            let generator = self.room_code_generator.clone();
            tokio::spawn(async move {
                let mut generator = generator.lock().await;
                if !generator.has_available_code() {
                    generator.create_next_collection();
                }
            });
        }
        code
    }
}

