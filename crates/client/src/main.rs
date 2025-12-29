use bevy::{diagnostic::FrameTimeDiagnosticsPlugin, prelude::*, window::CursorOptions};
use client::plugins::{camera::ClientCameraPlugin, core::ClientCorePlugin, input::ClientInputPlugin, player::ClientPlayerPlugin};
use networking::client::NetClientPlugin;
use game::setup_level;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_cursor_options: Some(CursorOptions {
                grab_mode: bevy::window::CursorGrabMode::Locked,
                ..Default::default()
            }),
            ..Default::default()
        }))

        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins((
            ClientCorePlugin,
            ClientInputPlugin,
            ClientCameraPlugin,
            ClientPlayerPlugin,
            NetClientPlugin,
        ))

        .add_systems(Startup, setup_level)
        .run();
}
