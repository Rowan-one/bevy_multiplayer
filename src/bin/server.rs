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

    app.add_systems(Update, server_update_system);

    app.run();
}

fn server_update_system(mut server_events: EventReader<ServerEvent>) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                info!("Client connected: {}", client_id);
            },
            ServerEvent::ClientDisconnected { client_id, reason } => {
                info!("Client disconnected: {}", client_id);
            },
        }
    }
}

// Systems
