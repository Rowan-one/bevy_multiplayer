use std::net::UdpSocket;
use std::time::SystemTime;
use std::collections::HashMap;
use bevy::prelude::*;
use crate::shared::events::PlayerInputEvent;
use crate::shared::networking::connection_config;
use bevy_renet::netcode::*;
use bevy_renet::RenetServerPlugin;
use bevy_renet::renet::RenetServer;
use bevy_multiplayer::*;
use crate::server::networking::receive_player_input;

#[derive(Debug, Component)]
struct Player {
    id: ClientId
}

#[derive(Debug, Default, Resource)]
struct Lobby {
    players: HashMap<ClientId, Entity>
}

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
    let server = RenetServer::new(connection_config());

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

    app.add_systems(Startup, (setup_level, setup_simple_camera));
    app.add_systems(Update, handle_events_system);
    app.add_systems(Update, receive_player_input);

    app.add_message::<PlayerInputEvent>();

    app.run();
}

fn handle_events_system(mut server_events: MessageReader<ServerEvent>) {
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

pub fn setup_simple_camera(mut commands: Commands) {
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-20.5, 30.0, 20.5).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}
