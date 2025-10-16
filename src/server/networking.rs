use std::collections::HashMap;
use bevy::prelude::*;
use bevy_renet::renet::{ClientId, RenetServer, ServerEvent};
use crate::{server::messages::ClientConnectMessage, shared::{networking::ClientChannel, physics::Velocity, player::PlayerInput}};

const PLAYER_MOVE_SPEED: f32 = 5.0;

#[derive(Debug, Default, Resource)]
pub struct ServerLobby {
    pub players: HashMap<ClientId, Entity>
}

pub fn handle_events_system(
    mut server_events: MessageReader<ServerEvent>,
    mut connect_message: MessageWriter<ClientConnectMessage>,
) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                info!("Client connected: {}", client_id);
                connect_message.write(ClientConnectMessage { client_id: *client_id });
            },
            ServerEvent::ClientDisconnected { client_id, reason } => {
                info!("Client {} disconnected: {}", client_id, reason);
            },
        }
    }
}

pub fn receive_player_input(
    mut server: ResMut<RenetServer>,
    mut lobby: ResMut<ServerLobby>,
    mut commands: Commands,
) {
    let config = bincode::config::standard();

    for client_id in server.clients_id() {
        while let Some(message) = server.receive_message(client_id, ClientChannel::Input) {
            let (input, _): (PlayerInput, usize) = bincode::serde::decode_from_slice(&message, config).unwrap();
            if let Some(player_entity) = lobby.players.get(&client_id) {
                commands.entity(*player_entity).insert(input);
            }
        }
    }
}

pub fn update_player_velocity(
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

pub fn apply_velocity_system(mut query: Query<(&Velocity, &mut Transform)>, time: Res<Time>) {
    for (velocity, mut transform) in query.iter_mut() {
        transform.translation += velocity.0 * time.delta_secs();
    }
}