use bevy::prelude::*;
use local_ip_address::local_ip;
use bevy_multiplayer::{shared::networking::connection_config, *};
use bevy_renet::netcode::*;
use std::{
    net::UdpSocket,
    time::SystemTime,
};
use shared::events::PlayerInputEvent;
use client::networking::send_player_inputs;

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

    let (client, transport) = new_renet_client();
    app.insert_resource(client);
    app.insert_resource(transport);

    app.add_systems(Startup, (setup_level, setup_camera));
    app.add_systems(Update, (send_input_system, send_player_inputs));

    app.add_message::<PlayerInputEvent>();

    app.run();
}

fn send_input_system(mut client: ResMut<RenetClient>, keyboard: Res<ButtonInput<KeyCode>>, mut event_writer: MessageWriter<PlayerInputEvent>) {
    if keyboard.just_pressed(KeyCode::KeyE) {
        println!("Writing input event");
        event_writer.write(PlayerInputEvent { direction: Vec2::new(1.,0.) });
    }
}

fn setup_camera(mut commands: Commands) {
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0., 8.0, 2.5).looking_at(Vec3::new(0.0, 0.5, 0.0), Vec3::Y),
    ));
}