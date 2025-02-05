use crate::proto::registry::{registry_to_server_message, registry_to_server_request, registry_to_server_response, RegistryToServerMessage, RegistryToServerRequest, RegistryToServerResponse};

pub trait RegistryToServerMessagePack {
    fn pack(self, request_id: Option<u64>) -> RegistryToServerMessage;
}

impl RegistryToServerMessagePack for registry_to_server_response::Response {
    fn pack(self, request_id: Option<u64>) -> RegistryToServerMessage {
        RegistryToServerMessage {
            request_id,
            message: Some(registry_to_server_message::Message::Response(RegistryToServerResponse {
                response: Some(self),
            })),
        }
    }
}

impl RegistryToServerMessagePack for registry_to_server_request::Request {
    fn pack(self, request_id: Option<u64>) -> RegistryToServerMessage {
        RegistryToServerMessage {
            request_id,
            message: Some(registry_to_server_message::Message::Request(RegistryToServerRequest {
                request: Some(self),
            })),
        }
    }
}