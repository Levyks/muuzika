use muuzika_registry::proto::common::RoomCode;
use muuzika_registry::proto::registry::registry_service_client::RegistryServiceClient;
use muuzika_registry::proto::registry::{server_registration_error, server_to_registry_request, ServerId, ServerLoadInfo, ServerRegistrationError, ServerRegistrationRequest};
use muuzika_registry::Options;

mod utils;

use crate::utils::{assert_message_is_registration_success, decode_bytes_any, make_request, register_server, setup_test_server, setup_test_server_with_codes, setup_test_server_with_options, ServerToRegistryMessagePack};

#[tokio::test]
async fn test_registers_successfully() {
    let (shutdown_tx, channel) = setup_test_server().await;

    let mut client = RegistryServiceClient::connect(channel)
        .await
        .expect("Failed to connect to server");

    let (tx, request) = make_request();

    let server_id = ServerId {
        server_id: 84
    };

    let registration_request_id = 42u64;
    let registration_message = ServerRegistrationRequest {
        id: Some(server_id),
        address: "".to_string(),
        rooms: vec![],
        load_info: Some(ServerLoadInfo {
            load: 0.0,
            accepting_new_rooms: true,
        }),
    }.pack(registration_request_id);
    tx.send(registration_message).await.unwrap();

    let mut response_stream = client
        .register_server(request)
        .await
        .expect("Failed to register server")
        .into_inner();

    let first_message = response_stream.message().await.unwrap().unwrap();
    assert_eq!(first_message.request_id, Some(registration_request_id));

    let registration = assert_message_is_registration_success(first_message);
    assert_eq!(registration.server_id, Some(server_id));
    assert_eq!(registration.conflicts.len(), 0);

    shutdown_tx.send(()).unwrap();
}

#[tokio::test]
async fn test_gets_timed_out_without_registration() {
    let (shutdown_tx, channel) = setup_test_server().await;
    let (tx, request) = make_request();

    let mut client = RegistryServiceClient::connect(channel)
        .await
        .expect("Failed to connect to server");

    let status = match client.register_server(request).await {
        Ok(_) => panic!("Expected an error"),
        Err(status) => status,
    };
    let _ = tx;

    assert_eq!(status.code(), tonic::Code::DeadlineExceeded);

    shutdown_tx.send(()).unwrap();
}

#[tokio::test]
async fn test_gets_cancelled_when_stream_closes() {
    let (shutdown_tx, channel) = setup_test_server().await;

    let mut client = RegistryServiceClient::connect(channel)
        .await
        .expect("Failed to connect to server");

    let (tx, request) = make_request();
    drop(tx);

    let status = match client.register_server(request).await {
        Ok(_) => panic!("Expected an error"),
        Err(status) => status,
    };

    assert_eq!(status.code(), tonic::Code::Cancelled);

    shutdown_tx.send(()).unwrap();
}

#[tokio::test]
async fn test_gets_invalid_argument_error_before_timeout() {
    let registration_timeout = std::time::Duration::from_millis(100);
    let options = Options { registration_timeout };

    let (shutdown_tx, channel) = setup_test_server_with_options(options).await;

    let mut client = RegistryServiceClient::connect(channel)
        .await
        .expect("Failed to connect to server");

    let test_timeout = registration_timeout - std::time::Duration::from_millis(50);
    let (tx, request) = make_request();

    let message = server_to_registry_request::Request::UpdateLoadInfo(
        ServerLoadInfo {
            load: 0.0,
            accepting_new_rooms: true,
        }
    ).pack(42);

    tx.send(message).await.unwrap();

    let result = tokio::time::timeout(test_timeout, client.register_server(request)).await
        .expect("Invalid argument error was not returned in time");

    let status = match result {
        Ok(_) => panic!("Expected an error"),
        Err(status) => status,
    };

    assert_eq!(status.code(), tonic::Code::InvalidArgument);

    shutdown_tx.send(()).unwrap();
}

