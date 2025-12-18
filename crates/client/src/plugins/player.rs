use bevy::{color::palettes::css::*, prelude::*};
use networking::replication::{NetIdMap, SnapshotBuffer, SnapshotReceiveMessage};
use shared::{components::*, consts::*, functions::*, messages::ClientTickMessage, resources::*};
use bevy_rapier3d::prelude::*;

pub struct ClientPlayerPlugin;
impl Plugin for ClientPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default());
        app.add_plugins(RapierDebugRenderPlugin::default());

        app.insert_resource(TimestepMode::Fixed { dt: CLIENT_TICK_RATE, substeps: 5 });

        app.init_resource::<PendingAssignLocalPlayer>();
        app.init_resource::<LocalPlayerNetId>();
        app.init_resource::<PrevPlayerPos>();

        app.add_message::<ClientTickMessage>();

        app.add_systems(Update, (
            assign_local_player_system,
            (
                update_player_rotation_system,
                local_player_movement_system,
                server_reconciliation_system,
                sync_player_transform_system,
                draw_player_velocity_system,
            ).chain()
                .before(bevy_rapier3d::plugin::PhysicsSet::StepSimulation)
                .before(bevy_rapier3d::plugin::PhysicsSet::SyncBackend)
                .before(bevy_rapier3d::plugin::PhysicsSet::Writeback)
                .before(networking::replication::receive_snapshots_system)
        ));
    }
}

#[derive(Debug, Default, Resource)]
pub struct PrevPlayerPos(pub Vec3);

pub fn assign_local_player_system(
    mut commands: Commands,
    mut pending_assign: ResMut<PendingAssignLocalPlayer>,
    net_id_map: Res<NetIdMap>,
) {
    // check if pending
    if let Some(assign_local_player) = &pending_assign.0 {
        // get player entity
        let Some(player_entity) = net_id_map.0.get(&assign_local_player.player_net_id) else { return ; };

        println!("assigning local player");
        commands.entity(*player_entity).insert(LocalPlayer);

        // no longer pending
        pending_assign.0 = None;
    }
}

pub fn update_player_rotation_system(
    mut query: Single<&mut CustomRotation, With<LocalPlayer>>,
    camera_transform: Single<&Transform, With<Camera3d>>,
) {
    query.0.y = camera_transform.rotation.y;
}

pub fn local_player_movement_system(
    rapier_context: ReadRapierContext,
    input: Res<PlayerInput>,
    mut prev_player_pos: ResMut<PrevPlayerPos>,
    mut player_position: Single<&mut CustomPosition, With<LocalPlayer>>,
    mut player_velocity: Single<&mut CustomVelocity, With<LocalPlayer>>,
    mut player_rotation: Single<&mut CustomRotation, With<LocalPlayer>>,
    mut player_wishdir: Single<&mut WishDir, With<LocalPlayer>>,
    mut player_grounded: Single<&mut Grounded, With<LocalPlayer>>,
    player_resting_height: Single<&mut RestingHeight, With<LocalPlayer>>,
    mut player_gravity: Single<&mut Gravity, With<LocalPlayer>>,
    mut tick_message: MessageReader<ClientTickMessage>,
) {
    for _tick in tick_message.read() {
        prev_player_pos.0 = player_position.0;
        (player_position.0, player_velocity.0, player_wishdir.0, player_rotation.0) = simulate_player(
            &rapier_context.single().unwrap(),
            player_position.0,
            player_rotation.0,
            player_velocity.0,
            &mut player_gravity,
            &mut player_grounded,
            player_resting_height.0,
            &input, 
            CLIENT_TICK_RATE,
        );
    }
}

pub fn server_reconciliation_system(
    rapier_context: ReadRapierContext,
    mut player_position: Single<&mut CustomPosition, With<LocalPlayer>>,
    mut player_rotation: Single<&mut CustomRotation, With<LocalPlayer>>,
    mut player_velocity: Single<&mut CustomVelocity, With<LocalPlayer>>,
    mut player_wishdir: Single<&mut WishDir, With<LocalPlayer>>,
    mut player_grounded: Single<&mut Grounded, With<LocalPlayer>>,
    mut player_gravity: Single<&mut Gravity, With<LocalPlayer>>,
    player_resting_height: Single<&mut RestingHeight, With<LocalPlayer>>,
    mut snapshot_receive_message: MessageReader<SnapshotReceiveMessage>,
    mut client_input_buffer: ResMut<ClientInputBuffer>,
    snapshot_buffer: Res<SnapshotBuffer>,
    local_player_net_id: Res<LocalPlayerNetId>,
) {
    for _message in snapshot_receive_message.read() {
        // make sure local player id resource is initialized
        let Some(local_player_id) = local_player_net_id.0 else { return; };
        let Some(local_player_buf) = snapshot_buffer.0.get(&local_player_id) else { return; };
        // TODO: fix order of interpolation buffer so latest sample is at index 0
        let Some(latest_snap) = local_player_buf.get(local_player_buf.len()-1) else { return; };

        // get last processed input sequence and re-compute inputs from that point
        let last_processed_seq = latest_snap.last_processed_seq;

        client_input_buffer.0.retain(|i| i.seq > last_processed_seq);
        client_input_buffer.0.sort_by_key(|i| i.seq);

        (player_position.0, player_velocity.0) = (latest_snap.position, latest_snap.velocity);
        if let Some(gravity) = latest_snap.gravity { // this should exist
            player_gravity.vector = gravity;
        }

        for payload in client_input_buffer.0.iter() {
            (player_position.0, player_velocity.0, player_wishdir.0, player_rotation.0) = simulate_player(
                &rapier_context.single().unwrap(),
                player_position.0,
                player_rotation.0,
                player_velocity.0,
                &mut player_gravity,
                &mut player_grounded,
                player_resting_height.0,
                &payload.input,
                CLIENT_TICK_RATE,
            );
        }
    }
}

pub fn sync_player_transform_system(
    mut query: Single<(&mut Transform, &CustomPosition), With<LocalPlayer>>,
) {
    query.0.translation = query.1.0;
}

pub fn draw_player_velocity_system(
    mut gizmos: Gizmos,
    player_velocity: Single<&mut CustomVelocity, With<LocalPlayer>>,
    player_wishdir: Single<&mut WishDir, With<LocalPlayer>>,
    player_position: Single<&mut CustomPosition, With<LocalPlayer>>,
) {
    gizmos.arrow(player_position.0, player_position.0 + player_velocity.0, GREEN);
    gizmos.arrow(player_position.0, player_position.0 + player_wishdir.0, BLUE);
}
