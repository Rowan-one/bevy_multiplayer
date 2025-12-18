use bevy_ecs::prelude::*;
use serde::{Serialize, Deserialize};
use glam::{Quat, Vec3};

#[derive(Debug, Component, Serialize, Deserialize)]
pub struct Player {
    pub client_id: u64,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, Copy, PartialEq, Component)]
pub struct LookAngles {
    pub yaw: f32,
    pub pitch: f32,
}

#[derive(Debug, Default, Component)]
pub struct CustomPosition(pub Vec3);

#[derive(Debug, Default, Component)]
pub struct CustomRotation(pub Quat);

#[derive(Debug, Default, Component)]
pub struct CustomVelocity(pub Vec3);

#[derive(Debug, Default, Component)]
pub struct WishDir(pub Vec3);

#[derive(Debug, Component)]
pub struct Gravity {
    pub vector: Vec3,
    pub scale: f32,
}
impl Default for Gravity {
    fn default() -> Self {
        Gravity {
            vector: Vec3::ZERO,
            scale: 1.0,
        }
    }
}

// local player marker component
#[derive(Debug, Default, Clone, Copy, Component)]
pub struct LocalPlayer;

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Component)]
pub struct Grounded(pub bool);

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Component)]
pub struct RestingHeight(pub f32); // used with grounded component to determine ground check
                                   // distance