#[tokio::test]
async fn test_registers_2_servers_successfully() {
    let (shutdown_tx, channel) = setup_test_server().await;

    let mut client = RegistryServiceClient::connect(channel)
        .await
        .expect("Failed to connect to server");

    let (_, first_tx, _) = register_server(&mut client, ServerRegistrationRequest {
        id: Some(ServerId {
            server_id: 84
        }),
        address: "".to_string(),
        rooms: vec![],
        load_info: Some(ServerLoadInfo {
            load: 0.0,
            accepting_new_rooms: true,
        }),
    }).await.expect("Failed to register first server");

    register_server(&mut client, ServerRegistrationRequest {
        id: Some(ServerId {
            server_id: 85
        }),
        address: "".to_string(),
        rooms: vec![],
        load_info: Some(ServerLoadInfo {
            load: 0.0,
            accepting_new_rooms: true,
        }),
    }).await.expect("Failed to register second server");

    drop(first_tx);
    shutdown_tx.send(()).unwrap();
}

#[tokio::test]
async fn test_returns_id_already_exists_error() {
    let (shutdown_tx, channel) = setup_test_server().await;

    let mut client = RegistryServiceClient::connect(channel)
        .await
        .expect("Failed to connect to server");

    let server_id = ServerId {
        server_id: 84
    };

    let (_, first_tx, _) = register_server(&mut client, ServerRegistrationRequest {
        id: Some(server_id),
        address: "".to_string(),
        rooms: vec![],
        load_info: Some(ServerLoadInfo {
            load: 0.0,
            accepting_new_rooms: true,
        }),
    }).await.expect("Failed to register first server");

    let result = register_server(&mut client, ServerRegistrationRequest {
        id: Some(server_id),
        address: "".to_string(),
        rooms: vec![],
        load_info: Some(ServerLoadInfo {
            load: 0.0,
            accepting_new_rooms: true,
        }),
    }).await;

    let status = match result {
        Ok(_) => panic!("Expected an error"),
        Err(status) => status,
    };

    assert_eq!(status.code(), tonic::Code::AlreadyExists);

    let error = decode_bytes_any::<ServerRegistrationError>(
        status.details(),
        "type.googleapis.com/com.muuzika.registry.ServerRegistrationError",
    );

    match error.error {
        Some(server_registration_error::Error::IdAlreadyExists(_)) => {}
        _ => panic!("Expected an ID already exists error"),
    }

    drop(first_tx);
    shutdown_tx.send(()).unwrap();
}

#[tokio::test]
async fn used_ids_get_reassigned() {
    let (shutdown_tx, channel) = setup_test_server_with_codes(vec![1907]).await;

    let mut client = RegistryServiceClient::connect(channel)
        .await
        .expect("Failed to connect to server");

    let (_, first_tx, _) = register_server(&mut client, ServerRegistrationRequest {
        id: Some(ServerId { server_id: 1 }),
        address: "".to_string(),
        rooms: vec![RoomCode { code: 2602 }],
        load_info: Some(ServerLoadInfo {
            load: 0.0,
            accepting_new_rooms: true,
        }),
    }).await.expect("Failed to register first server");

    let (registration, _, _) = register_server(&mut client, ServerRegistrationRequest {
        id: Some(ServerId { server_id: 2 }),
        address: "".to_string(),
        rooms: vec![RoomCode { code: 2602 }, RoomCode { code: 1107 }],
        load_info: Some(ServerLoadInfo {
            load: 0.0,
            accepting_new_rooms: true,
        }),
    }).await.expect("Failed to register second server");

    assert_eq!(registration.conflicts.len(), 1);
    assert_eq!(registration.conflicts[0].before, Some(RoomCode { code: 2602 }));
    assert_eq!(registration.conflicts[0].after, Some(RoomCode { code: 1907 }));

    drop(first_tx);
    shutdown_tx.send(()).unwrap();
}