use std::{collections::HashMap, net::UdpSocket, time::SystemTime};
use bevy::prelude::*;
use bevy_renet::{netcode::{NetcodeServerPlugin, NetcodeServerTransport, ServerAuthentication, ServerConfig}, renet::*, RenetServerPlugin};
use shared::components::*;
use super::protocol::*;

pub struct NetServerPlugin;

impl Plugin for NetServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RenetServerPlugin);
        app.add_plugins(NetcodeServerPlugin);

        let (server, transport) = new_renet_server();
        app.insert_resource(server);
        app.insert_resource(transport);

        app.add_systems(Update, (
            handle_events_system,
            receive_player_input,
        ));

        app.add_message::<ClientConnectMessage>();
    }
}

#[derive(Message)]
pub struct ClientConnectMessage {
    pub client_id: ClientId,
}

pub fn new_renet_server() -> (RenetServer, NetcodeServerTransport) {
    let public_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind(public_addr).unwrap();
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let server_config = ServerConfig {
        current_time,
        max_clients: 64,
        protocol_id: PROTOCOL_ID,
        public_addresses: vec![public_addr],
        authentication: ServerAuthentication::Unsecure,
    };

    let transport = NetcodeServerTransport::new(server_config, socket).unwrap();
    let server = RenetServer::new(connection_config());

    (server, transport)
}

pub fn handle_events_system(
    mut server_events: MessageReader<ServerEvent>,
    mut connect_message: MessageWriter<ClientConnectMessage>,
) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                info!("Client connected: {}", client_id);
                connect_message.write(ClientConnectMessage { client_id: *client_id });
            },
            ServerEvent::ClientDisconnected { client_id, reason } => {
                info!("Client {} disconnected: {}", client_id, reason);
            },
        }
    }
}

pub fn receive_player_input(
    mut server: ResMut<RenetServer>,
    mut lobby: ResMut<ServerLobby>,
    mut commands: Commands,
) {
    let config = bincode::config::standard();

    for client_id in server.clients_id() {
        while let Some(message) = server.receive_message(client_id, ClientChannel::Input) {
            let (input, _): (PlayerInput, usize) = bincode::serde::decode_from_slice(&message, config).unwrap();
            if let Some(player_entity) = lobby.players.get(&client_id) {
                commands.entity(*player_entity).insert(input);
            }
        }
    }
}