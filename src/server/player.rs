use bevy::prelude::*;
use crate::{server::{messages::ClientConnectMessage, networking::ServerLobby}, shared::{physics::Velocity, player::{Player, PlayerInput}}};

pub fn spawn_player_system(
    mut connect_message: MessageReader<ClientConnectMessage>,
    mut lobby: ResMut<ServerLobby>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,

) {
    for message in connect_message.read() {
        let player_entity = commands
            .spawn((
                Mesh3d(meshes.add(Mesh::from(Capsule3d::default()))),
                MeshMaterial3d(materials.add(Color::srgb(0.8, 0.7, 0.6))),
                Transform::from_xyz(0., 0.51, 0.),
            ))
            .insert(PlayerInput::default())
            .insert(Player {id: message.client_id})
            .insert(Velocity(Vec3::ZERO))
            .id();

        // add player to lobby
        lobby.players.insert(message.client_id, player_entity);
    }
}