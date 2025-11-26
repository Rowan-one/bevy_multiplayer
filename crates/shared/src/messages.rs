use bevy_ecs::prelude::*;

use crate::structs::AssignLocalPlayer;

#[derive(Debug, Default, Message)]
pub struct AssignLocalPlayerMessage(pub AssignLocalPlayer);

#[derive(Debug, Default, Message)]
pub struct ServerTickMessage {
    pub tick: u64,
    pub timestamp: f32,
}
