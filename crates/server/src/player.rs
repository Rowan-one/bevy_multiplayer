use bevy_rapier3d::{parry::shape::Ball, prelude::*};
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use networking::{replication::NetIdGen, server::{ClientConnectMessage, OwnedByClient}};
use shared::{components::*, consts::{CLIENT_TICK_RATE, SERVER_TICK_RATE, SKIN_WIDTH}, functions::{collide_and_slide, integrate, simulate_player}, messages::{AssignLocalPlayerMessage, ServerTickMessage}, resources::*, structs::{AssignLocalPlayer, InputPayload}};

pub struct ServerPlayerPlugin;
impl Plugin for ServerPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<InputStateMap>();
        app.add_systems(Update, (
            spawn_players_system,
            process_inputs_system
                .before(bevy_rapier3d::plugin::PhysicsSet::StepSimulation)
                .before(networking::replication::send_snapshots_system),
            integrate_players_system
                .after(process_inputs_system),
            sync_transforms_system
                .after(integrate_players_system),
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
            .insert(Gravity::default())
            .insert(Transform::default())
            .insert(Grounded(true))
            .insert(RestingHeight(2.0))
            .insert(KinematicCharacterController::default())
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
    mut query: Query<(&mut Transform, &mut CustomVelocity, &mut Gravity, &mut Grounded)>
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
            
            let (mut transform, mut velocity, mut gravity, mut grounded) = query.get_mut(*player_entity).unwrap();
            for i in 0..inputs_to_apply.len() {
                let payload = &inputs_to_apply[i];
                (transform.translation, velocity.0) = simulate_player(transform.translation, velocity.0, &mut gravity, &mut grounded, &payload.input, CLIENT_TICK_RATE);
                input_state.last_processed_input = payload.clone();
            }
            
            // clear buffer of all inputs already consumed
            input_state.buffer.retain(|p| p.seq > input_state.last_processed_input.seq);
        }
    }
}

pub fn integrate_players_system(
    rapier_context: ReadRapierContext,
    mut server_tick_message: MessageReader<ServerTickMessage>,
    mut query: Query<(&CustomVelocity, &mut CustomPosition, &Gravity), With<OwnedByClient>>,
) {
    for _tick in server_tick_message.read() {
        for (velocity, mut position, gravity) in query.iter_mut() {
            // scale velocity by delta time for integration
            let move_vel: Vec3 = velocity.0 * SERVER_TICK_RATE;
            let gravity_vel: Vec3 = gravity.vector * SERVER_TICK_RATE;

            let shape = Ball::new(1.0 - SKIN_WIDTH);

            // collide and slide movement pass
            let collision_move_vector = collide_and_slide(
                position.0,
                move_vel,
                &shape,
                0,
                false,
                move_vel,
                &rapier_context.single().unwrap(),
            );

            position.0 = integrate(position.0, collision_move_vector, SERVER_TICK_RATE);

            // collide and slide gravity pass
            let collision_gravity_vector = collide_and_slide(
                position.0,
                gravity_vel,
                &shape,
                0,
                true,
                gravity_vel,
                &rapier_context.single().unwrap(),
            );

            position.0 = integrate(position.0, collision_gravity_vector, SERVER_TICK_RATE);
        }
    }
}

pub fn sync_transforms_system(
    mut query: Query<(&CustomPosition, &mut Transform)>,
) {
    for (pos, mut transform) in query.iter_mut() {
        transform.translation = pos.0;
    }
}
