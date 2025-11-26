use std::collections::HashMap;
use bevy_ecs::prelude::*;
use serde::{Serialize, Deserialize};
use glam::Vec3;

#[derive(Debug, Component, Serialize, Deserialize)]
pub struct Player {
    pub client_id: u64,
}

// local player marker component
#[derive(Debug, Default, Clone, Copy, Component)]
pub struct LocalPlayer;


#[derive(Debug, Default, Serialize, Deserialize, Component)]
pub struct Velocity(pub Vec3);

#[derive(Debug, Default, Serialize, Deserialize, Component)]
pub struct Position(pub Vec3);
