use bevy::prelude::*;
use client::plugins::{camera::ClientCameraPlugin, input::ClientInputPlugin};
use networking::client::NetClientPlugin;
use game::setup_level;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins((
            ClientInputPlugin,
            ClientCameraPlugin,
            NetClientPlugin,
        ))

        .add_systems(Startup, setup_level)
        .run();
}