use crate::codes::RoomCodeGenerator;
use crate::errors::{RequestError, RequestResult};
use crate::proto::common::RoomCode;
use crate::proto::registry::{create_room_in_server_error, create_room_in_server_response, registry_to_server_request, server_to_registry_message, server_to_registry_response, CreateRoomInServerRequest, CreateRoomRequest, RegistryToServerMessage, ServerId, ServerToRegistryMessage, ServerToRegistryRequest, ServerToRegistryResponse};
use crate::registry::Registry;
use crate::{handle_response_complete, send_request};
use dashmap::{DashMap, DashSet};
use std::fmt::Display;
use std::hash::Hash;
use std::sync::atomic::AtomicU64;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tokio::sync::oneshot;
use tonic::Status;

pub struct Server<G: RoomCodeGenerator + Send + 'static> {
    pub(crate) id: ServerId,
    runner: Arc<ServerRunner<G>>,
    pub(crate) address: String,
    capacity: Option<u32>,
    pub(crate) rooms: DashSet<RoomCode>,
}

impl<G: RoomCodeGenerator + Send + 'static> Server<G> {
    pub fn new(id: ServerId, runner: Arc<ServerRunner<G>>, address: String, capacity: Option<u32>, rooms: DashSet<RoomCode>) -> Self {
        Self {
            id,
            runner,
            address,
            capacity,
            rooms,
        }
    }

    pub fn add_room(&self, room: RoomCode) -> bool {
        self.rooms.insert(room)
    }

    pub fn remove_room(&self, room: &RoomCode) -> Option<RoomCode> {
        self.rooms.remove(room)
    }

    pub fn has_capacity(&self) -> bool {
        self.capacity.map_or(true, |c| self.rooms.len() < c as usize)
    }

    pub fn room_count(&self) -> usize {
        self.rooms.len()
    }

    pub async fn create_room(&self, code: RoomCode, request: CreateRoomRequest) -> RequestResult<(), create_room_in_server_error::Error> {
        self.runner.create_room(code, request).await
    }
}

pub struct ServerRunner<G: RoomCodeGenerator + Send + 'static> {
    pub id: ServerId,
    description: String,
    pending: DashMap<u64, oneshot::Sender<ServerToRegistryResponse>>,
    registry: Arc<Registry<G>>,
    request_id_counter: AtomicU64,
    tx: Sender<Result<RegistryToServerMessage, Status>>,
}

impl<G: RoomCodeGenerator + Send + 'static> ServerRunner<G> {
    pub fn new(id: ServerId, description: String, tx: Sender<Result<RegistryToServerMessage, Status>>, registry: Arc<Registry<G>>) -> Self {
        Self {
            id,
            description,
            tx,
            registry,
            pending: DashMap::new(),
            request_id_counter: AtomicU64::new(0),
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

    pub fn add_pending(&self) -> (u64, oneshot::Receiver<ServerToRegistryResponse>) {
        let (tx, rx) = oneshot::channel();
        let request_id = self.request_id_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.pending.insert(request_id, tx);
        (request_id, rx)
    }

    pub async fn create_room(&self, code: RoomCode, request: CreateRoomRequest) -> RequestResult<(), create_room_in_server_error::Error> {
        send_request!(
            self,
            registry_to_server_request::Request::CreateRoom(
                CreateRoomInServerRequest {
                    code: Some(code),
                    request: Some(request),
                }
            ),
            server_to_registry_response::Response::CreateRoom,
            create_room_in_server_response::Response::Success,
            create_room_in_server_response::Response::Error
        )
    }
}

impl<G: RoomCodeGenerator + Send + 'static> Display for ServerRunner<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} ({})", self.id, self.description)
    }
}

impl<G: RoomCodeGenerator + Send + 'static> Display for Server<G> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.runner.fmt(f)
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