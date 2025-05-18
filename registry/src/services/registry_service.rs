use std::sync::Arc;
use nanoid::nanoid;
use tokio::sync::{mpsc, RwLock};
use crate::proto::registry::registry_service_server::{RegistryService, RegistryServiceServer};
use crate::proto::registry::{registry_to_server_response, server_registration_response, server_to_registry_message, server_to_registry_request, RegistryToServerMessage, ServerRegistrationResponse, ServerToRegistryMessage};
use crate::utils::stream::NotifiableReceiverStream;
use tonic::codegen::tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Status, Streaming};
use crate::messages::registration::ExtractedServerRegistrationRequest;
use crate::server::{Server, ServerRunner};
use crate::options::Options;
use crate::registry::Registry;
use crate::utils::packing::RegistryToServerMessagePack;

pub struct RegistryServiceImpl {
    registry: Arc<RwLock<Registry>>,
    options: Arc<Options>,
}

impl RegistryServiceImpl {
    pub fn new(registry: Arc<RwLock<Registry>>, options: Arc<Options>) -> Self {
        Self { registry, options }
    }

    pub fn server(registry: Arc<RwLock<Registry>>, options: Arc<Options>) -> RegistryServiceServer<Self> {
        RegistryServiceServer::new(Self::new(registry, options))
    }

    async fn register_server_inner<S>(
        &self,
        request: Request<S>,
    ) -> Result<Response<RegisterServerStream>, Status>
    where
        S: Stream<Item = Result<ServerToRegistryMessage, Status>> + Send + Unpin + 'static,
    {
        let log_id = nanoid!();

        let remote_addr = request.remote_addr().map(|a| a.to_string()).unwrap_or_default();
        log::debug!("[{}] Started registration request from address {}", log_id, remote_addr);

        let mut stream = request.into_inner();

        let registration = self.wait_for_registration(&mut stream).await
            .map_err(|e| {
                log::warn!("[{log_id}] Failed to get registration data: {e:?}");
                e
            })?;

        log::debug!("[{}] Received registration data: {:?}", log_id, registration);

        let (tx, rx) = mpsc::channel::<Result<RegistryToServerMessage, Status>>(1);

        let server_runner = Arc::new(ServerRunner::new(registration.server_id, tx.clone(), self.registry.clone()));
        let server = Server::new(
            registration.server_id,
            registration.address,
            registration.load_info,
            registration.rooms.into_iter().collect(),
            server_runner.clone()
        );

        let mut registry = self.registry.write().await;
        let response = registry.register_server(server)?;

        let message = registry_to_server_response::Response::ServerRegistration(
            ServerRegistrationResponse {
                response: Some(server_registration_response::Response::Success(response)),
            }
        ).pack(Some(registration.request_id));

        tx.send(Ok(message)).await.map_err(|_| {
            log::warn!("[{log_id}] Failed to send registration response");
            Status::internal("Failed to send response")
        })?;

        tokio::spawn(async move {
            server_runner.run(stream).await;
        });

        let server_id = registration.server_id;
        let registry = self.registry.clone();
        let stream = NotifiableReceiverStream::new(rx, move || {
            log::debug!("[{log_id}] Output stream for server {server_id} closed");
            tokio::spawn(async move {
                registry.write().await.remove_server(&server_id);
            });
        });

        Ok(Response::new(stream))
    }

    async fn wait_for_registration<S>(&self, stream: &mut S) -> Result<ExtractedServerRegistrationRequest, Status>
    where
        S: Stream<Item = Result<ServerToRegistryMessage, Status>> + Send + Unpin + 'static
    {
        tokio::time::timeout(self.options.registration_timeout, stream.next()).await
            .map_err(|_| Status::deadline_exceeded("Registration request timeout"))?
            .ok_or_else(|| Status::cancelled("Input stream closed"))?
            .map(RegistryServiceImpl::extract_registration)?
            .ok_or_else(|| Status::invalid_argument("First message should be a valid registration request"))
    }

    fn extract_registration(
        message: ServerToRegistryMessage,
    ) -> Option<ExtractedServerRegistrationRequest> {
        let request = match message.message? {
            server_to_registry_message::Message::Request(req) => req,
            _ => return None,
        };

        let registration = match request.request? {
            server_to_registry_request::Request::Registration(reg) => reg,
            _ => return None,
        };


        Some(ExtractedServerRegistrationRequest {
            request_id: message.request_id?,
            server_id: registration.id?,
            address: registration.address,
            rooms: registration.rooms,
            load_info: registration.load_info?,
        })
    }
}

#[tonic::async_trait]
impl RegistryService for RegistryServiceImpl {
    type RegisterServerStream = RegisterServerStream;

