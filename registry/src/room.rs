use std::hash::Hash;
use crate::proto::common::RoomCode;

impl Eq for RoomCode {}

impl Hash for RoomCode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.code.hash(state);
    }
}
