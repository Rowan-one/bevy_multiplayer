use bevy::prelude::*;
use bevy_renet::renet::ClientId;

#[derive(Message)]
pub struct ClientConnectMessage {
    pub client_id: ClientId,
}