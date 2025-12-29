use bevy::{input::mouse::MouseMotion, prelude::*};
use shared::{components::{LocalPlayer, LookAngles}, resources::{ClientInputBuffer, CurrentFrameInput, InputSequence, PlayerInput}};

pub struct ClientInputPlugin;
impl Plugin for ClientInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerInput>();
        app.init_resource::<InputSequence>();
        app.init_resource::<ClientInputBuffer>();
        app.add_systems(Update, update_player_input_system);
    }
}

fn update_player_input_system(
    mut input: ResMut<PlayerInput>,
    keyboard: Res<ButtonInput<KeyCode>>,
    look_angles: Single<&LookAngles, With<LocalPlayer>>,
) {
    input.left = keyboard.pressed(KeyCode::KeyA);
    input.right = keyboard.pressed(KeyCode::KeyD);
    input.up = keyboard.pressed(KeyCode::KeyW);
    input.down = keyboard.pressed(KeyCode::KeyS);

    input.look_angles = **look_angles;
    
    input.jump = keyboard.just_pressed(KeyCode::Space);
}
