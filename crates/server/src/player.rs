use bevy::prelude::*;
use bevy_replicon::prelude::*;
use networking::{replication::NetIdGen, server::{ClientConnectMessage, OwnedByClient}};
use shared::{components::*, consts::CLIENT_TICK_RATE, functions::simulate_player, messages::{AssignLocalPlayerMessage, ServerTickMessage}, resources::*, structs::{AssignLocalPlayer, InputPayload}};

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
            .insert(Player {client_id: message.client_id as u64})
            .insert(Velocity(Vec3::ZERO))
            .insert(Position(Vec3::new(0., 4.0, 0.)))
            .insert(Grounded(true))
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
    mut input_state_map: ResMut<InputStateMap>,
    server_lobby: Res<ServerLobby>,
    mut server_tick_message: MessageReader<ServerTickMessage>,
    mut query: Query<(&mut Position, &mut shared::components::Velocity, &mut Grounded)>
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
            
            let (mut position, mut velocity, mut grounded) = query.get_mut(*player_entity).unwrap();
            for i in 0..inputs_to_apply.len() {
                let payload = &inputs_to_apply[i];
                simulate_player(&mut position, &mut velocity, &mut grounded, &payload.input, CLIENT_TICK_RATE);
                input_state.last_processed_input = payload.clone();
            }
            
            // clear buffer of all inputs already consumed
            input_state.buffer.retain(|p| p.seq > input_state.last_processed_input.seq);
        }
    }
}
