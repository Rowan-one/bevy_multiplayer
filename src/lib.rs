pub use bevy::prelude::*;
pub use bevy_renet::renet::*;
pub use bevy_renet::*;

use serde::{Deserialize, Serialize};
use std::f32::consts::PI;

use crate::client::networking::ClientChannel;

pub mod shared;
pub mod client;
pub mod server;

pub const PROTOCOL_ID: u64 = 1000;

pub fn setup_level(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Cuboid::new(40., 1., 40.)))),
        Transform::from_xyz(0., -1., 0.),
        MeshMaterial3d(materials.add(Color::srgb(0., 0., 0.3))),
    ));

    // light
    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        Transform {
            translation: Vec3::new(0.0, 2.0, 0.0),
            rotation: Quat::from_rotation_x(-PI / 4.),
            ..default()
        },
    ));
}