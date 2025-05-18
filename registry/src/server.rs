use std::collections::{HashMap, HashSet};
use std::fmt::Display;
use std::hash::Hash;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;
use tokio::sync::mpsc::Sender;
use tokio::sync::{oneshot, Mutex, RwLock};
use tonic::Status;
use tonic::codegen::tokio_stream::{Stream, StreamExt};
use crate::proto::common::RoomCode;
use crate::proto::registry::{server_to_registry_message, RegistryToServerMessage, ServerId, ServerLoadInfo, ServerToRegistryMessage, ServerToRegistryRequest, ServerToRegistryResponse};
use crate::registry::Registry;

pub struct Server {
    pub id: ServerId,
    pub rooms: HashSet<RoomCode>,
    address: String,
    load_info: ServerLoadInfo,
    runner: Arc<ServerRunner>
}

impl Server {
    pub fn new(id: ServerId, address: String, load_info: ServerLoadInfo, rooms: HashSet<RoomCode>, runner: Arc<ServerRunner>) -> Self {
        Self {
            id,
            address,
            load_info,
            rooms,
            runner
        }
    }
}

pub struct ServerRunner {
    pub id: ServerId,
    registry: Arc<RwLock<Registry>>,
    pending: Arc<Mutex<HashMap<u64, oneshot::Sender<ServerToRegistryResponse>>>>,
    request_id_counter: AtomicU64,
    tx: Sender<Result<RegistryToServerMessage, Status>>,
}

impl ServerRunner {
    pub fn new(id: ServerId, tx: Sender<Result<RegistryToServerMessage, Status>>, registry: Arc<RwLock<Registry>>) -> Self {
        Self {
            id,
            pending: Arc::new(Mutex::new(HashMap::new())),
            request_id_counter: AtomicU64::new(0),
            tx,
            registry
        }
    }

    pub async fn add_pending(&self) -> (u64, oneshot::Receiver<ServerToRegistryResponse>) {
        let (tx, rx) = oneshot::channel();
        let request_id = self.request_id_counter.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        self.pending.lock().await.insert(request_id, tx);
        (request_id, rx)
    }

    pub async fn handle_message(&self, message: ServerToRegistryMessage) {
        match message.message {
            Some(server_to_registry_message::Message::Response(response)) =>
                self.handle_response(message.request_id, response).await,
            Some(server_to_registry_message::Message::Request(request)) =>
                self.handle_request(message.request_id, request),
            None => (),
        }
    }

    async fn handle_response(&self, request_id: Option<u64>, response: ServerToRegistryResponse) {
        let request_id = if let Some(request_id) = request_id {
            request_id
        } else {
            log::warn!("[{self}] Got response without request ID");
            return;
        };
        
        let tx = if let Some(tx) = self.pending.lock().await.remove(&request_id) {
            tx
        } else {
            log::warn!("[{self}] Could not find pending request with ID {request_id}");
            return;
        };

        if let Err(e) = tx.send(response) {
            log::warn!("[{self}] Failed to resolve pending request with ID {request_id}: {e:?}");
        }
    }

    fn handle_request(&self, request_id: Option<u64>, request: ServerToRegistryRequest) {
        log::debug!("[{self}] Got request: {request:?}");
    }
    
    pub async fn run<S>(&self, mut stream: S)
    where
        S: Stream<Item = ServerToRegistryMessage> + Send + Unpin+ 'static
    {
        log::debug!("[{self}] Starting input stream handler loop");
        while let Some(message) = stream.next().await {
            self.handle_message(message).await
        }
        log::debug!("[{self}] Input stream ended");
        self.remove_self().await;
    }

    pub async fn remove_self(&self) {
        self.registry.write().await.remove_server(&self.id);
    }
}

impl Eq for ServerId {}

impl Hash for ServerId {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.server_id.hash(state);
    }
}

impl Display for ServerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.server_id)
    }
}

impl Display for ServerRunner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ServerRunner({})", self.id)
    }
}