use bevy::{color::palettes::css::{GREEN, RED}, prelude::*, render::alpha};
use networking::replication::{NetIdMap, SnapshotBuffer, SnapshotReceiveMessage};
use shared::{components::{CustomPosition, CustomVelocity, Grounded, LocalPlayer, RestingHeight}, consts::{CLIENT_TICK_RATE, MAX_FRAME_TIME}, functions::{integrate, simulate_player, spring_damper}, messages::ClientTickMessage, resources::{ClientInputBuffer, LocalPlayerNetId, PendingAssignLocalPlayer, PlayerInput, PrevFrameTime, TickAccumulator}};
use bevy_rapier3d::{prelude::*, rapier};

pub struct ClientPlayerPlugin;
impl Plugin for ClientPlayerPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::in_fixed_schedule(RapierPhysicsPlugin::default()));
        app.add_plugins(RapierDebugRenderPlugin::default());

        app.insert_resource(TimestepMode::Fixed { dt: CLIENT_TICK_RATE, substeps: 10 });

        app.init_resource::<ClientTick>();
        app.init_resource::<TickAlpha>();
        app.init_resource::<PrevFrameTime>();
        app.init_resource::<TickAccumulator>();
        app.init_resource::<PendingAssignLocalPlayer>();
        app.init_resource::<LocalPlayerNetId>();
        app.init_resource::<PrevPlayerPos>();

        app.add_message::<ClientTickMessage>();

        app.add_systems(PreUpdate, client_tick_system);

        app.add_systems(Update, (
            assign_local_player_system,
            (
                local_player_movement_system,
                server_reconciliation_system,
                ground_check_system,
                integrate_player_system,
                sync_player_transform_system,
                draw_player_velocity_system,
            ).chain()
                // .before(bevy_rapier3d::plugin::PhysicsSet::StepSimulation)
                // .before(bevy_rapier3d::plugin::PhysicsSet::SyncBackend)
                // .before(bevy_rapier3d::plugin::PhysicsSet::Writeback)
                // .before(networking::replication::receive_snapshots_system)
        ));
    }
}

#[derive(Debug, Default, Resource)]
pub struct ClientTick(pub u64); 

#[derive(Debug, Default, Resource)]
pub struct TickAlpha(pub f32);

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

pub fn interpolate_local_player_system(
    tick_alpha: Res<TickAlpha>,
    prev_player_pos: Res<PrevPlayerPos>,
    mut player_query: Single<(&mut Transform, &CustomPosition), With<LocalPlayer>>,
) {
    let alpha = tick_alpha.0;
    let current_pos = player_query.1.0;
    let prev_pos = prev_player_pos.0;

    let interpolated_pos = current_pos*alpha + prev_pos*(1.0-alpha);
    player_query.0.translation = interpolated_pos;
}

pub fn client_tick_system(
    time: Res<Time>,
    mut tick_alpha: ResMut<TickAlpha>,
    mut prev_frame_time: ResMut<PrevFrameTime>,
    mut accumulator: ResMut<TickAccumulator>,
    mut client_tick: ResMut<ClientTick>,
    mut tick_message_writer: MessageWriter<ClientTickMessage>,
) {
    let current_time = time.elapsed_secs();
    let mut frame_time = current_time - prev_frame_time.0;
    if frame_time > MAX_FRAME_TIME {
        frame_time = MAX_FRAME_TIME;
    }
    prev_frame_time.0 = current_time;

    // increment accumulator by frame time
    accumulator.0 += frame_time;

    let mut ticks: u8 = 0;
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
    let alpha: f32 = accumulator.0 / CLIENT_TICK_RATE;
    tick_alpha.0 = alpha;
}

pub fn local_player_movement_system(
    input: Res<PlayerInput>,
    mut prev_player_pos: ResMut<PrevPlayerPos>,
    mut player_position: Single<&mut CustomPosition, With<LocalPlayer>>,
    mut player_velocity: Single<&mut CustomVelocity, With<LocalPlayer>>,
    mut player_grounded: Single<&mut Grounded, With<LocalPlayer>>,
    mut tick_message: MessageReader<ClientTickMessage>,
) {
    for _tick in tick_message.read() {
        prev_player_pos.0 = player_position.0;
        (player_position.0, player_velocity.0) = simulate_player(
            player_position.0,
            player_velocity.0,
            &mut player_grounded,
            &input, 
            CLIENT_TICK_RATE,
        );
    }
}

pub fn server_reconciliation_system(
    mut player_position: Single<&mut CustomPosition, With<LocalPlayer>>,
    mut player_velocity: Single<&mut CustomVelocity, With<LocalPlayer>>,
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
            (player_position.0, player_velocity.0) = simulate_player(
                player_position.0,
                player_velocity.0,
                &mut player_grounded,
                &payload.input,
                CLIENT_TICK_RATE,
            );
        }
    }
}

pub fn ground_check_system(
    rapier_context: ReadRapierContext,
    mut query: Query<(&CustomPosition, &mut CustomVelocity, &mut Grounded, &RestingHeight)>,
    mut client_tick_message: MessageReader<ClientTickMessage>,
) {
    for _tick in client_tick_message.read() {
        let rapier_context = rapier_context.single().unwrap();

        for (position, mut velocity, mut grounded, resting_height) in query.iter_mut() {
            if let Some(result) = rapier_context.cast_ray(position.0, Vec3::NEG_Y, resting_height.0, true, QueryFilter::only_fixed()) {
                grounded.0 = true;

                let x = resting_height.0 - result.1;
                let x_dot = velocity.0.y;

                let spring_force = spring_damper(1.0, 15.0, x, x_dot, CLIENT_TICK_RATE);
                velocity.0.y += spring_force;

                continue;
            }

            grounded.0 = false;
        }
    }
}

pub fn integrate_player_system(
    mut gizmos: Gizmos,
    rapier_context: ReadRapierContext,
    mut tick_message: MessageReader<ClientTickMessage>,
    mut player_position: Single<&mut CustomPosition, With<LocalPlayer>>,
    player_velocity: Single<&mut CustomVelocity, With<LocalPlayer>>,
) {
    for _tick in tick_message.read() {
        let prev_pos = player_position.0;
        player_position.0 = integrate(&rapier_context.single().unwrap(), player_position.0, player_velocity.0, CLIENT_TICK_RATE);

        gizmos.arrow(prev_pos, prev_pos + (player_position.0 - prev_pos).normalize_or_zero() * 2.0, RED);
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
    player_position: Single<&mut CustomPosition, With<LocalPlayer>>,
) {
    gizmos.arrow(player_position.0 + Vec3::new(0., -1.5, 0.), player_position.0 + player_velocity.0 + Vec3::new(0., -1.5, 0.), GREEN);
}
