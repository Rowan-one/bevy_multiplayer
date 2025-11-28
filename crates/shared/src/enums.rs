use bevy_ecs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone, Copy)]
pub enum PlayerActionType {
    Jump,
    PrimaryUse,
    SecondaryUse,
}
