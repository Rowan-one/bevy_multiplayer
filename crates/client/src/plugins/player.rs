use bevy::{mesh::CylinderAnchor, prelude::*};
use networking::replication::{NetIdMap, SnapshotBuffer, SnapshotReceiveMessage};
use shared::{components::{Grounded, LocalPlayer}, consts::CLIENT_TICK_RATE, functions::{simulate_player}, messages::ClientTickMessage, resources::{ClientInputBuffer, LocalPlayerNetId, PendingAssignLocalPlayer, PlayerInput, PrevFrameTime, TickAccumulator}};
use avian3d::prelude::*;

pub struct ClientPlayerPlugin;
impl Plugin for ClientPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(PhysicsPlugins::default());
        app.add_plugins(PhysicsDebugPlugin::default());

        app.init_resource::<ClientTick>();
        app.init_resource::<PrevFrameTime>();
        app.init_resource::<TickAccumulator>();
        app.init_resource::<PendingAssignLocalPlayer>();
        app.init_resource::<LocalPlayerNetId>();
        
        app.add_message::<ClientTickMessage>();

        app.add_systems(Update, (
            assign_local_player_system,
            client_tick_system
                .before(avian3d::dynamics::integrator::integrate_positions)
                .before(avian3d::dynamics::integrator::integrate_velocities),

            (
                local_player_movement_system
                //server_reconciliation_system,
            )
            .chain()
            .before(networking::replication::receive_snapshots_system)
            .before(avian3d::dynamics::integrator::integrate_positions)
            .before(avian3d::dynamics::integrator::integrate_velocities)
            .before(avian3d::dynamics::integrator::pre_process_velocity_increments)
            .after(client_tick_system),
        ));
    }
}

#[derive(Debug, Default, Resource)]
pub struct ClientTick(pub u64); 

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

pub fn client_tick_system(
    time: Res<Time>,
    mut prev_frame_time: ResMut<PrevFrameTime>,
    mut accumulator: ResMut<TickAccumulator>,
    mut client_tick: ResMut<ClientTick>,
    mut tick_message_writer: MessageWriter<ClientTickMessage>,
) {
    let current_time = time.elapsed_secs();
    let frame_time = current_time - prev_frame_time.0;
    prev_frame_time.0 = current_time;

    // increment accumulator by frame time
    accumulator.0 += frame_time;

    let mut ticks: u64 = 0;
    
    while accumulator.0 >= CLIENT_TICK_RATE {
        ticks += 1;
        // send tick message
        tick_message_writer.write(ClientTickMessage {
            tick: client_tick.0,
            timestamp: time.elapsed_secs(),
        });

       accumulator.0 -= CLIENT_TICK_RATE;
       client_tick.0 += 1;
    }
}

pub fn local_player_movement_system(
    input: Res<PlayerInput>,
    mut player_position: Single<&mut Position, With<LocalPlayer>>,
    mut player_velocity: Single<&mut LinearVelocity, With<LocalPlayer>>,
    mut player_grounded: Single<&mut Grounded, With<LocalPlayer>>,
    mut tick_message: MessageReader<ClientTickMessage>,
) {
    for message in tick_message.read() {
        simulate_player(&mut player_position, &mut player_velocity, &mut player_grounded, &input, CLIENT_TICK_RATE);
    }
}

pub fn server_reconciliation_system(
    mut player_position: Single<&mut Position, With<LocalPlayer>>,
    mut player_velocity: Single<&mut LinearVelocity, With<LocalPlayer>>,
    mut player_grounded: Single<&mut Grounded, With<LocalPlayer>>,
    mut snapshot_receive_message: MessageReader<SnapshotReceiveMessage>,
    mut client_input_buffer: ResMut<ClientInputBuffer>,
    snapshot_buffer: Res<SnapshotBuffer>,
    local_player_net_id: Res<LocalPlayerNetId>,
) {
    for message in snapshot_receive_message.read() {
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

        for payload in client_input_buffer.0.iter() {
            simulate_player(&mut player_position, &mut player_velocity, &mut player_grounded, &payload.input, CLIENT_TICK_RATE);
        }
    }
}

