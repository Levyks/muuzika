use crate::codes::RoomCodeGenerator;
use crate::errors::RequestError;
use crate::proto::common::RoomCode;
use crate::proto::registry::{create_room_in_server_error, CreateRoomRequest, CreateRoomResponse, RegistryToServerMessage, RoomCodeChange, RoomRegistryDefinition, ServerId, ServerIdentifier, ServerRegistrationRequest, ServerRegistrationSuccess};
use crate::room::Room;
use crate::server::{Server, ServerRunner};
use dashmap::mapref::multiple::RefMulti;
use dashmap::{DashMap, DashSet, Entry};
use nanoid::nanoid;
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
    pub fn new(room_code_generator: G) -> Self {
        Self {
            servers: DashMap::new(),
            rooms: DashMap::new(),
            server_id_count: AtomicU32::new(0),
            room_code_generator: Arc::new(Mutex::new(room_code_generator)),
        }
    }

    pub async fn register_server(&self, registration: ServerRegistrationRequest, tx: Sender<Result<RegistryToServerMessage, Status>>, registry: Arc<Registry<G>>) -> (ServerId, ServerRegistrationSuccess, Arc<ServerRunner<G>>) {
        let server_id: ServerId = self.server_id_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst).into();
        let runner = Arc::new(ServerRunner::new(server_id, registration.description, tx, registry));
        let server = Server::new(server_id, runner.clone(), registration.address, registration.capacity, DashSet::with_capacity(registration.rooms.len()));

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

                    let room = Room::new(new_code, server.id);
                    self.rooms.insert(new_code, room);
                    server.add_room(new_code);
                }
                Entry::Vacant(entry) => {
                    let room = Room::new(room_code, server.id);
                    entry.insert(room);
                    server.add_room(room_code);
                }
            };
        }

        self.servers.insert(server_id, server);

        (
            server_id,
            ServerRegistrationSuccess {
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

    pub async fn create_room(&self, request: CreateRoomRequest) -> Result<CreateRoomResponse, Status> {
        let log_id = nanoid!();
        log::debug!("[{}] Creating room with request: {:?}", log_id, request);

        let mut attempt = 0;
        static MAX_ATTEMPTS: u32 = 3;
        loop {
            attempt += 1;

            let server = self
                .get_server_with_capacity()
                .ok_or(Status::resource_exhausted("No servers available"))?;
            log::debug!("[{}] Selected server: {}", log_id, server.value());

            let room_code = self.new_room_code().await;
            log::debug!("[{}] Generated room code: {}", log_id, room_code.code);

            match server.create_room(room_code, request.clone()).await {
                Ok(_) => {
                    log::debug!("[{}] Room {} created in server {}", log_id, room_code, server.id);

                    let room = Room::new(room_code, server.id);
                    self.rooms.insert(room_code, room);
                    server.add_room(room_code);

                    return Ok(CreateRoomResponse {
                        code: Some(room_code),
                        server: Some(ServerIdentifier {
                            address: server.address.clone(),
                        }),
                    });
                }
                Err(e) => {
                    log::warn!("[{}] Failed to create room in attempt {}: {}", log_id, attempt, e);

                    let error_message = if attempt >= MAX_ATTEMPTS {
                        Some(format!("Failed to create room: {}", e))
                    } else {
                        None
                    };

                    // Server already has a room with this code, let's add it to our registry and try again with a new code
                    if let RequestError::ErrorResponse(create_room_in_server_error::Error::RoomAlreadyExists(room)) = e {
                        log::warn!("[{}] Room {} already exists in server {}, registering it", log_id, room_code, server.value());
                        let room = Room::new(room_code, server.id);
                        self.rooms.insert(room_code, room);
                        server.add_room(room_code);
                    } else {
                        self.return_room_code(room_code).await;
                    }

                    if let Some(message) = error_message {
                        log::warn!("[{}] MAX_ATTEMPTS reached, failing request with: {}", log_id, message);
                        return Err(Status::internal(message));
                    }
                }
            }
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
            log::warn!("Code generator exhausted, scheduling new collection creation");
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

    async fn return_room_code(&self, code: RoomCode) {
        log::debug!("Returning room code {} to generator", code.code);
        self.room_code_generator.lock().await.return_code(code);
        log::debug!("Returned room code {} to generator", code.code);
    }
}

