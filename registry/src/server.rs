use crate::proto::common::RoomCode;
use crate::proto::registry::{
    server_to_registry_message, CreateRoomRequest, RegistryToServerMessage, RoomCodeChange,
    RoomRegistryDefinition, ServerId, ServerRegistrationRequest, ServerRegistrationResponse,
    ServerToRegistryMessage,
};
use crate::state::{Room, State};
use std::hash::{Hash, Hasher};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tonic::{Status, Streaming};

impl Eq for ServerId {}

impl Hash for ServerId {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.server_id.hash(state);
    }
}

pub struct Server {
    state: Arc<State>,
    id: ServerId,
    callsign: String,
    address: String,
    capacity: Option<u32>,
    r#type: Option<String>,
    tx: mpsc::Sender<Result<RegistryToServerMessage, Status>>,
    request_id_counter: AtomicU64,
    rooms: Vec<Arc<Room>>,
}

pub async fn create_and_insert(
    state: Arc<State>,
    request: ServerRegistrationRequest,
    tx: mpsc::Sender<Result<RegistryToServerMessage, Status>>,
) -> (Arc<Server>, ServerRegistrationResponse) {
    let id = ServerId {
        server_id: state.server_id_counter.fetch_add(1, Ordering::SeqCst),
    };

    let mut rooms: Vec<Arc<Room>> = vec![];
    let mut conflicts: Vec<RoomCodeChange> = vec![];

    for mut room in request.rooms {
        let mut code = if let Some(code) = room.code {
            code
        } else {
            continue;
        };

        if state.rooms.contains_key(&code) {
            let after = state.get_room_code().await;
            room.code = Some(after);
            conflicts.push(RoomCodeChange {
                before: Some(code),
                after: Some(after),
            });
            code = after;
        }

        let room = Arc::new(Room::from_def(room, code.clone(), id.clone()));

        state.rooms.insert(code.clone(), room.clone());
        rooms.push(room);
    }

    let server = Server::new(
        state.clone(),
        id.clone(),
        request.callsign,
        request.address,
        request.capacity,
        request.r#type,
        rooms,
        tx,
    );

    state.servers.insert(id.clone(), server.clone());

    (
        server,
        ServerRegistrationResponse {
            server_id: Some(id),
            conflicts,
        },
    )
}

pub async fn remove_server(server: &Server) {
    if let Some(_) = server.state.servers.remove(&server.id) {
        for room in server.rooms.iter() {
            server.state.rooms.remove(&room.code);
        }
    }
}

impl Server {
    pub fn new(
        state: Arc<State>,
        id: ServerId,
        callsign: String,
        address: String,
        capacity: Option<u32>,
        r#type: Option<String>,
        rooms: Vec<Arc<Room>>,
        tx: mpsc::Sender<Result<RegistryToServerMessage, Status>>,
    ) -> Self {
        Self {
            state,
            id,
            callsign,
            address,
            capacity,
            r#type,
            tx,
            rooms,
            request_id_counter: AtomicU64::new(0),
        }
    }

    pub fn add_room(&mut self, room: Arc<Room>) {
        self.state.rooms.insert(room.code.clone(), room.clone());
        self.rooms.push(room);
    }

    pub async fn handle_stream(&self, stream: &mut Streaming<ServerToRegistryMessage>) {
        while let Ok(Some(message)) = stream.message().await {
            log::debug!("Received message: {:?}", message);

            let request_id = if let Some(request_id) = message.request_id {
                request_id
            } else {
                continue;
            };

            let message = if let Some(message) = message.message {
                message
            } else {
                continue;
            };

            match message {
                _ => {
                    log::debug!("No op for now {:?}", message);
                }
            }
        }
    }

    pub async fn create_room(&self, request: CreateRoomRequest) -> anyhow::Result<()> {
        Ok(())
    }
}
