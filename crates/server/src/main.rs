use bevy::prelude::*;
use bevy_replicon::prelude::*;
use networking::server::*;
use networking::replication::*;
use shared::components::*;
use shared::messages::*;
use game::setup_level;

const PLAYER_MOVE_SPEED: f32 = 5.0;
const PHYSICS_DT: f32 = 1./10.;

#[derive(Debug, Default, Resource)]
struct ServerTick(u64);

#[derive(Debug, Default, Resource)]
struct PhysicsAccumulator(f32);

#[derive(Debug, Default, Resource)]
struct PrevFrameTime(f32);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(NetServerPlugin)

        .add_systems(Startup, (setup_level, setup_simple_camera))
        .add_systems(Update, (
            spawn_players_system,
            update_player_velocity,
            server_tick_system,
            integrate_physics_system.before(send_snapshots_system),
        ))

        .insert_resource(ServerLobby::default())
        .insert_resource(ServerTick::default())
        .insert_resource(PhysicsAccumulator::default())
        .insert_resource(PrevFrameTime::default())

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
                net_id,
            ))
            .insert(PlayerInput::default())
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

fn update_player_velocity(
    mut query: Query<(&mut Velocity, &PlayerInput)>,
) {
    for (mut velocity, input) in query.iter_mut() {
        let x = (input.right as i8 - input.left as i8) as f32;
        let y = (input.down as i8 - input.up as i8) as f32;
        let direction = Vec2::new(x, y).normalize_or_zero();
        velocity.0.x = direction.x * PLAYER_MOVE_SPEED;
        velocity.0.z = direction.y * PLAYER_MOVE_SPEED;
    }
}

fn server_tick_system(
    time: Res<Time>,
    mut prev_frame_time: ResMut<PrevFrameTime>,
    mut accumulator: ResMut<PhysicsAccumulator>,
    mut server_tick: ResMut<ServerTick>,
    mut tick_message_writer: MessageWriter<ServerTickMessage>,
) {
    let current_time = time.elapsed_secs();
    let frame_time = current_time - prev_frame_time.0;
    prev_frame_time.0 = current_time;
    
    // increment accumulator by frame time
    accumulator.0 += frame_time;

    while accumulator.0 >= PHYSICS_DT {
        // send tick message
        tick_message_writer.write(ServerTickMessage(server_tick.0));

        accumulator.0 -= PHYSICS_DT;
        server_tick.0 += 1;
    }
}

fn integrate_physics_system(
    mut query: Query<(&Velocity, &mut Transform)>, 
    mut server_tick_message: MessageReader<ServerTickMessage>,
) {
    for message in server_tick_message.read() {
        for (velocity, mut transform) in query.iter_mut() {
            transform.translation += velocity.0 * PHYSICS_DT;
        }
    }
}
