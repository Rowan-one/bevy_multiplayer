use bevy::prelude::*;
use bevy_renet::renet::RenetClient;
use crate::shared::networking::ClientChannel;
use crate::shared::player::PlayerInput;

pub fn update_player_input(
    mut input: ResMut<PlayerInput>,
    keyboard: Res<ButtonInput<KeyCode>>,
) {
    input.left = keyboard.pressed(KeyCode::KeyA);
    input.right = keyboard.pressed(KeyCode::KeyD);
    input.up = keyboard.pressed(KeyCode::KeyW);
    input.down = keyboard.pressed(KeyCode::KeyS);
}

pub fn send_player_input(
    mut client: ResMut<RenetClient>,
    input: Res<PlayerInput>,
) {
    let config = bincode::config::standard();
    let bytes = bincode::serde::encode_to_vec(&*input, config).unwrap();
    client.send_message(ClientChannel::Input, bytes);
}
