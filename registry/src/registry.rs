use crate::errors::RequestError;
use crate::generator::RoomCodeGenerator;
use crate::proto::common::RoomCode;
use crate::proto::registry::{create_room_error, create_room_in_server_error, join_room_error, server_registration_error, CodeWithUsernameAndPassword, JoinRoomResponse, RoomCodeChange, ServerId, ServerInfo, ServerRegistrationError, ServerRegistrationSuccess, UsernameAndPassword};
use crate::server::Server;
use nanoid::nanoid;
use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt::Display;
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct Registry {
    servers: HashMap<ServerId, Server>,
    rooms: HashMap<RoomCode, ServerId>,
    room_code_generator: Box<dyn RoomCodeGenerator + Send + Sync>,
}

impl Registry {
    pub fn new(room_code_generator: impl RoomCodeGenerator + Send + Sync + 'static) -> Self {
        Self {
            servers: HashMap::new(),
            rooms: HashMap::new(),
            room_code_generator: Box::new(room_code_generator),
        }
    }

    pub fn get_server_info(&self, server_id: &ServerId) -> Option<ServerInfo> {
        self.servers.get(server_id).map(|server| server.info())
    }

    pub fn register_server(&mut self, mut server: Server) -> Result<ServerRegistrationSuccess, ServerRegistrationError> {
        if self.servers.contains_key(&server.id) {
            return Err(ServerRegistrationError {
                error: Some(server_registration_error::Error::IdAlreadyExists(()))
            });
        }

        let mut conflicts: Vec<RoomCodeChange> = Vec::new();

        server.rooms = server.rooms.into_iter()
            .map(|room_code| {
                match self.rooms.entry(room_code) {
                    Entry::Occupied(_) => {
                        let new_code = self.room_code_generator.new_code();
                        conflicts.push(RoomCodeChange {
                            before: Some(room_code),
                            after: Some(new_code),
                        });
                        self.rooms.insert(new_code, server.id);
                        new_code
                    }
                    Entry::Vacant(entry) => {
                        entry.insert(server.id);
                        room_code
                    }
                }
            })
            .collect();

        let server_id = server.id.clone();

        self.servers.insert(server.id, server);

        Ok(ServerRegistrationSuccess {
            server_id: Some(server_id),
            conflicts,
        })
    }

    pub async fn create_room(self_lock: &Arc<RwLock<Self>>, request: UsernameAndPassword) -> Result<JoinRoomResponse, create_room_error::Error> {
        let log_id = nanoid!();
        log::debug!("[{log_id}] Creating room with request: {request:?}");

        let mut attempt = 0;
        static MAX_ATTEMPTS: u32 = 3;

        loop {
            attempt += 1;
            let (server_info, runner, room_code) = {
                let mut registry = self_lock.write().await;
                let server = registry.get_best_suited_server().ok_or(create_room_error::Error::NoServerAvailable(()))?;
                (server.info(), server.runner.clone(), registry.room_code_generator.new_code())
            };
            log::debug!("[{log_id}] Attempting to create room on server: {server_info:?} with room code: {room_code}");

            let request = CodeWithUsernameAndPassword {
                code: Some(room_code.clone()),
                username_and_password: Some(request.clone()),
            };

            let token = match runner.create_room(request).await {
                Ok(token) => token,
                Err(err) => {
                    log::warn!("[{log_id}] Failed to create room in attempt {attempt}: {err:?}");

                    let mut registry = self_lock.write().await;
                    if let RequestError::ErrorResponse(create_room_in_server_error::Error::RoomAlreadyExists(_)) = err {
                        registry.add_room(&room_code, &runner.id);
                    } else {
                        registry.room_code_generator.return_code(room_code.clone());
                    }

                    if attempt >= MAX_ATTEMPTS {
                        return Err(create_room_error::Error::InternalError(()));
                    }

                    continue;
                }
            };

            self_lock.write().await.add_room(&room_code, &runner.id);

            return Ok(JoinRoomResponse {
                server: Some(server_info),
                token: Some(token),
                code: Some(room_code),
            });
        };
    }

    pub async fn join_room(self_lock: &Arc<RwLock<Self>>, room_code: RoomCode, request: CodeWithUsernameAndPassword) -> Result<JoinRoomResponse, join_room_error::Error> {
        let log_id = nanoid!();
        log::debug!("[{log_id}] Joining room with request: {request:?}");

        let (server_info, runner) = {
            let mut registry = self_lock.write().await;
            let server_id = registry.rooms.get(&room_code).ok_or(join_room_error::Error::RoomNotFound(()))?;
            match registry.servers.get(server_id) {
                Some(server) => (server.info(), server.runner.clone()),
                None => {
                    log::warn!("[{log_id}] Could not find server {server_id}, removing room {room_code}");
                    registry.remove_room(&room_code);
                    return Err(join_room_error::Error::InternalError(()));
                }
            }
        };

        let token = match runner.join_room(request).await {
            Ok(token) => token,
            Err(RequestError::ErrorResponse(join_room_error::Error::RoomNotFound(_))) => {
                log::warn!("[{log_id}] Room {room_code} not found on server {}, removing from registry", runner.id);
                self_lock.write().await.rooms.remove(&room_code);
                Err(join_room_error::Error::RoomNotFound(()))?
            }
            Err(RequestError::ErrorResponse(err)) => Err(err)?,
            _ => Err(join_room_error::Error::InternalError(()))?,
        };

        Ok(JoinRoomResponse {
            server: Some(server_info),
            token: Some(token),
            code: Some(room_code),
        })
    }

    fn add_room(&mut self, room_code: &RoomCode, server_id: &ServerId) {
        self.rooms.insert(room_code.clone(), server_id.clone());
        if let Some(server) = self.servers.get_mut(server_id) {
            server.rooms.insert(room_code.clone());
        }
    }

    fn remove_room(&mut self, room_code: &RoomCode) {
        self.room_code_generator.return_code(room_code.clone());
        if let Some(server_id) = self.rooms.remove(room_code) {
            if let Some(server) = self.servers.get_mut(&server_id) {
                server.rooms.remove(room_code);
            }
        }
    }

    pub fn remove_server(&mut self, server_id: &ServerId) {
        if let Some(server) = self.servers.remove(server_id) {
            for room_code in server.rooms {
                self.rooms.remove(&room_code);
                self.room_code_generator.return_code(room_code);
            }
        }
    }

    fn get_best_suited_server(&self) -> Option<&Server> {
        self.servers.values()
            .filter(|server| server.load_info.accepting_new_rooms)
            .min_by(|a, b| {
                a.load_info.load.partial_cmp(&b.load_info.load).unwrap_or(std::cmp::Ordering::Equal)
            })
    }
}

impl Eq for RoomCode {}

impl Hash for RoomCode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.code.hash(state);
    }
}

impl Display for RoomCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code)
    }
}