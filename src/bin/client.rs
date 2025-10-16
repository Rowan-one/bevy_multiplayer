use bevy::prelude::*;
use bevy_multiplayer::client::ClientPlugin;
use bevy_multiplayer::shared::player::PlayerInput;
use local_ip_address::local_ip;
use bevy_multiplayer::{shared::networking::connection_config, *};
use bevy_renet::netcode::*;
use renet_visualizer::{RenetClientVisualizer, RenetVisualizerStyle};
use std::{
    net::UdpSocket,
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
    let client = RenetClient::new(connection_config());

    (client, transport)
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(RenetClientPlugin);
    app.add_plugins(NetcodeClientPlugin);           

    app.add_plugins(ClientPlugin);

    app.insert_resource(RenetClientVisualizer::<200>::new(RenetVisualizerStyle::default()));
    app.insert_resource(PlayerInput::default());

    let (client, transport) = new_renet_client();
    app.insert_resource(client);
    app.insert_resource(transport);

    app.add_systems(Startup, (
        setup_camera,
    ));

    app.run();
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0., 16.0, 2.5).looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
    ));
}
