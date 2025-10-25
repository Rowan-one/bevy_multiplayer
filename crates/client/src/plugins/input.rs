use bevy::prelude::*;
use shared::components::PlayerInput;

pub struct ClientInputPlugin;
impl Plugin for ClientInputPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(PlayerInput::default());
        app.add_systems(Update, update_player_input);
    }
}

fn update_player_input(
    mut input: ResMut<PlayerInput>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    input.left = keyboard.pressed(KeyCode::KeyA);
    input.right = keyboard.pressed(KeyCode::KeyD);
    input.up = keyboard.pressed(KeyCode::KeyW);
    input.down = keyboard.pressed(KeyCode::KeyS);
}