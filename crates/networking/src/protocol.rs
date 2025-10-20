use std::time::Duration;
use bevy_renet::renet::*;

pub const PROTOCOL_ID: u64 = 1000;

pub fn connection_config() -> ConnectionConfig {
    ConnectionConfig {
        available_bytes_per_tick: 1024 * 1024,
        client_channels_config: ClientChannel::channels_config(),
        ..Default::default()
    }
}

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
                send_type: SendType::ReliableOrdered { resend_time: Duration::ZERO },
            }
        }
    }
}