use std::collections::hash_map::Entry;
use std::collections::HashMap;
use crate::generator::RoomCodeGenerator;
use crate::proto::common::RoomCode;
use crate::proto::registry::{server_registration_error, RoomCodeChange, ServerId, ServerRegistrationError, ServerRegistrationSuccess};
use crate::server::Server;

pub struct Registry {
    servers: HashMap<ServerId, Server>,
    rooms: HashMap<RoomCode, ServerId>,
    room_code_generator: Box<dyn RoomCodeGenerator + Send + Sync>,
}

impl Registry {
    pub fn new(room_code_generator: impl RoomCodeGenerator + Send + Sync +'static) -> Self {
        Self {
            servers: HashMap::new(),
            rooms: HashMap::new(),
            room_code_generator: Box::new(room_code_generator),
        }
    }

    pub fn register_server(&mut self, mut server: Server) -> Result<ServerRegistrationSuccess, ServerRegistrationError> {
        if (self.servers.contains_key(&server.id)) {
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
            conflicts
        })
    }
    
    pub fn remove_server(&mut self, server_id: &ServerId) {
        if let Some(server) = self.servers.remove(server_id) {
            for room_code in server.rooms {
                self.rooms.remove(&room_code);
                self.room_code_generator.return_code(room_code);
            }
        }
    }
}