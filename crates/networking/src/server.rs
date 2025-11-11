use std::{net::UdpSocket, time::SystemTime};
use bevy::prelude::*;
use bevy_renet::{netcode::*, renet::{ClientId, *}, RenetServerPlugin};
use shared::components::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::*;
use super::protocol::*;
use super::replication::*;

#[derive(Message)]
pub struct ClientConnectMessage {
    pub client_id: ClientId,
}
pub struct NetServerPlugin;

impl Plugin for NetServerPlugin {
    fn build(&self, app: &mut App) {
        //app.add_plugins(RenetServerPlugin);
        //app.add_plugins(NetcodeServerPlugin);
        app.add_plugins((RepliconPlugins, RepliconRenetPlugins));

        app.replicate::<Player>();
        app.replicate::<NetId>();

        app.add_systems(Startup, setup_renet_server);
        app.add_systems(Update, (
            handle_events_system,
            receive_input_system,
        ));

        app.insert_resource(Time::<Fixed>::from_hz(1.));
        app.add_systems(FixedUpdate, (
            send_snapshots_system,
        ));

        app.add_message::<ClientConnectMessage>();

        app.insert_resource(NetIdGen::default());
    }
}

fn setup_renet_server(
    mut commands: Commands,
    replicon_channels: Res<RepliconChannels>,
) {
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
    let server = RenetServer::new(connection_config(replicon_channels));

    commands.insert_resource(server);
    commands.insert_resource(transport);
}

fn handle_events_system(
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

fn receive_input_system(
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