use std::{net::UdpSocket, time::SystemTime};
use bevy::prelude::*;
use bevy_renet::{netcode::{ClientAuthentication, NetcodeClientPlugin, NetcodeClientTransport}, renet::RenetClient, RenetClientPlugin};
use super::protocol::*;
use shared::components::PlayerInput;

pub struct NetClientPlugin;

impl Plugin for NetClientPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RenetClientPlugin);
        app.add_plugins(NetcodeClientPlugin);

        let (client, transport) = new_renet_client();
        app.insert_resource(client);
        app.insert_resource(transport);

        app.add_systems(Update, send_player_input);
    }
}

fn new_renet_client() -> (RenetClient, NetcodeClientTransport) {
    let server_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let client_id = current_time.as_millis() as u64;
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr,
        user_data: None,
    };

    let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();
    let client = RenetClient::new(connection_config());

    (client, transport)
}

fn send_player_input(
    mut client: ResMut<RenetClient>,
    input: Res<PlayerInput>,
) {
    let config = bincode::config::standard();
    let bytes = bincode::serde::encode_to_vec(&*input, config).unwrap();
    client.send_message(ClientChannel::Input, bytes);
}
