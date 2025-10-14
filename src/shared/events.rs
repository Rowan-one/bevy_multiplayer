use bevy::prelude::*;
use serde::{Serialize, Deserialize};

#[derive(Message, Serialize, Deserialize, Debug, Clone)]
pub struct PlayerInputEvent {
    pub direction: Vec2,
}