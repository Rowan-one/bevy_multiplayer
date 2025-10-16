pub mod networking;
pub mod player;
pub mod messages;

use bevy::prelude::*;

use crate::{server::{networking::*, player::*}, shared::networking::setup_level};

pub struct ServerPlugin;

impl Plugin for ServerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (
            setup_level,
        ));

        app.add_systems(Update, (
            handle_events_system,
            receive_player_input,
            spawn_player_system,
            update_player_velocity,
            apply_velocity_system
        ));
    }
}