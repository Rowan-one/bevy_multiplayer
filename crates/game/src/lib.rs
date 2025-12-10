use std::{f32::consts::PI, time::Duration};
use bevy_rapier3d::prelude::*;
use bevy::prelude::*;

pub fn setup_level(mut commands: Commands, mut meshes: ResMut<Assets<Mesh>>, mut materials: ResMut<Assets<StandardMaterial>>) {
    // ground
    commands.spawn((
        Mesh3d(meshes.add(Mesh::from(Cuboid::new(40., 1., 40.)))),
        Transform::from_xyz(0., -1., 0.),
        MeshMaterial3d(materials.add(Color::srgb(0., 0., 0.3))),
        Collider::cuboid(20., 0.5, 20.),
        RigidBody::Fixed
    ));

    // test 
    commands.spawn((
        Collider::cuboid(1., 5., 2.5),
        RigidBody::Fixed,
        Transform {
            translation: Vec3::new(-10., 0., 2.5),
            rotation: Quat::from_xyzw(0., 0., 0., 0.),
            ..Default::default()
        },
    ));
    commands.spawn((
        Collider::cuboid(1., 5., 2.5),
        RigidBody::Fixed,
        Transform {
            translation: Vec3::new(-10., 0., -2.5),
            rotation: Quat::from_xyzw(45., 45., 0., 0.),
            ..Default::default()
        },
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
