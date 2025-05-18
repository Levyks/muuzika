use crate::proto::common::RoomCode;
use crate::proto::registry::{ServerId, ServerLoadInfo};

#[derive(Debug)]
pub struct ExtractedServerRegistrationRequest {
    pub request_id: u64,
    pub server_id: ServerId,
    pub address: String,
    pub rooms: Vec<RoomCode>,
    pub load_info: ServerLoadInfo,
}