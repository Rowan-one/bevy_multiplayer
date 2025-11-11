use bevy::prelude::*;
use bevy_replicon::prelude::*;
use networking::server::*;
use networking::replication::*;
use shared::components::*;
use game::setup_level;

const PLAYER_MOVE_SPEED: f32 = 5.0;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(NetServerPlugin)

        .add_systems(Startup, (setup_level, setup_simple_camera))
        .add_systems(Update, (
            spawn_players_system,
            update_player_velocity,
        ))
        .add_systems(FixedUpdate, (
            apply_velocity_system.after(update_player_velocity),
        ))

        .insert_resource(ServerLobby::default())

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

fn apply_velocity_system(mut query: Query<(&Velocity, &mut Transform)>, time: Res<Time<Fixed>>) {
    for (velocity, mut transform) in query.iter_mut() {
        transform.translation += velocity.0 * time.delta_secs();
    }
}