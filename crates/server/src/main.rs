use bevy::prelude::*;
use bevy_rapier3d::plugin::NoUserData;
use bevy_rapier3d::plugin::RapierPhysicsPlugin;
use bevy_rapier3d::render::RapierDebugRenderPlugin;
use glam::Vec3;
use networking::server::*;
use server::physics::ground_check_system;
use server::physics::sync_positions_system;
use server::player::process_inputs_system;
use server::player::spawn_players_system;
use shared::{consts::*, messages::*, resources::*};
use game::setup_level;

#[derive(Debug, Default, Resource)]
struct ServerTick(u64);

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(NetServerPlugin)
        .add_plugins(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugins(RapierDebugRenderPlugin::default())

        .add_systems(Startup, (setup_level, setup_simple_camera))
        .add_systems(Update, (
            spawn_players_system,
            server_tick_system,
            process_inputs_system
                .after(server_tick_system)
                .before(networking::replication::send_snapshots_system),
            ground_check_system
                .after(process_inputs_system)
                .before(networking::replication::send_snapshots_system),
            sync_positions_system
                .after(process_inputs_system)
                .after(ground_check_system)
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
