use crate::proto::registry::{registry_to_server_message, registry_to_server_request, registry_to_server_response, RegistryToServerMessage, RegistryToServerRequest, RegistryToServerResponse};
use prost::bytes::Bytes;
use prost::Message;

pub fn into_any(message: impl Message, type_url: &str) -> prost_types::Any {
    let mut buf = Vec::new();
    message.encode(&mut buf).unwrap();

    prost_types::Any {
        type_url: type_url.to_string(),
        value: buf,
    }
}

pub fn into_any_bytes(message: impl Message, type_url: &str) -> Bytes {
    let mut buf = Vec::new();
    into_any(message, type_url).encode(&mut buf).unwrap();
    buf.into()
}

pub trait RegistryToServerMessagePack {
    fn pack(self, request_id: u64) -> RegistryToServerMessage;
}

impl RegistryToServerMessagePack for registry_to_server_response::Response {
    fn pack(self, request_id: u64) -> RegistryToServerMessage {
        RegistryToServerMessage {
            request_id: Some(request_id),
            message: Some(registry_to_server_message::Message::Response(RegistryToServerResponse {
                response: Some(self),
            })),
        }
    }
}

impl RegistryToServerMessagePack for registry_to_server_request::Request {
    fn pack(self, request_id: u64) -> RegistryToServerMessage {
        RegistryToServerMessage {
            request_id: Some(request_id),
            message: Some(registry_to_server_message::Message::Request(RegistryToServerRequest {
                request: Some(self),
            })),
        }
    }
}

