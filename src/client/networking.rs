use std::time::Duration;

use bevy::prelude::*;
use bevy_renet::{renet::{ChannelConfig, DefaultChannel, RenetClient}, *};
use bincode::*;
use crate::shared::events::PlayerInputEvent;    

pub enum ClientChannel {
    Input,
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
                send_type: renet::SendType::ReliableOrdered { resend_time: Duration::ZERO },
            }
        }
    }
}

pub fn send_player_inputs(
    mut client: ResMut<RenetClient>,
    mut ev_input: MessageReader<PlayerInputEvent>
) {
    for event in ev_input.read() {
        println!("Sending event from client");
        let config = bincode::config::standard();
        let bytes = bincode::serde::encode_to_vec(event, config).unwrap();
        client.send_message(ClientChannel::Input, bytes);
    }
}