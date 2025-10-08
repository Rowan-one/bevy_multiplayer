use bevy::prelude::*;
use local_ip_address::local_ip;
use bevy_multiplayer::*;
use bevy_renet::netcode::*;
use bevy_renet::*;
use std::{
    net::{SocketAddr, UdpSocket},
    time::SystemTime,
};
use bincode::*;

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
    let client = RenetClient::new(ConnectionConfig::default());

    (client, transport)
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(RenetClientPlugin);
    app.add_plugins(NetcodeClientPlugin);           

    let (client, transport) = new_renet_client();
    app.insert_resource(client);
    app.insert_resource(transport);

    app.add_systems(Update, client_ping_system);

    app.run();
}

fn client_ping_system(mut client: ResMut<RenetClient>, keyboard: Res<ButtonInput<KeyCode>>) {
    let config = bincode::config::standard();

    if keyboard.just_pressed(KeyCode::Space) {
        let ping_message = bincode::serde::encode_to_vec(&ClientMessage::Ping, config).unwrap();
        client.send_message(DefaultChannel::ReliableOrdered, ping_message);
        info!("Sent ping!")
    }

    while let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
        let server_message: (ServerMessage, usize) = bincode::serde::decode_from_slice(&message, config).unwrap();
        match server_message.0 {
            ServerMessage::Pong => {
                info!("Got pong!");
            }
        }
    }
}