    async fn register_server(
        &self,
        request: Request<Streaming<ServerToRegistryMessage>>,
    ) -> Result<Response<Self::RegisterServerStream>, Status> {
        self.register_server_inner(request).await
    }
}

type RegisterServerStream = NotifiableReceiverStream<Result<RegistryToServerMessage, Status>>;

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use prost::Message;
    use tokio::sync::{mpsc, RwLock};
    use tokio::sync::mpsc::Sender;
    use tonic::{Request, Status};
    use tonic::codegen::tokio_stream::StreamExt;
    use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
    use super::RegistryServiceImpl;
    use crate::generator::{RoomCodeCollection, RoomCodeGeneratorImpl};
    use crate::Options;
    use crate::proto::common::RoomCode;
    use crate::proto::registry::{registry_to_server_message, registry_to_server_response, server_registration_error, server_registration_response, server_to_registry_message, server_to_registry_request, RegistryToServerMessage, ServerId, ServerLoadInfo, ServerRegistrationError, ServerRegistrationRequest, ServerRegistrationSuccess, ServerToRegistryMessage, ServerToRegistryRequest};
    use crate::registry::Registry;
    use crate::utils::test_utils::ServerToRegistryMessagePack;

    fn initialize_registry() -> Arc<RwLock<Registry>> {
        let generator = RoomCodeGeneratorImpl::new(Some(42), 3, 0);
        Arc::new(RwLock::new(Registry::new(generator)))
    }

    fn initialize_service() -> RegistryServiceImpl {
        let options = Options {
            registration_timeout: std::time::Duration::from_millis(100),
        };
        RegistryServiceImpl::new(initialize_registry(), Arc::new(options))
    }

    fn make_registration_request_message(request_id: u64, server_id: ServerId) -> ServerToRegistryMessage {
        ServerRegistrationRequest {
            id: Some(server_id),
            address: "".to_string(),
            rooms: vec![],
            load_info: Some(ServerLoadInfo {
                load: 0.0,
                accepting_new_rooms: true
            }),
        }.pack(Some(request_id))
    }

    fn make_request() -> (Sender<Result<ServerToRegistryMessage, Status>>, Request<ReceiverStream<Result<ServerToRegistryMessage, Status>>>) {
        let (tx, rx) = mpsc::channel::<Result<ServerToRegistryMessage, Status>>(1);
        let request = Request::new(ReceiverStream::new(rx));
        (tx, request)
    }
    
    fn assert_message_is_registration_success(
        message: RegistryToServerMessage
    ) -> ServerRegistrationSuccess {
        match message.message {
            Some(registry_to_server_message::Message::Response(response)) => match response.response {
                Some(registry_to_server_response::Response::ServerRegistration(response)) => match response.response {
                    Some(server_registration_response::Response::Success(registration)) => registration,
                    _ => panic!("Expected a successful server registration response"),
                },
                _ => panic!("Expected a server registration response"),
            },
            _ => panic!("Expected a response message"),
        }
    }

    #[tokio::test]
    async fn test_registers_successfully() {
        let service = initialize_service();
        let (tx, request) = make_request();

        let server_id = ServerId {
            server_id: 84
        };

        let registration_request_id = 42u64;

        tx.send(Ok(make_registration_request_message(registration_request_id, server_id))).await.unwrap();

        let response = service.register_server_inner(request).await.expect("Failed to register server");

        let mut response_stream = response.into_inner();
        let first_message = response_stream.next().await.unwrap().unwrap();

        assert_eq!(first_message.request_id, Some(registration_request_id));

        let registration = assert_message_is_registration_success(first_message);

        assert_eq!(registration.server_id, Some(server_id));
        assert_eq!(registration.conflicts.len(), 0);
    }

    #[tokio::test]
    async fn test_gets_timed_out_without_registration() {
        let service = initialize_service();
        let (tx, request) = make_request();

        let status = match service.register_server_inner(request).await {
            Ok(_) => panic!("Expected an error"),
            Err(status) => status,
        };
        let _ = tx;

        assert_eq!(status.code(), tonic::Code::DeadlineExceeded);
    }

    #[tokio::test]
    async fn test_gets_cancelled_when_stream_closes() {
        let service = initialize_service();
        let (tx, request) = make_request();

        drop(tx);

        let status = match service.register_server_inner(request).await {
            Ok(_) => panic!("Expected an error"),
            Err(status) => status,
        };

        assert_eq!(status.code(), tonic::Code::Cancelled);
    }

    #[tokio::test]
    async fn test_gets_invalid_argument_error_before_timeout() {
        let registration_timeout = std::time::Duration::from_millis(100);

        let options = Options { registration_timeout };
        let service = RegistryServiceImpl::new(initialize_registry(), Arc::new(options));

        let test_timeout = registration_timeout - std::time::Duration::from_millis(50);

        let (tx, request) = make_request();

        let message = ServerToRegistryMessage {
            request_id: Some(42),
            message: Some(server_to_registry_message::Message::Request(
                ServerToRegistryRequest {
                    request: Some(server_to_registry_request::Request::UpdateLoadInfo(
                        ServerLoadInfo {
                            load: 0.0,
                            accepting_new_rooms: true,
                        }
                    ))
                }
            ))
        };
        tx.send(Ok(message)).await.unwrap();

        let result = tokio::time::timeout(test_timeout, service.register_server_inner(request)).await
            .expect("Invalid argument error was not returned in time");

        let status = match result {
            Ok(_) => panic!("Expected an error"),
            Err(status) => status,
        };

        assert_eq!(status.code(), tonic::Code::InvalidArgument);
    }

    #[tokio::test]
    async fn test_registers_2_servers_successfully() {
        let service = initialize_service();

        let server_id_1 = ServerId {
            server_id: 84
        };

        let server_id_2 = ServerId {
            server_id: 85
        };

        let (tx, request) = make_request();
        tx.send(Ok(make_registration_request_message(42u64, server_id_1))).await.unwrap();
        let mut response_stream = service.register_server_inner(request).await.expect("Failed to register first server").into_inner();
        let first_message = response_stream.next().await.unwrap().unwrap();
        assert_message_is_registration_success(first_message);

        let (tx, request) = make_request();
        tx.send(Ok(make_registration_request_message(43u64, server_id_2))).await.unwrap();
        let mut response_stream = service.register_server_inner(request).await.expect("Failed to register second server").into_inner();
        let first_message = response_stream.next().await.unwrap().unwrap();
        assert_message_is_registration_success(first_message);
    }
    
    #[tokio::test]
    async fn test_returns_id_already_exists_error() {
        let service = initialize_service();

        let server_id = ServerId {
            server_id: 84
        };

        let (tx, request) = make_request();
        tx.send(Ok(make_registration_request_message(42u64, server_id))).await.unwrap();
        service.register_server_inner(request).await.expect("Failed to register first server");

        let (tx, request) = make_request();
        tx.send(Ok(make_registration_request_message(11u64, server_id))).await.unwrap();
        let status = match service.register_server_inner(request).await {
            Ok(_) => panic!("Expected an error"),
            Err(status) => status,
        };

        assert_eq!(status.code(), tonic::Code::AlreadyExists);
        let details = prost_types::Any::decode(status.details())
            .expect("Failed to decode error details");

        assert_eq!(details.type_url, "type.googleapis.com/com.muuzika.registry.ServerRegistrationError");
        let error = ServerRegistrationError::decode(details.value.as_ref())
            .expect("Failed to decode error");

        match error.error {
            Some(server_registration_error::Error::IdAlreadyExists(_)) => {},
            _ => panic!("Expected an ID already exists error"),
        }
    }
    
    #[tokio::test]
    async fn used_ids_get_reassigned() {
        let code_collection = RoomCodeCollection::new_from_vec(3, vec![1907]);
        let generator = RoomCodeGeneratorImpl::new_with_collection(code_collection, 0);
        let registry = Arc::new(RwLock::new(Registry::new(generator)));
        let options = Options {
            registration_timeout: std::time::Duration::from_millis(100),
        };
        let service = RegistryServiceImpl::new(registry, Arc::new(options));

        let (tx, request) = make_request();
        
        let first_registration = ServerRegistrationRequest {
            id: Some(ServerId { server_id: 1 }),
            address: "".to_string(),
            rooms: vec![RoomCode { code: 2602 }],
            load_info: Some(ServerLoadInfo {
                load: 0.0,
                accepting_new_rooms: true
            }),
        };
        
        tx.send(Ok(first_registration.pack(Some(0)))).await.unwrap();
        service.register_server_inner(request).await.expect("Failed to register first server");

        let (tx, request) = make_request();

        let second_registration = ServerRegistrationRequest {
            id: Some(ServerId { server_id: 2 }),
            address: "".to_string(),
            rooms: vec![RoomCode { code: 2602 }, RoomCode { code: 1107 }],
            load_info: Some(ServerLoadInfo {
                load: 0.0,
                accepting_new_rooms: true
            }),
        };

        tx.send(Ok(second_registration.pack(Some(0)))).await.unwrap();
        let mut response_stream = service.register_server_inner(request).await.expect("Failed to register second server").into_inner();
        let first_message = response_stream.next().await.unwrap().unwrap();
        
        let registration = assert_message_is_registration_success(first_message);
        
        assert_eq!(registration.conflicts.len(), 1);
        assert_eq!(registration.conflicts[0].before, Some(RoomCode {code: 2602}));
        assert_eq!(registration.conflicts[0].after, Some(RoomCode {code: 1907}));
    }
}