use std::sync::Arc;
use nanoid::nanoid;
use tokio::sync::{mpsc, RwLock};
use crate::proto::registry::registry_service_server::{RegistryService, RegistryServiceServer};
use crate::proto::registry::{registry_to_server_response, server_registration_response, server_to_registry_message, server_to_registry_request, RegistryToServerMessage, ServerRegistrationRequest, ServerRegistrationResponse, ServerToRegistryMessage};
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
    use tokio::sync::{mpsc, RwLock};
    use tonic::{Request, Status};
    use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
    use super::RegistryServiceImpl;
    use crate::generator::RoomCodeGeneratorImpl;
    use crate::Options;
    use crate::proto::registry::ServerToRegistryMessage;
    use crate::registry::Registry;

    fn initialize_registry() -> Arc<RwLock<Registry>> {
        let generator = RoomCodeGeneratorImpl::new(Some(42), 3, 0);
        Arc::new(RwLock::new(Registry::new(generator)))
    }
    
    #[tokio::test]
    async fn test_gets_timed_out_without_registration() {
        let options = Options {
            registration_timeout: std::time::Duration::from_millis(200),
        };
        let service = RegistryServiceImpl::new(initialize_registry(), Arc::new(options));
        let (tx, rx) = mpsc::channel::<Result<ServerToRegistryMessage, Status>>(1);
        
        let request = Request::new(ReceiverStream::new(rx));
        
        let status = match service.register_server_inner(request).await {
            Ok(_) => panic!("Expected an error"),
            Err(status) => status,
        };
        let _ = tx;
        
        assert_eq!(status.code(), tonic::Code::DeadlineExceeded);
    }
}