use crate::codes::RoomCodeGenerator;
use crate::proto::common::RoomCode;
use crate::proto::registry::{server_to_registry_message, RegistryToServerMessage, ServerId, ServerToRegistryMessage, ServerToRegistryRequest, ServerToRegistryResponse};
use crate::registry::Registry;
use dashmap::DashMap;
use std::collections::HashSet;
use std::fmt::Display;
use std::hash::Hash;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;
use tonic::Status;

pub struct Server<G: RoomCodeGenerator + Send + 'static> {
    runner: Arc<ServerRunner<G>>,
    address: String,
    capacity: Option<u32>,
    pub(crate) rooms: HashSet<RoomCode>,
}

impl<G: RoomCodeGenerator + Send + 'static> Server<G> {
    pub fn new(runner: Arc<ServerRunner<G>>, address: String, capacity: Option<u32>, rooms: HashSet<RoomCode>) -> Self {
        Self {
            runner,
            address,
            capacity,
            rooms,
        }
    }

    pub fn add_room(&mut self, room: RoomCode) -> bool {
        self.rooms.insert(room)
    }

    pub fn remove_room(&mut self, room: &RoomCode) -> bool {
        self.rooms.remove(room)
    }

    pub fn has_capacity(&self) -> bool {
        self.capacity.map_or(true, |c| self.rooms.len() < c as usize)
    }

    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }
}

pub struct ServerRunner<G: RoomCodeGenerator + Send + 'static> {
    pub id: ServerId,
    description: String,
    pending: DashMap<u64, oneshot::Sender<ServerToRegistryResponse>>,
    tx: Sender<Result<RegistryToServerMessage, Status>>,
    registry: Arc<Registry<G>>,
}

impl<G: RoomCodeGenerator + Send + 'static> ServerRunner<G> {
    pub fn new(id: ServerId, description: String, tx: Sender<Result<RegistryToServerMessage, Status>>, registry: Arc<Registry<G>>) -> Self {
        Self {
            id,
            description,
            tx,
            pending: DashMap::new(),
            registry,
        }
    }

    pub fn remove_self(&self) {
        self.registry.remove_server(self.id);
    }

    pub async fn handle_message(&self, message: ServerToRegistryMessage) {
        match message.message {
            Some(server_to_registry_message::Message::Response(response)) =>
                self.handle_response(message.request_id, response),
            Some(server_to_registry_message::Message::Request(request)) =>
                self.handle_request(message.request_id, request),
            None => (),
        }
    }

    fn handle_response(&self, request_id: Option<u64>, response: ServerToRegistryResponse) {
        let request_id = if let Some(request_id) = request_id {
            request_id
        } else {
            log::warn!("Server {} sent a response without a request ID", self);
            return;
        };

        let tx = if let Some((_, tx)) = self.pending.remove(&request_id) {
            tx
        } else {
            log::warn!("Could not find pending request {} for server {}", request_id, self);
            return;
        };

        if let Err(e) = tx.send(response) {
            log::warn!("Failed to respond to request {} for server {}: {:?}", request_id, self, e);
        }
    }

    fn handle_request(&self, request_id: Option<u64>, response: ServerToRegistryRequest) {
        log::debug!("Server {} sent a request: {:?}", self, response);
    }
}

impl<G: RoomCodeGenerator + Send + 'static> Display for ServerRunner<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.id, self.description)
    }
}

impl Display for ServerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.server_id)
    }
}

impl From<u32> for ServerId {
    fn from(server_id: u32) -> Self {
        Self { server_id }
    }
}

impl Eq for ServerId {}

impl Hash for ServerId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.server_id.hash(state);
    }
}