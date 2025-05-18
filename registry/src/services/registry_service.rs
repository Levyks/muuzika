use crate::messages::registration::ExtractedServerRegistrationRequest;
use crate::options::Options;
use crate::proto::registry::registry_service_server::{RegistryService, RegistryServiceServer};
use crate::proto::registry::{registry_to_server_response, server_to_registry_message, server_to_registry_request, RegistryToServerMessage, ServerToRegistryMessage};
use crate::registry::Registry;
use crate::server::{Server, ServerRunner};
use crate::utils::packing::RegistryToServerMessagePack;
use crate::utils::stream::NotifiableReceiverStream;
use nanoid::nanoid;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};
use tonic::codegen::tokio_stream::{Stream, StreamExt};
use tonic::{Request, Response, Status, Streaming};

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

    async fn wait_for_registration(&self, stream: &mut Streaming<ServerToRegistryMessage>) -> Result<ExtractedServerRegistrationRequest, Status>
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
    type RegisterServerStream = NotifiableReceiverStream<Result<RegistryToServerMessage, Status>>;

    async fn register_server(
        &self,
        request: Request<Streaming<ServerToRegistryMessage>>,
    ) -> Result<Response<Self::RegisterServerStream>, Status> {
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
            server_runner.clone(),
        );

        let mut registry = self.registry.write().await;
        let response = registry.register_server(server)?;

        let message = registry_to_server_response::Response::RegistrationSuccess(response)
            .pack(Some(registration.request_id));

        tx.send(Ok(message)).await.map_err(|_| {
            log::warn!("[{log_id}] Failed to send registration response");
            Status::internal("Failed to send response")
        })?;

        let stream_log_id = log_id.clone();
        tokio::spawn(async move {
            server_runner.run(
                stream.filter_map(move |m|
                    m.map_err(|_| log::warn!("[{stream_log_id}] Failed to receive message")).ok()
                )
            ).await;
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
}