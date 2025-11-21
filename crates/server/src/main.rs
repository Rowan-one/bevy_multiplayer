use bevy::prelude::*;
use bevy_replicon::prelude::*;
use networking::server::*;
use networking::replication::*;
use shared::components::*;
use shared::consts::CLIENT_TICK_RATE;
use shared::consts::PLAYER_MOVE_SPEED;
use shared::consts::SERVER_TICK_RATE;
use shared::messages::*;
use game::setup_level;
use shared::resources::InputStateMap;
use shared::resources::PlayerInput;
use shared::resources::PrevFrameTime;
use shared::resources::ServerLobby;
use shared::resources::TickAccumulator;
use shared::structs::AssignLocalPlayer;
use shared::structs::InputPayload;

#[derive(Debug, Default, Resource)]
struct ServerTick(u64);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(NetServerPlugin)

        .add_systems(Startup, (setup_level, setup_simple_camera))
        .add_systems(Update, (
            spawn_players_system,
            server_tick_system,
            process_inputs_system.before(networking::replication::send_snapshots_system),
            //integrate_physics_system.before(send_snapshots_system),
        ))

        .insert_resource(ServerLobby::default())
        .insert_resource(ServerTick::default())
        .insert_resource(TickAccumulator::default())
        .insert_resource(PrevFrameTime::default())
        .insert_resource(InputStateMap::default())

        .add_message::<ServerTickMessage>() 

        .run();
}

pub fn setup_simple_camera(mut commands: Commands) {
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-20.5, 30.0, 20.5).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn spawn_players_system(
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
                Transform::from_xyz(0., 0.51, 0.),
                Replicated,
                OwnedByClient { id: message.client_id },
                net_id,
            ))
            .insert(Player {client_id: message.client_id as u64})
            .insert(Velocity(Vec3::ZERO))
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

fn server_tick_system(
    time: Res<Time>,
    mut prev_frame_time: ResMut<PrevFrameTime>,
    mut accumulator: ResMut<TickAccumulator>,
    mut server_tick: ResMut<ServerTick>,
    mut tick_message_writer: MessageWriter<ServerTickMessage>,
) {
    let current_time = time.elapsed_secs();
    let frame_time = current_time - prev_frame_time.0;
    prev_frame_time.0 = current_time;
    
    // increment accumulator by frame time
    accumulator.0 += frame_time;

    while accumulator.0 >= SERVER_TICK_RATE {
        // send tick message
        tick_message_writer.write(ServerTickMessage{
            tick: server_tick.0,
            timestamp: current_time,
        });

        accumulator.0 -= SERVER_TICK_RATE;
        server_tick.0 += 1;
    }
}

fn update_player_velocity(
    velocity: &mut Velocity,
    input: &PlayerInput,
) {
    let x = (input.right as i8 - input.left as i8) as f32;
    let y = (input.down as i8 - input.up as i8) as f32;
    let direction = Vec2::new(x, y).normalize_or_zero();
    velocity.0.x = direction.x * PLAYER_MOVE_SPEED;
    velocity.0.z = direction.y * PLAYER_MOVE_SPEED;
}

fn process_inputs_system(
    time: Res<Time>,
    mut input_state_map: ResMut<InputStateMap>,
    server_lobby: Res<ServerLobby>,
    mut server_tick_message: MessageReader<ServerTickMessage>,
    mut query: Query<(&mut Transform, &mut Velocity)>
) {
    for tick in server_tick_message.read() {
        for (client_id, input_state) in input_state_map.0.iter_mut() {
            println!("Processing inputs for client {}", client_id);
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
            
            let (mut transform, mut velocity) = query.get_mut(*player_entity).unwrap();
            println!("n inputs to apply: {:?}",inputs_to_apply.len());
            for i in 0..inputs_to_apply.len() {
                let payload = inputs_to_apply[i];

                update_player_velocity(&mut velocity, &payload.input);
                integrate(&mut transform, &mut velocity, CLIENT_TICK_RATE);

                input_state.last_processed_input = payload;
            }
            
            // clear buffer of all inputs already consumed
            input_state.buffer.retain(|p| p.seq > input_state.last_processed_input.seq);
        }
    }
}

fn integrate(
    transform: &mut Transform,
    velocity: &mut Velocity,
    dt: f32,
) {
    transform.translation += velocity.0 * dt;
}
