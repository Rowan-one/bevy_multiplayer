use bevy_renet::renet::*;
use crate::client::networking::ClientChannel;

pub fn connection_config() -> ConnectionConfig {
    ConnectionConfig {
        available_bytes_per_tick: 1024 * 1024,
        client_channels_config: ClientChannel::channels_config(),
        ..Default::default()
    }
}