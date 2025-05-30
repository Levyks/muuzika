use crate::utils::{register_server_and_start_mock_thread, setup_test_server_with_codes};
use muuzika_registry::proto::common::RoomCode;
use muuzika_registry::proto::registry::registry_lobby_service_client::RegistryLobbyServiceClient;
use muuzika_registry::proto::registry::registry_service_client::RegistryServiceClient;
use muuzika_registry::proto::registry::{JoinRoomResponse, RoomToken, ServerId, ServerInfo, ServerLoadInfo, ServerRegistrationRequest, UsernameAndPassword};

mod utils;

#[tokio::test]
async fn test_creates_room_successfully() {
    let (shutdown_tx, channel) = setup_test_server_with_codes(vec![1107]).await;

    let mut client = RegistryServiceClient::connect(channel.clone())
        .await
        .expect("Failed to connect to server");

    let server_id = ServerId {
        server_id: 84
    };

    register_server_and_start_mock_thread(&mut client, ServerRegistrationRequest {
        id: Some(server_id),
        address: "foo".to_string(),
        rooms: vec![],
        load_info: Some(ServerLoadInfo {
            load: 0.0,
            accepting_new_rooms: true,
        }),
    }).await.expect("Failed to register first server");

    let mut lobby_client = RegistryLobbyServiceClient::connect(channel)
        .await
        .expect("Failed to connect to server");

    let response = lobby_client.create_room(UsernameAndPassword {
        username: "test_user".to_string(),
        password: None,
    }).await.expect("Failed to create room").into_inner();

    assert_eq!(response, JoinRoomResponse {
        code: Some(RoomCode { code: 1107 }),
        token: Some(RoomToken { token: "test_token".to_string() }),
        server: Some(ServerInfo {
            id: Some(server_id),
            address: "foo".to_string(),
        }),
    });

    shutdown_tx.send(()).unwrap();
}

#[tokio::test]
async fn test_creates_room_in_least_loaded_server() {
    let (shutdown_tx, channel) = setup_test_server_with_codes(vec![1107]).await;

    let mut client = RegistryServiceClient::connect(channel.clone())
        .await
        .expect("Failed to connect to server");

    register_server_and_start_mock_thread(&mut client, ServerRegistrationRequest {
        id: Some(ServerId {
            server_id: 84
        }),
        address: "foo".to_string(),
        rooms: vec![],
        load_info: Some(ServerLoadInfo {
            load: 0.3,
            accepting_new_rooms: true,
        }),
    }).await.expect("Failed to register first server");
    register_server_and_start_mock_thread(&mut client, ServerRegistrationRequest {
        id: Some(ServerId {
            server_id: 85
        }),
        address: "foo".to_string(),
        rooms: vec![],
        load_info: Some(ServerLoadInfo {
            load: 0.2,
            accepting_new_rooms: true,
        }),
    }).await.expect("Failed to register second server");
    register_server_and_start_mock_thread(&mut client, ServerRegistrationRequest {
        id: Some(ServerId {
            server_id: 86
        }),
        address: "foo".to_string(),
        rooms: vec![],
        load_info: Some(ServerLoadInfo {
            load: 0.1,
            accepting_new_rooms: false,
        }),
    }).await.expect("Failed to register third server");

    let mut lobby_client = RegistryLobbyServiceClient::connect(channel)
        .await
        .expect("Failed to connect to server");

    let response = lobby_client.create_room(UsernameAndPassword {
        username: "test_user".to_string(),
        password: None,
    }).await.expect("Failed to create room").into_inner();

    assert_eq!(response, JoinRoomResponse {
        code: Some(RoomCode { code: 1107 }),
        token: Some(RoomToken { token: "test_token".to_string() }),
        server: Some(ServerInfo {
            id: Some(ServerId {
                server_id: 85
            }),
            address: "foo".to_string(),
        }),
    });

    shutdown_tx.send(()).unwrap();
}