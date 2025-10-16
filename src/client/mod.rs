pub mod networking;

use bevy::prelude::*;
use crate::{client::networking::*, shared::networking::setup_level};

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, (
            setup_level,
        ));
        app.add_systems(Update, (
            update_player_input,
            send_player_input,
        ));
    }
}