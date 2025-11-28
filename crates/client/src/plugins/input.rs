use bevy::prelude::*;
use shared::{components::{Grounded, LocalPlayer}, enums::PlayerActionType, resources::{ClientInputBuffer, InputSequence, PlayerInput}};

pub struct ClientInputPlugin;
impl Plugin for ClientInputPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PlayerInput>();
        app.init_resource::<InputSequence>();
        app.init_resource::<ClientInputBuffer>();
        app.add_systems(Update, update_player_input);
    }
}

fn update_player_input(
    mut input: ResMut<PlayerInput>,
    keyboard: Res<ButtonInput<KeyCode>>,
    mut player_grounded: Single<&mut Grounded, With<LocalPlayer>>,
) {
    input.left = keyboard.pressed(KeyCode::KeyA);
    input.right = keyboard.pressed(KeyCode::KeyD);
    input.up = keyboard.pressed(KeyCode::KeyW);
    input.down = keyboard.pressed(KeyCode::KeyS);
    
    input.jump = keyboard.just_pressed(KeyCode::Space);
}
