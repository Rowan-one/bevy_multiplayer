use bevy::{input::mouse::MouseMotion, prelude::*};
use shared::{components::{LocalPlayer, LookAngles}, consts::BASE_SENSITIVITY};
use crate::plugins::core::Settings;

pub struct ClientCameraPlugin;

impl Plugin for ClientCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera);
        app.add_systems(PostUpdate, (
            (
                rotate_camera_system,
                follow_player_system,
            ).chain(),
        ));
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0., 16.0, 2.5).looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
    ));
}

fn rotate_camera_system(
    settings: Res<Settings>,
    mut look_angles: Single<&mut LookAngles, With<LocalPlayer>>,
    mut mouse: MessageReader<MouseMotion>,
    mut camera: Single<&mut Transform, With<Camera3d>>,
) {
    let mut delta = Vec2::ZERO;
    for motion in mouse.read() {
        delta += motion.delta;
    }

    let sensitivity = settings.sensitivity * BASE_SENSITIVITY;
    look_angles.yaw -= delta.x * sensitivity;
    look_angles.pitch -= delta.y * sensitivity;

    // clamp vertical look angle
    look_angles.pitch = look_angles.pitch.clamp(-1.0, 1.0); // radians

    camera.rotation = Quat::from_axis_angle(Vec3::Y, look_angles.yaw)
        * Quat::from_axis_angle(Vec3::X, look_angles.pitch);
}

fn follow_player_system(
    local_player: Single<&Transform, With<LocalPlayer>>,
    mut camera: Single<&mut Transform, (With<Camera3d>, Without<LocalPlayer>)>,
) {
    let forward = camera.forward();
    let right = camera.right();
    camera.translation = local_player.translation + forward * -8.0;
    camera.translation += Vec3::new(0., 0.5, 0.);
    camera.translation += right * 1.0;
}
