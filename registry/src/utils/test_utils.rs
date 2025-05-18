use crate::proto::registry::{server_to_registry_message, server_to_registry_request, server_to_registry_response, ServerRegistrationRequest, ServerToRegistryMessage, ServerToRegistryRequest, ServerToRegistryResponse};

pub trait ServerToRegistryMessagePack {
    fn pack(self, request_id: Option<u64>) -> ServerToRegistryMessage;
}

impl ServerToRegistryMessagePack for server_to_registry_request::Request {
    fn pack(self, request_id: Option<u64>) -> ServerToRegistryMessage {
        ServerToRegistryMessage {
            request_id,
            message: Some(server_to_registry_message::Message::Request(ServerToRegistryRequest {
                request: Some(self),
            })),
        }
    }
}

impl ServerToRegistryMessagePack for server_to_registry_response::Response {
    fn pack(self, request_id: Option<u64>) -> ServerToRegistryMessage {
        ServerToRegistryMessage {
            request_id,
            message: Some(server_to_registry_message::Message::Response(ServerToRegistryResponse {
                response: Some(self),
            })),
        }
    }
}

impl ServerToRegistryMessagePack for ServerRegistrationRequest {
    fn pack(self, request_id: Option<u64>) -> ServerToRegistryMessage {
        server_to_registry_request::Request::Registration(self).pack(request_id)
    }
}