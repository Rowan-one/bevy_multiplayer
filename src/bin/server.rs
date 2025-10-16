use std::net::UdpSocket;
use std::time::SystemTime;
use bevy::prelude::*;
use bevy_egui::*;
use bevy_multiplayer::server::ServerPlugin;
use renet_visualizer::*;
use bevy_renet::netcode::*;
use bevy_renet::RenetServerPlugin;
use bevy_renet::renet::RenetServer;
use bevy_multiplayer::*;
use crate::server::networking::*;
use crate::shared::networking::connection_config;
use crate::server::messages::ClientConnectMessage;
use crate::shared::networking::setup_level;

fn new_renet_server() -> (RenetServer, NetcodeServerTransport) {
    let public_addr = "127.0.0.1:5000".parse().unwrap();
    let socket = UdpSocket::bind(public_addr).unwrap();
    let current_time = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH).unwrap();
    let server_config = ServerConfig {
        current_time,
        max_clients: 64,
        protocol_id: PROTOCOL_ID,
        public_addresses: vec![public_addr],
        authentication: ServerAuthentication::Unsecure,
    };

    let transport = NetcodeServerTransport::new(server_config, socket).unwrap();
    let server = RenetServer::new(connection_config());

    (server, transport)
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_plugins(RenetServerPlugin);
    app.add_plugins(NetcodeServerPlugin);
    app.add_plugins(EguiPlugin::default());

    app.add_plugins(ServerPlugin);

    app.insert_resource(RenetServerVisualizer::<200>::default());
    app.insert_resource(ServerLobby::default());

    let (server, transport) = new_renet_server();
    app.insert_resource(server);
    app.insert_resource(transport);

    app.add_systems(Startup, (
        setup_simple_camera,
    ));

    app.add_systems(EguiPrimaryContextPass, update_visualizer_system);

    app.add_message::<ClientConnectMessage>();

    app.run();
}

fn update_visualizer_system(
    mut egui_contexts: EguiContexts,
    mut visualizer: ResMut<RenetServerVisualizer<200>>,
    server: Res<RenetServer>,
) -> Result<()> {
    visualizer.update(&server);
    visualizer.show_window(egui_contexts.ctx_mut()?);
    Ok(())
}

pub fn setup_simple_camera(mut commands: Commands) {
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-20.5, 30.0, 20.5).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

