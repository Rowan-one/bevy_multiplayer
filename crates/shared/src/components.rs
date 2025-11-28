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


#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Component)]
pub struct Velocity(pub Vec3);

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Component)]
pub struct Position(pub Vec3);

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Component)]
pub struct Grounded(pub bool);

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Component)]
pub struct RestingHeight(pub f32); // used with grounded component to determine ground check
                                   // distance
