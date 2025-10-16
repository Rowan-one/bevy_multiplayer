use bevy::prelude::*;
use bevy_renet::renet::ClientId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Component)]
pub struct Player {
    pub id: ClientId,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Component, Resource)]
pub struct PlayerInput {
    pub up: bool,
    pub down: bool,
    pub left: bool,
    pub right: bool,
}