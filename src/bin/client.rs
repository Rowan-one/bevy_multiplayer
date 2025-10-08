use bevy::{prelude::*, state::app};
use local_ip_address::local_ip;
use bevy_multiplayer::*;
use bevy_renet::{netcode::{ClientAuthentication, NetcodeClientPlugin, NetcodeClientTransport}, *};
use std::{
    net::{SocketAddr, UdpSocket},
    time::SystemTime,
};

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

    app.run();
}

