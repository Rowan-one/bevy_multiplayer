use bevy::prelude::*;
use shared::components::LocalPlayer;

pub struct ClientCameraPlugin;

impl Plugin for ClientCameraPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_camera);
        app.add_systems(Update, (
            follow_player_system,
        ));
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0., 16.0, 2.5).looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
    ));
}

fn follow_player_system(
    local_player: Single<&Transform, With<LocalPlayer>>,
    mut camera: Single<&mut Transform, (With<Camera3d>, Without<LocalPlayer>)>,
) {
    camera.translation = local_player.translation + Vec3::new(0., 10., 2.);
}
