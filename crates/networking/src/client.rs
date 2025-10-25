use std::{net::UdpSocket, time::SystemTime};
use bevy::prelude::*;
use bevy_renet::{netcode::*, renet::RenetClient, RenetClientPlugin};
use shared::components::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::*;
use crate::server::ClientConnectMessage;

use super::protocol::*;
use super::replication::*;

pub struct NetClientPlugin;

impl Plugin for NetClientPlugin {
    fn build(&self, app: &mut App) {
        //app.add_plugins(RenetClientPlugin);
        //app.add_plugins(NetcodeClientPlugin);
        app.add_plugins((RepliconPlugins, RepliconRenetPlugins));

        app.replicate::<Player>();
        app.replicate::<NetId>();

        app.add_systems(Startup, setup_renet_client);
        app.add_systems(Update, (
            send_input_system,
            detect_players_system,
            receive_snapshots_system,
            update_id_map_system,
        ));

        app.add_message::<ClientConnectMessage>();

        app.insert_resource(NetIdGen::default());
        app.insert_resource(NetIdMap::default());
    }
}

fn setup_renet_client(
    mut commands: Commands,
    replicon_channels: Res<RepliconChannels>,
) {
    let server_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let client_id = current_time.as_millis() as u64;
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };

    let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();
    let client = RenetClient::new(connection_config(replicon_channels));

    commands.insert_resource(client);
    commands.insert_resource(transport);
}

fn send_input_system(
    mut client: ResMut<RenetClient>,
    input: Res<PlayerInput>,
) {
    let config = bincode::config::standard();
    let bytes = bincode::serde::encode_to_vec(&*input, config).unwrap();
    client.send_message(ClientChannel::Input, bytes);
}


fn detect_players_system(
    mut commands: Commands,
    query: Query<Entity, (Added<Player>, With<Replicated>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for player in query.iter() {
        println!("Player replicated to client");
        let mesh = meshes.add(Mesh::from(Capsule3d::default()));
        let material = materials.add(Color::srgb(1., 0., 1.));
        commands.entity(player).insert((Mesh3d(mesh), MeshMaterial3d(material)));
    }
}

fn update_id_map_system(
    query: Query<(Entity, &NetId), Added<NetId>>,
    mut id_map: ResMut<NetIdMap>,
) {
    for (entity, net_id) in query.iter() {
        println!("Adding net id {} for replicated entity", net_id.0);
        id_map.0.insert(net_id.0, entity);
    }
}
