use std::{net::UdpSocket, time::SystemTime};
use bevy::prelude::*;
use bevy_renet::{netcode::*, renet::{ClientId, *}};
use shared::{components::*, messages::AssignLocalPlayerMessage, resources::InputStateMap, structs::InputPayload};
use bevy_replicon::prelude::*;
use bevy_replicon_renet::*;
use super::protocol::*;
use super::replication::*;

#[derive(Message)]
pub struct ClientConnectMessage {
    pub client_id: ClientId,
}

#[derive(Debug, Default, Clone, Copy, Component)]
pub struct OwnedByClient { pub id: u64 }

pub struct NetServerPlugin;

impl Plugin for NetServerPlugin {
    fn build(&self, app: &mut App) {
        //app.add_plugins(RenetServerPlugin);
        //app.add_plugins(NetcodeServerPlugin);
        app.add_plugins((RepliconPlugins, RepliconRenetPlugins));

        app.replicate::<Player>();
        app.replicate::<NetId>();
        app.replicate::<Velocity>();

        app.add_systems(Startup, setup_renet_server);
        app.add_systems(Update, (
            handle_events_system,
            receive_input_system,
            send_snapshots_system,
            send_local_player_system,
        ));

        app.add_message::<ClientConnectMessage>();
        app.add_message::<AssignLocalPlayerMessage>();

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
    mut input_state_map: ResMut<InputStateMap>,
) {
    let config = bincode::config::standard();

    for client_id in server.clients_id() {
        while let Some(message) = server.receive_message(client_id, ClientChannel::Input) {
            let (input, _): (InputPayload, usize) = bincode::serde::decode_from_slice(&message, config).unwrap();

            // append new input to client's input buffer
            input_state_map.0.entry(client_id).or_default().buffer.push(input);
        }
    }
}

fn send_local_player_system(
    mut renet_server: ResMut<RenetServer>,
    mut assign_local_player_message: MessageReader<AssignLocalPlayerMessage>,
) {
    for message in assign_local_player_message.read() {
        // encode assign local player struct
        let config = bincode::config::standard();
        let encoded_message = bincode::serde::encode_to_vec(&message.0, config).unwrap();
        
        // send assign local player struct
        renet_server.send_message(message.0.client_id, ServerChannel::AssignLocalPlayer, encoded_message);
    }
}
