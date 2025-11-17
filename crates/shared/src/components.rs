use std::collections::HashMap;
use bevy_ecs::prelude::*;
use serde::{Serialize, Deserialize};
use glam::Vec3;

#[derive(Debug, Component, Serialize, Deserialize)]
pub struct Player {
    pub client_id: u64,
}

// local player marker component
#[derive(Debug, Component)]
pub struct LocalPlayer;

#[derive(Debug, Default, Resource)]
pub struct ServerLobby {
    pub players: HashMap<u64, Entity>
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Component, Resource)]
pub struct PlayerInput {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}

#[derive(Debug, Default, Component)]
pub struct Velocity(pub Vec3);


#[derive(Serialize, Deserialize, Debug, Copy, Clone)]
pub struct AssignLocalPlayer {
    pub client_id: u64,
    pub player_net_id: u64,
}
