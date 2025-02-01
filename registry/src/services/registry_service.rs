use crate::proto::registry::registry_service_server::{RegistryService, RegistryServiceServer};
use crate::proto::registry::{
    registry_to_server_error, registry_to_server_message, registry_to_server_response,
    registry_to_server_success, server_to_registry_error, server_to_registry_message,
    server_to_registry_request, server_to_registry_response, RegistryToServerError,
    RegistryToServerMessage, RegistryToServerResponse, RegistryToServerSuccess, ServerId,
    ServerRegistrationRequest, ServerRegistrationResponse, ServerToRegistryMessage,
    ServerToRegistryRequest,
};
use crate::server::{create_and_insert, Server};
use crate::state::State;
use std::sync::atomic::Ordering;
use std::sync::Arc;
use tokio::sync::mpsc::Sender;
use tonic::codegen::tokio_stream::wrappers::ReceiverStream;
use tonic::codegen::tokio_stream::StreamExt;
use tonic::{Request, Response, Status, Streaming};

struct RegistryServiceImpl {
    state: Arc<State>,
}

#[tonic::async_trait]
impl RegistryService for RegistryServiceImpl {
    type RegisterServerStream = ReceiverStream<Result<RegistryToServerMessage, Status>>;

    async fn register_server(
        &self,
        request: Request<Streaming<ServerToRegistryMessage>>,
    ) -> Result<Response<Self::RegisterServerStream>, Status> {
        let (tx, rx) = tokio::sync::mpsc::channel::<Result<RegistryToServerMessage, Status>>(1);
        tokio::spawn(register_server(
            self.state.clone(),
            request.into_inner(),
            tx,
        ));
        Ok(Response::new(ReceiverStream::new(rx)))
    }
}

pub fn create_registry_service_server(
    state: &Arc<State>,
) -> RegistryServiceServer<impl RegistryService> {
    RegistryServiceServer::new(RegistryServiceImpl {
        state: state.clone(),
    })
}

async fn register_server(
    state: Arc<State>,
    mut stream: Streaming<ServerToRegistryMessage>,
    tx: Sender<Result<RegistryToServerMessage, Status>>,
) -> () {
    // TODO: from env
    let registration_timeout = std::time::Duration::from_secs(5);

    let message =
        match tokio::time::timeout(registration_timeout, wait_for_first_message(&mut stream)).await
        {
            Ok(Ok(message)) => message,
            Ok(Err(e)) => {
                log::debug!("Failed to receive first message: {:?}", e);
                return;
            }
            Err(_) => {
                log::debug!("First message timeout");
                send_or_log(&tx, Err(Status::deadline_exceeded("Registration timeout"))).await;
                return;
            }
        };

    let (request_id, registration) = if let Some(x) = extract_registration(message) {
        x
    } else {
        log::debug!("First message should be a registration request");
        send_or_log(
            &tx,
            Err(Status::invalid_argument(
                "First message should be a registration request",
            )),
        )
        .await;
        return;
    };

    let (server, response) = create_and_insert(state.clone(), registration, tx.clone()).await;

    if send_or_log(
        &tx,
        Ok(RegistryToServerMessage {
            request_id: Some(request_id),
            message: Some(pack_success(
                registry_to_server_success::Success::RegistrationResponse(response),
            )),
        }),
    )
    .await
    {
        server.handle_stream(&mut stream).await;
        server.handle_stream(&mut stream).await;
    }
}

async fn wait_for_first_message(
    stream: &mut Streaming<ServerToRegistryMessage>,
) -> anyhow::Result<ServerToRegistryMessage> {
    loop {
        if let Some(message) = stream.next().await? {
            return Ok(message);
        }
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

async fn wait_for_registration(
    stream: &mut Streaming<ServerToRegistryMessage>,
) -> anyhow::Result<(u64, ServerRegistrationRequest)> {
    let message = wait_for_first_message(stream).await?;
    extract_registration(message)
        .ok_or_else(|| anyhow::anyhow!("First message should be a registration request"))
}

async fn send_or_log(
    tx: &Sender<Result<RegistryToServerMessage, Status>>,
    message: Result<RegistryToServerMessage, Status>,
) -> bool {
    if let Err(e) = tx.send(message).await {
        log::debug!("Failed to send message: {:?}", e);
        false
    } else {
        true
    }
}

fn pack_success(
    response: registry_to_server_success::Success,
) -> registry_to_server_message::Message {
    registry_to_server_message::Message::Response(RegistryToServerResponse {
        response: Some(registry_to_server_response::Response::Success(
            RegistryToServerSuccess {
                success: Some(response),
            },
        )),
    })
}

fn pack_error(error: registry_to_server_error::Error) -> registry_to_server_message::Message {
    registry_to_server_message::Message::Response(RegistryToServerResponse {
        response: Some(registry_to_server_response::Response::Error(
            RegistryToServerError { error: Some(error) },
        )),
    })
}
