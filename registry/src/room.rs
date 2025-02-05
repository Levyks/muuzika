use crate::proto::common::RoomCode;
use crate::proto::registry::ServerId;
use std::fmt::Display;
use std::hash::Hash;

pub struct Room {
    code: RoomCode,
    server_id: ServerId,
}

impl Room {
    pub fn new(code: RoomCode, server_id: ServerId) -> Self {
        Self {
            code,
            server_id,
        }
    }
}

impl Hash for RoomCode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.code.hash(state);
    }
}
impl Eq for RoomCode {}

impl Display for RoomCode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.code)
    }
}