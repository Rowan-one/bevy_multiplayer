use std::collections::HashMap;

use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

use crate::structs::{AssignLocalPlayer, ClientInputState, InputPayload};

#[derive(Debug, Default, Serialize, Deserialize, Resource)]
pub struct ClientInputBuffer(pub Vec<InputPayload>);

#[derive(Debug, Default, Resource)]
pub struct InputSequence(pub u64);
impl InputSequence {
    pub fn next(&mut self) -> u64 {
        self.0 += 1;
        self.0
    }
}

#[derive(Debug, Default, Resource)]
pub struct LocalPlayerNetId(pub Option<u64>);

#[derive(Debug, Default, Resource)]
pub struct PendingAssignLocalPlayer(pub Option<AssignLocalPlayer>);

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Resource)]
pub struct PlayerInput{
    pub right: bool,
    pub left: bool,
    pub up: bool,
    pub down: bool,
    pub seq: u64,
}

#[derive(Debug, Default, Resource)]
pub struct InputStateMap(pub HashMap<u64, ClientInputState>);

#[derive(Debug, Default, Resource)]
pub struct PrevFrameTime(pub f32);

#[derive(Debug, Default, Resource)]
pub struct ServerLobby {
    pub players: HashMap<u64, Entity>
}

#[derive(Debug, Default, Resource)]
pub struct TickAccumulator(pub f32);
