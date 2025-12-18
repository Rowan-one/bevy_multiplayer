use bevy_rapier3d:: prelude::*;
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use networking::{replication::NetIdGen, server::{ClientConnectMessage, OwnedByClient}};
use shared::{components::*, consts::{CLIENT_TICK_RATE}, functions::{simulate_player}, messages::{AssignLocalPlayerMessage, ServerTickMessage}, resources::*, structs::{AssignLocalPlayer, InputPayload}};

pub struct ServerPlayerPlugin;
impl Plugin for ServerPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputStateMap>();
        app.add_systems(Update, (
            spawn_players_system,
            process_inputs_system
                .before(bevy_rapier3d::plugin::PhysicsSet::StepSimulation)
                .before(networking::replication::send_snapshots_system),
        ));
    }
}

pub fn spawn_players_system(
    mut connect_message: MessageReader<ClientConnectMessage>,
    mut assign_local_player_message: MessageWriter<AssignLocalPlayerMessage>,
    mut lobby: ResMut<ServerLobby>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut net_id_gen: ResMut<NetIdGen>,
) {
    for message in connect_message.read() {
        // spawn player entity
        let net_id = net_id_gen.next();
        let player_entity = commands
            .spawn((
                Mesh3d(meshes.add(Mesh::from(Capsule3d::default()))),
                MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
                Transform::default(),
                Replicated,
                OwnedByClient { id: message.client_id },
                net_id,
            ))
            .insert(Collider::ball(1.))
            .insert(RigidBody::KinematicPositionBased)
            .insert(Player {client_id: message.client_id as u64})
            .insert(CustomVelocity::default())
            .insert(CustomPosition(Vec3::new(0., 4.0, 0.)))
            .insert(CustomRotation::default())
            .insert(Gravity::default())
            .insert(Transform::default())
            .insert(Grounded(true))
            .insert(WishDir::default())
            .insert(RestingHeight(2.0))
            .id();

        // add player to lobby
        lobby.players.insert(message.client_id, player_entity);

        // assign client their local player
        assign_local_player_message.write(AssignLocalPlayerMessage(AssignLocalPlayer {
            client_id: message.client_id,
            player_net_id: net_id.0 
        }));
    }
}

pub fn process_inputs_system(
    rapier_context: ReadRapierContext,
    mut input_state_map: ResMut<InputStateMap>,
    server_lobby: Res<ServerLobby>,
    mut server_tick_message: MessageReader<ServerTickMessage>,
    mut query: Query<(&mut CustomPosition, &mut CustomRotation, &mut CustomVelocity, &mut WishDir, &mut Gravity, &mut Grounded, &RestingHeight)>
) {
    for _tick in server_tick_message.read() {
        for (client_id, input_state) in input_state_map.0.iter_mut() {
            // get player entity else return
            let Some(player_entity) = server_lobby.players.get(client_id) else { continue; };

            // get all inputs to apply in order
            let mut inputs_to_apply: Vec<InputPayload> = input_state
                .buffer
                .iter()
                .filter(|p| p.seq > input_state.last_processed_input.seq)
                .cloned()
                .collect();

            inputs_to_apply.sort_by_key(|p| p.seq);

            // make sure there are inputs to apply
            if inputs_to_apply.is_empty() { continue; }

            let (mut position, mut rotation, mut velocity, mut wish_dir, mut gravity, mut grounded, resting_height) = query.get_mut(*player_entity).unwrap();
            for i in 0..inputs_to_apply.len() {
                let payload = &inputs_to_apply[i];
                (position.0, velocity.0, wish_dir.0, rotation.0) = simulate_player(
                    &rapier_context.single().unwrap(),
                    position.0,
                    rotation.0,
                    velocity.0,
                    &mut gravity,
                    &mut grounded, 
                    resting_height.0,
                    &payload.input,
                    CLIENT_TICK_RATE
                );
                input_state.last_processed_input = payload.clone();
            }
            
            // clear buffer of all inputs already consumed
            input_state.buffer.retain(|p| p.seq > input_state.last_processed_input.seq);
        }
    }
}
