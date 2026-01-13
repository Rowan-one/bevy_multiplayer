use bevy_ecs::prelude::*;
use bevy_rapier3d::parry::utils::hashmap::HashMap;
use serde::{Serialize, Deserialize};
use glam::{Quat, Vec3};

use crate::enums::IKSolverType;

#[derive(Debug, Component, Serialize, Deserialize)]
pub struct Player {
    pub client_id: u64,
}

#[derive(Debug, Component)]
pub struct PlayerVisualRoot;

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

#[derive(Debug, Default, Component)]
pub struct ServerPosition(pub Vec3);

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Component)]
pub struct Grounded(pub bool);

#[derive(Debug, Default, Clone, Copy, Serialize, Deserialize, Component)]
pub struct RestingHeight(pub f32); // used with grounded component to determine ground check
                                   // distance

#[derive(Debug, Default, Component)]
pub struct BoneMap(pub HashMap<String, Entity>);

#[derive(Debug, Default, Component)]
pub struct IKSegment {
    pub length: f32,
    pub attached_object: Option<Entity>,
}
impl IKSegment {
    pub fn new(length: f32, attached_object: Option<Entity>) -> IKSegment {
        IKSegment {
            length,
            attached_object,
        }
    }
}

#[derive(Debug, Component)]
pub struct IKChain {
    pub segments: Vec<IKSegment>,
    pub target: Entity,
    pub solve_type: IKSolverType,
}
impl IKChain {
    pub fn new(target: Entity, solve_type: IKSolverType) -> IKChain {
        IKChain {
            segments: Vec::new(),
            target,
            solve_type,
        }
    }

    pub fn add(mut self, segment: IKSegment) -> Self {
        self.segments.push(segment);
        self
    }
}

#[derive(Debug, Default, Component)]
pub struct IKRig {
    pub chains: Vec<IKChain>,
}
impl IKRig {
    pub fn add_chain(mut self, chain: IKChain) -> Self {
        self.chains.push(chain);
        self
    }
}

#[derive(Debug, Default, Component)]
pub struct AnimStateTime(pub f32); // affected by player's speed & time spent moving, dictates
                                   // where we currently are in the animation cycle

#[derive(Debug, Component)]
pub struct IKTarget;

#[derive(Debug, Component)]
pub struct LoadingBoneCache;
