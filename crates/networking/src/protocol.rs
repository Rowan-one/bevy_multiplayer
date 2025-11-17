use std::time::Duration;
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon_renet::{renet::*, RenetChannelsExt};

pub const PROTOCOL_ID: u64 = 1000;

pub fn connection_config(channels: Res<RepliconChannels>) -> ConnectionConfig {
    let mut server_cfg = channels.server_configs();
    let mut client_cfg = channels.client_configs();

    server_cfg.extend(ServerChannel::channels_config());
    client_cfg.extend(ClientChannel::channels_config());

    ConnectionConfig {
        available_bytes_per_tick: 1024 * 1024,
        server_channels_config: server_cfg,
        client_channels_config: client_cfg,

        ..Default::default()
    }
}

pub enum ServerChannel {
    Replication,
    AssignLocalPlayer,
}

impl From<ServerChannel> for u8 {
    fn from(value: ServerChannel) -> Self {
        match value {
            ServerChannel::Replication => 3,
            ServerChannel::AssignLocalPlayer => 4,
        }
    }
}

impl ServerChannel {
    pub fn channels_config() -> Vec<ChannelConfig> {
        vec!{
            ChannelConfig {
                channel_id: Self::Replication.into(),
                max_memory_usage_bytes: 5 * 1024 * 1024,
                send_type: SendType::Unreliable,
            },
            ChannelConfig {
                channel_id: Self::AssignLocalPlayer.into(),
                max_memory_usage_bytes: 5 * 1024 * 1024,
                send_type: SendType::ReliableUnordered { resend_time: Duration::ZERO }
            }
        }
    }
}

pub enum ClientChannel {
    Input,
}

impl From<ClientChannel> for u8 {
    fn from(value: ClientChannel) -> Self {
        match value {
            ClientChannel::Input => 3
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
