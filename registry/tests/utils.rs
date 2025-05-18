use muuzika_registry::proto::registry::registry_service_client::RegistryServiceClient;
use muuzika_registry::proto::registry::{create_room_in_server_response, registry_to_server_message, registry_to_server_request, registry_to_server_response, server_to_registry_message, server_to_registry_request, server_to_registry_response, CreateRoomInServerResponse, RegistryToServerMessage, RoomToken, ServerRegistrationRequest, ServerRegistrationSuccess, ServerToRegistryMessage, ServerToRegistryRequest, ServerToRegistryResponse};
use muuzika_registry::{serve_with_shutdown_and_codes, Options};
use prost::Message;
use tokio::net::TcpListener;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use tonic::transport::{Channel, Endpoint};
use tonic::{Request, Status, Streaming};

async fn setup_test_server_base(options: Option<Options>, codes: Option<Vec<u32>>) -> (tokio::sync::oneshot::Sender<()>, Endpoint) {
    let options = options.unwrap_or_else(|| Options {
        registration_timeout: std::time::Duration::from_millis(100),
    });
    let (shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move {
        serve_with_shutdown_and_codes(listener, options, shutdown_rx, codes).await;
    });
    (shutdown_tx, format!("http://{}", addr).try_into().unwrap())
}

pub async fn setup_test_server_with_options(options: Options) -> (tokio::sync::oneshot::Sender<()>, Endpoint) {
    setup_test_server_base(Some(options), None).await
}

pub async fn setup_test_server_with_codes(codes: Vec<u32>) -> (tokio::sync::oneshot::Sender<()>, Endpoint) {
    setup_test_server_base(None, Some(codes)).await
}

pub async fn setup_test_server_with_options_and_codes(options: Options, codes: Vec<u32>) -> (tokio::sync::oneshot::Sender<()>, Endpoint) {
    setup_test_server_base(Some(options), Some(codes)).await
}

pub async fn setup_test_server() -> (tokio::sync::oneshot::Sender<()>, Endpoint) {
    setup_test_server_base(None, None).await
}

pub fn make_request() -> (Sender<ServerToRegistryMessage>, Request<ReceiverStream<ServerToRegistryMessage>>,) {
    let (tx, rx) = mpsc::channel::<ServerToRegistryMessage>(1);
    let stream = ReceiverStream::new(rx);
    (tx, Request::new(stream))
}

pub async fn register_server(
    client: &mut RegistryServiceClient<Channel>,
    registration: ServerRegistrationRequest,
) -> Result<(ServerRegistrationSuccess, Sender<ServerToRegistryMessage>, Streaming<RegistryToServerMessage>), Status> {
    let (request_tx, request) = make_request();

    let registration_request_id = 42u64;
    request_tx.send(registration.pack_opt(Some(registration_request_id))).await.unwrap();

    let mut response_stream = client.register_server(request)
        .await?
        .into_inner();

    let registration_message = loop {
        if let Some(message) = response_stream.message().await? {
            if message.request_id == Some(registration_request_id) {
                break message;
            }
        }
    };

    Ok((assert_message_is_registration_success(registration_message), request_tx, response_stream))
}

pub async fn register_server_and_start_mock_thread(
    client: &mut RegistryServiceClient<Channel>,
    registration: ServerRegistrationRequest,
) -> Result<(ServerRegistrationSuccess, Sender<ServerToRegistryMessage>), Status> {
    let (registration, request_tx, response_stream) = register_server(client, registration).await?;

    let thread_tx = request_tx.clone();
    tokio::spawn(async move {
        mock_server(thread_tx, response_stream).await;
    });

    Ok((registration, request_tx))
}

async fn mock_server(
    tx: Sender<ServerToRegistryMessage>,
    mut response_stream: Streaming<RegistryToServerMessage>,
) {
    while let Some(message) = response_stream.message().await.unwrap() {
        if let Some(registry_to_server_message::Message::Request(request)) = message.message {
            match request.request {
                Some(registry_to_server_request::Request::CreateRoom(_)) => {
                    let response = server_to_registry_response::Response::CreateRoom(CreateRoomInServerResponse {
                        response: Some(create_room_in_server_response::Response::Success(RoomToken {
                            token: "test_token".to_string(),
                        })),
                    });
                    tx.send(response.pack_opt(message.request_id)).await.unwrap();
                }
                _ => {}
            }
        }
    }
}

pub fn assert_message_is_registration_success(
    message: RegistryToServerMessage
) -> ServerRegistrationSuccess {
    match message.message {
        Some(registry_to_server_message::Message::Response(response)) => match response.response {
            Some(registry_to_server_response::Response::RegistrationSuccess(success)) => success,
            _ => panic!("Expected a server registration response"),
        },
        _ => panic!("Expected a response message"),
    }
}

pub trait ServerToRegistryMessagePack {
    fn pack_opt(self, request_id: Option<u64>) -> ServerToRegistryMessage;
    fn pack(self, request_id: u64) -> ServerToRegistryMessage;
}

impl ServerToRegistryMessagePack for server_to_registry_request::Request {
    fn pack_opt(self, request_id: Option<u64>) -> ServerToRegistryMessage {
        ServerToRegistryMessage {
            request_id,
            message: Some(server_to_registry_message::Message::Request(ServerToRegistryRequest {
                request: Some(self),
            })),
        }
    }

    fn pack(self, request_id: u64) -> ServerToRegistryMessage {
        self.pack_opt(Some(request_id))
    }
}

impl ServerToRegistryMessagePack for server_to_registry_response::Response {
    fn pack_opt(self, request_id: Option<u64>) -> ServerToRegistryMessage {
        ServerToRegistryMessage {
            request_id,
            message: Some(server_to_registry_message::Message::Response(ServerToRegistryResponse {
                response: Some(self),
            })),
        }
    }

    fn pack(self, request_id: u64) -> ServerToRegistryMessage {
        self.pack_opt(Some(request_id))
    }
}

impl ServerToRegistryMessagePack for ServerRegistrationRequest {
    fn pack_opt(self, request_id: Option<u64>) -> ServerToRegistryMessage {
        server_to_registry_request::Request::Registration(self).pack_opt(request_id)
    }

    fn pack(self, request_id: u64) -> ServerToRegistryMessage {
        self.pack_opt(Some(request_id))
    }
}

pub fn decode_bytes_any<T>(bytes: &[u8], type_url: &str) -> T
where
    T: prost::Message + Default,
{
    let any = prost_types::Any::decode(bytes).expect("Failed to decode any message");
    assert_eq!(any.type_url, type_url, "Type URL mismatch");
    T::decode(any.value.as_ref()).expect("Failed to decode message")
}