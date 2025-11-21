use std::{net::UdpSocket, time::SystemTime};
use bevy::prelude::*;
use bevy_renet::{netcode::*, renet::RenetClient};
use shared::components::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::*;
use shared::resources::ClientInputBuffer;
use shared::resources::InputSequence;
use shared::resources::LocalPlayerNetId;
use shared::resources::PendingAssignLocalPlayer;
use shared::resources::PlayerInput;
use shared::structs::AssignLocalPlayer;
use shared::structs::EntitySnap;
use shared::structs::InputPayload;
use crate::server::ClientConnectMessage;

use super::protocol::*;
use super::replication::*;

const INTERPOLATION_DELAY: f32 = 0.2;

pub struct NetClientPlugin;

impl Plugin for NetClientPlugin {
    fn build(&self, app: &mut App) {
        //app.add_plugins(RenetClientPlugin);
        //app.add_plugins(NetcodeClientPlugin);
        app.add_plugins((RepliconPlugins, RepliconRenetPlugins));

        app.replicate::<Player>();
        app.replicate::<NetId>();
        app.replicate::<Velocity>();

        app.add_systems(Startup, setup_renet_client);
        app.add_systems(Update, (
            send_input_system,
            replicate_players_system,
            receive_snapshots_system,
            update_id_map_system,
            interpolate_entities_system,
            receive_local_player_system,
        ));

        app.add_message::<ClientConnectMessage>();
        app.add_message::<SnapshotReceiveMessage>();

        app.insert_resource(NetIdMap::default());
        app.insert_resource(SnapshotBuffer::default());
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
    time: Res<Time>,
    mut client: ResMut<RenetClient>,
    mut input_sequence: ResMut<InputSequence>,
    player_input: Res<PlayerInput>,
    mut client_input_buffer: ResMut<ClientInputBuffer>,
) {
    // only send input if it is different from doing nothing
    // if player_input.clone() == PlayerInput::default() { return; }

    // create input payload from resource
    let input = InputPayload {
        seq: input_sequence.next(),
        input: player_input.clone(),
        timestamp: time.elapsed_secs(),
        dt: time.delta_secs(),
    };

    // add input to local buffer
    client_input_buffer.0.push(input);

    // send input to server
    let config = bincode::config::standard();
    let bytes = bincode::serde::encode_to_vec(input, config).unwrap();
    client.send_message(ClientChannel::Input, bytes);
}

fn replicate_players_system(
    mut commands: Commands,
    mut client: ResMut<RenetClient>,
    query: Query<Entity, (Added<Player>, With<Replicated>)>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for entity in query.iter() {
        println!("Player replicated to client");
        let mesh = meshes.add(Mesh::from(Capsule3d::default()));
        let material = materials.add(Color::srgb(1., 0., 1.));
        commands.entity(entity).insert((Mesh3d(mesh), MeshMaterial3d(material)));
    }
}

fn update_id_map_system(
    query: Query<(Entity, &NetId), Added<NetId>>,
    mut id_map: ResMut<NetIdMap>,
) {
    for (entity, net_id) in query.iter() {
        id_map.0.insert(net_id.0, entity);
        println!("Added entity to id map. id: {:?}, entity: {:?}", net_id.0, entity)
    }
}

fn interpolate_entities_system(
    time: Res<Time>,
    snapshot_buffer: ResMut<SnapshotBuffer>,
    id_map: ResMut<NetIdMap>,
    local_player_net_id: ResMut<LocalPlayerNetId>,
    mut transforms: Query<&mut Transform>,
) {
    let render_time = time.elapsed_secs() - INTERPOLATION_DELAY;

    // iterate through all entitys' buffers
    for (net_id, deque) in snapshot_buffer.0.iter() {
        // dont interpolate local player, that will be left to client prediction
        if let Some(local_player_id) = &local_player_net_id.0 {
            if net_id == local_player_id { continue; }
        }

        // make sure we have at least 2 samples
        if deque.len() < 2 { continue; }

        // get local entity
        let Some(&entity) = id_map.0.get(net_id) else { continue; };
        let Ok(mut transform) = transforms.get_mut(entity) else { continue; };

        let samples: Vec<EntitySnap> = deque.iter().copied().collect();

        // find window bracketing render time
        let mut found = false;
        let mut s1: (f32, Vec3) = (0., Vec3::ZERO);
        let mut s2: (f32, Vec3) = (0., Vec3::ZERO);
        for window in samples.windows(2) {
            let (t0, p0) = (window[0].timestamp, window[0].position);
            let (t1, p1) = (window[1].timestamp, window[1].position);
            if t0 <= render_time && t1 >= render_time {
                s1 = (t0, p0);
                s2 = (t1, p1);
                found = true;
                break;
            }
        }

        // calculate alpha
        let denom = (s2.0 - s1.0).max(f32::EPSILON);
        let a = ((render_time - s1.0) / denom).clamp(0.0, 1.0);
        //println!("Alpha: {}", a);

        transform.translation = s1.1.lerp(s2.1, a);
    }
}

fn receive_local_player_system(
    mut client: ResMut<RenetClient>,
    mut local_player_net_id: ResMut<LocalPlayerNetId>,
    mut pending_assign_local_player: ResMut<PendingAssignLocalPlayer>,
    net_id_map: Res<NetIdMap>,
    mut commands: Commands,
) {
    let config = bincode::config::standard();

    while let Some(message) = client.receive_message(ServerChannel::AssignLocalPlayer) {
        let (assign_local_player, _): (AssignLocalPlayer, usize) = bincode::serde::decode_from_slice(&message, config).unwrap();
        println!("got assign local player. net id: {}", assign_local_player.player_net_id);
        
        // update local player resources 
        local_player_net_id.0 = Some(assign_local_player.player_net_id);
        pending_assign_local_player.0 = Some(assign_local_player);
    }
}












