use crate::codes::RoomCodeGenerator;
use crate::packing::RegistryToServerMessagePack;
use crate::proto::registry::registry_service_server::{RegistryService, RegistryServiceServer};
use crate::proto::registry::{registry_to_server_response, server_registration_response, server_to_registry_message, server_to_registry_request, RegistryToServerMessage, ServerRegistrationRequest, ServerRegistrationResponse, ServerToRegistryMessage};
use crate::registry::Registry;
use crate::utils::NotifiableReceiverStream;
use nanoid::nanoid;
use std::sync::Arc;
use tonic::codegen::tokio_stream::StreamExt;
use tonic::{Request, Response, Status, Streaming};

pub struct RegistryServiceImpl<G: RoomCodeGenerator + Send + 'static> {
    registry: Arc<Registry<G>>,
    registration_timeout: std::time::Duration,
}

#[tonic::async_trait]
impl<G: RoomCodeGenerator + Send + 'static> RegistryService for RegistryServiceImpl<G> {
    type RegisterServerStream = NotifiableReceiverStream<Result<RegistryToServerMessage, Status>>;

    async fn register_server(
        &self,
        request: Request<Streaming<ServerToRegistryMessage>>,
    ) -> Result<Response<Self::RegisterServerStream>, Status> {
        let log_id = nanoid!();

        let remote_addr = request.remote_addr().map(|a| a.to_string()).unwrap_or_default();
        log::debug!("[{}] Started registration request from address {}", log_id, remote_addr);

        let mut stream = request.into_inner();
        let (request_id, registration) = self.wait_for_registration(&mut stream).await
            .map_err(|e| {
                log::warn!("[{}] Failed to get registration data: {:?}", log_id, e);
                e
            })?;

        log::debug!("[{}] Received registration data: {:?}", log_id, registration);

        let (tx, rx) = tokio::sync::mpsc::channel::<Result<RegistryToServerMessage, Status>>(1);

        let (server_id, response, runner) = self.registry.register_server(registration, tx.clone(), self.registry.clone()).await;
        log::debug!("[{}] Registered server as {}", log_id, runner);

        let message = registry_to_server_response::Response::ServerRegistration(ServerRegistrationResponse {
            response: Some(server_registration_response::Response::Success(response)),
        }).pack(Some(request_id));

        tx.send(Ok(message)).await.map_err(|_| Status::internal("Failed to send response"))?;

        tokio::spawn(async move {
            // TODO: extract to runner method?!?
            log::debug!("Starting input stream handler loop for server {}", runner);
            while let Some(message) = stream.next().await {
                match message {
                    Ok(message) => runner.handle_message(message).await,
                    Err(e) => log::warn!("Got error from input stream for server {}: {:?}", server_id, e),
                }
            }
            log::debug!("Input stream for server {} ended", runner);
            runner.remove_self();
        });

        let registry = self.registry.clone();
        Ok(Response::new(NotifiableReceiverStream::new(rx, move || {
            log::debug!("[{}] Output stream for server {} closed", log_id, server_id);
            registry.remove_server(server_id);
        })))
    }
}


impl<G: RoomCodeGenerator + Send + 'static> RegistryServiceImpl<G> {
    pub fn server(registry: &Arc<Registry<G>>) -> RegistryServiceServer<impl RegistryService> {
        RegistryServiceServer::new(Self {
            registry: registry.clone(),
            // TODO: from env/config
            registration_timeout: std::time::Duration::from_secs(5),
        })
    }

    async fn wait_for_registration(&self, stream: &mut Streaming<ServerToRegistryMessage>) -> Result<(u64, ServerRegistrationRequest), Status> {
        tokio::time::timeout(self.registration_timeout, stream.next()).await
            .map_err(|_| Status::deadline_exceeded("Registration request timeout"))?
            .ok_or_else(|| Status::cancelled("Input stream closed"))?
            .map(extract_registration)?
            .ok_or_else(|| Status::invalid_argument("First message should be a valid registration request"))
    }
}


fn extract_registration(
    message: ServerToRegistryMessage,
) -> Option<(u64, ServerRegistrationRequest)> {
    let request = match message.message? {
        server_to_registry_message::Message::Request(req) => req,
        _ => return None,
    };

    let registration = match request.request? {
        server_to_registry_request::Request::Registration(reg) => reg,
        _ => return None,
    };

    Some((message.request_id?, registration))
}
