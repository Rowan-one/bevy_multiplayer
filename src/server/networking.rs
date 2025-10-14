use bevy::prelude::*;
use bevy_renet::renet::{DefaultChannel, RenetServer};
use bincode::*;
use crate::{client::networking::ClientChannel, shared::events::PlayerInputEvent};

pub fn receive_player_input(
    mut server: ResMut<RenetServer>,
    mut commands: Commands,
    mut message_writer: MessageWriter<PlayerInputEvent>,
) {
    let config = bincode::config::standard();

    for client_id in server.clients_id() {
        while let Some(message) = server.receive_message(client_id, ClientChannel::Input) {
            println!("Received a message from client");
            let (decoded, _) = bincode::serde::decode_from_slice(&message, config).unwrap();
            message_writer.write(decoded);
        }
    }
}
