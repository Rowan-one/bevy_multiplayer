use std::{f32::consts::PI, time::Duration};
use bevy::prelude::*;
use bevy_renet::renet::*;
use serde::{Deserialize, Serialize};

pub enum ClientChannel {
    Input,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct NetworkedEntities {
    pub entities: Vec<Entity>,
    pub translations: Vec<[f32; 3]>,
}

pub fn connection_config() -> ConnectionConfig {
    ConnectionConfig {
        available_bytes_per_tick: 1024 * 1024,
        client_channels_config: ClientChannel::channels_config(),
        ..Default::default()
    }
}

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

impl From<ClientChannel> for u8 {
    fn from(value: ClientChannel) -> Self {
        match value {
            ClientChannel::Input => 0
        }
    }
}

impl ClientChannel {
    pub fn channels_config() -> Vec<ChannelConfig> {
        vec!{
            ChannelConfig {
                channel_id: Self::Input.into(),
                max_memory_usage_bytes: 5 * 1024 * 1024,
                send_type: SendType::ReliableOrdered { resend_time: Duration::ZERO },
            }
        }
    }
}