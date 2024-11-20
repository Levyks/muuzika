use sqlx::{Pool, Sqlite};

pub struct State {
    // TODO: this RwLock might be a bottleneck later on, but I won't prematurely optimize
    // pub rooms: Arc<RwLock<HashMap<RoomCode, String>>>,
    // pub codes: Arc<RwLock<VecDeque<RoomCode>>>,
    // pub servers: Arc<RwLock<HashMap<ServerId, Arc<RwLock<server_handler::Server>>>>>,
    pub db: Pool<Sqlite>,
}
