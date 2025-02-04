use crate::proto::registry::{registry_to_server_message, registry_to_server_response, registry_to_server_success, RegistryToServerMessage, RegistryToServerResponse, RegistryToServerSuccess};

pub trait RegistryToServerMessagePack {
    fn pack(self, request_id: Option<u64>) -> RegistryToServerMessage;
}

impl RegistryToServerMessagePack for registry_to_server_success::Success {
    fn pack(self, request_id: Option<u64>) -> RegistryToServerMessage {
        RegistryToServerMessage {
            request_id,
            message: Some(registry_to_server_message::Message::Response(RegistryToServerResponse {
                response: Some(registry_to_server_response::Response::Success(
                    RegistryToServerSuccess {
                        success: Some(self),
                    },
                )),
            })),
        }
    }
}