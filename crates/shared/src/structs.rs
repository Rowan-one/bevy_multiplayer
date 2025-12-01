use glam::Vec3;
use serde::{Serialize, Deserialize};
use avian3d::prelude::{Position, LinearVelocity};
use crate::{components::{Grounded, Player}, resources::{ClientInputBuffer, PlayerInput}};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct AssignLocalPlayer {
    pub player_net_id: u64,
    pub client_id: u64,
}

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize)]
pub struct EntitySnap {
    pub net_id: u64,
    pub position: Vec3,
    pub velocity: Vec3,
    pub last_processed_seq: u64,
    pub timestamp: f32,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct InputPayload {
    pub seq: u64,
    pub input: PlayerInput, 
    pub timestamp: f32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct ClientInputState {
    pub last_processed_input: InputPayload,
    pub buffer: Vec<InputPayload>,
}

// currently unused
#[derive(Debug, Default)]
pub struct PlayerState {
    pub position: Position,
    pub velocity: LinearVelocity,
    pub grounded: Grounded,
    pub input: PlayerInput,
}
