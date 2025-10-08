use std::net::{SocketAddr, UdpSocket};
use std::time::SystemTime;
use bevy::{log::LogPlugin, prelude::*};
use bevy_renet::netcode::{NetcodeServerPlugin, NetcodeServerTransport, ServerAuthentication, ServerConfig};
use bevy_renet::{RenetServerPlugin};
use bevy_renet::renet::{RenetServer};
use bevy_multiplayer::*;
use local_ip_address::linux::local_ip;

fn new_renet_server() -> (RenetServer, NetcodeServerTransport) {
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
    let server = RenetServer::new(ConnectionConfig::default());

    (server, transport)
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(RenetServerPlugin);
    app.add_plugins(NetcodeServerPlugin);

    let (server, transport) = new_renet_server();
    app.insert_resource(server);
    app.insert_resource(transport);

    app.add_systems(Update, handle_events_system);
    app.add_systems(Update, server_ping_system);

    app.run();
}

fn handle_events_system(mut server_events: EventReader<ServerEvent>) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                info!("Client connected: {}", client_id);
            },
            ServerEvent::ClientDisconnected { client_id, reason } => {
                info!("Client {} disconnected: {}", client_id, reason);
            },
        }
    }
}

fn server_ping_system(mut server: ResMut<RenetServer>) {
    let reliable_channel_id: u8 = 1;
    let config = bincode::config::standard();

    for client_id in server.clients_id().into_iter() {
        while let Some(message) = server.receive_message(client_id, reliable_channel_id) {
            let client_message: (ClientMessage, usize) = bincode::serde::decode_from_slice(&message, config).unwrap();
            match client_message.0 {
                ClientMessage::Ping => {
                    info!("Got ping from {}!", client_id);
                    let pong = bincode::serde::encode_to_vec(&ServerMessage::Pong, config).unwrap();
                    server.send_message(client_id, reliable_channel_id, pong);
                }
            }
        }
    }
}
