use bevy::{color::palettes::css::*, prelude::*};
use bevy_egui::egui::TextBuffer;
use networking::replication::{NetIdMap, SnapshotBuffer, SnapshotReceiveMessage};
use shared::{components::*, consts::*, enums::IKSolverType, functions::*, messages::ClientTickMessage, resources::*};
use bevy_rapier3d::{prelude::*};

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
            init_player_rig_system,
            detect_bones_system,
            (
                update_anim_state_time_system,
                update_player_rotation_system,
                server_reconciliation_system,
                local_player_movement_system,
                sync_player_transform_system,
                draw_player_velocity_system,
                draw_server_position_system,
            ).chain()
                .before(bevy_rapier3d::plugin::PhysicsSet::StepSimulation)
                .before(bevy_rapier3d::plugin::PhysicsSet::SyncBackend)
                .before(bevy_rapier3d::plugin::PhysicsSet::Writeback)
                .before(networking::replication::receive_snapshots_system)
        ));

        app.add_systems(PostUpdate, animate_player_system);
    }
}

#[derive(Debug, Default, Resource)]
pub struct PrevPlayerPos(pub Vec3);

#[derive(Debug, Component)]
pub struct PlayerBoneMap {
    pub shoulder_l: Entity,
    pub shoulder_r: Entity,
    pub upper_arm_l: Entity,
    pub upper_arm_r: Entity,
    pub forearm_l: Entity,
    pub forearm_r: Entity,
    pub hand_l: Entity,
    pub hand_r: Entity,
}

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

pub fn init_player_rig_system(
    mut commands: Commands,
    mut query: Query<(Entity, &PlayerBoneMap), (With<LocalPlayer>, Added<PlayerBoneMap>)>,
) {
    for (entity, bone_map) in query.iter_mut() {
        let target = commands.spawn((IKTarget, Transform::from_xyz(0., 1.5, 0.))).id();
        let rig = IKRig::default()
            .add_chain(
                IKChain::new(target, IKSolverType::TwoBone)
                    .add(IKSegment::new(2.0, Some(bone_map.upper_arm_l)))
                    .add(IKSegment::new(1.0, Some(bone_map.forearm_l)))
                    .add(IKSegment::new(0.5, Some(bone_map.hand_l)))
            );

        commands.entity(entity).insert(rig);
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
    mut player_server_position: Single<&mut ServerPosition, With<LocalPlayer>>,
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
        
        player_server_position.0 = latest_snap.position;

        for payload in client_input_buffer.0.iter() {
            // println!("re-applying unprocessed input");
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
    mut query: Single<(&mut Transform, &CustomPosition, &CustomRotation), With<LocalPlayer>>,
) {
    query.0.translation = query.1.0;
    query.0.rotation = query.2.0;
}

pub fn update_anim_state_time_system(
    time: Res<Time>,
    mut anim_state_time: Single<&mut AnimStateTime, With<LocalPlayer>>,
    player_velocity: Single<&CustomVelocity, With<LocalPlayer>>,
) {
    let scale = player_velocity.0.length() / PLAYER_MOVE_SPEED;
    anim_state_time.0 += time.delta_secs() * scale;
}

pub fn animate_player_system(
    time: Res<Time>,
    player_transform: Single<&Transform, With<LocalPlayer>>,
    ik_rig: Single<&mut IKRig, With<LocalPlayer>>,
    anim_state_time: Single<&AnimStateTime, With<LocalPlayer>>,
    mut transforms: Query<&mut Transform, Without<LocalPlayer>>,
) {
    let mut transform = transforms.get_mut(ik_rig.chains.get(0).unwrap().target).unwrap();
    transform.translation = player_transform.translation
        - (player_transform.right() * 0.5)
        + (player_transform.forward() * ((time.elapsed_secs() * 5.).sin() * 0.5))
        + (player_transform.up() * 0.0)
}

pub fn detect_bones_system(
    mut commands: Commands,
    roots: Query<(Entity, &Children), With<LoadingBoneCache>>,
    names: Query<&Name>,
    children: Query<&Children>,
) {
    for (root, kids) in &roots {
        let mut shoulder_l: Option<Entity> = None;
        let mut shoulder_r: Option<Entity> = None;
        let mut upper_arm_l: Option<Entity> = None;
        let mut upper_arm_r: Option<Entity> = None;
        let mut forearm_l: Option<Entity> = None;
        let mut forearm_r: Option<Entity> = None;
        let mut hand_l: Option<Entity> = None;
        let mut hand_r: Option<Entity> = None;

        let mut stack: Vec<Entity> = kids.iter().collect();
        while let Some(e) = stack.pop() {
            if let Ok(name) = names.get(e) {
                match name.as_str() {
                    "DEF-shoulder.L" => shoulder_l = Some(e),
                    "DEF-shoulder.R" => shoulder_r = Some(e),
                    "DEF-upper_arm.L" => upper_arm_l = Some(e),
                    "DEF-upper_arm.R" => upper_arm_r = Some(e),
                    "DEF-forearm.L" => forearm_l = Some(e),
                    "DEF-forearm.R" => forearm_r = Some(e),
                    "DEF-hand.L" => hand_l = Some(e),
                    "DEF-hand.R" => hand_r = Some(e),
                    _ => { }
                }
            }

            if let Ok(k) = children.get(e) {
                stack.extend(k.iter());
            }

            // make sure we have all of the bones
            if let (
                Some(shoulder_l),
                Some(shoulder_r),
                Some(upper_arm_l),
                Some(upper_arm_r),
                Some(forearm_l),
                Some(forearm_r),
                Some(hand_l),
                Some(hand_r),
            ) = (
                shoulder_l,
                shoulder_r,
                upper_arm_l,
                upper_arm_r,
                forearm_l,
                forearm_r,
                hand_l,
                hand_r,
            ) {
                commands.entity(root)
                    .remove::<LoadingBoneCache>()
                    .insert(PlayerBoneMap {
                        shoulder_l,
                        shoulder_r,
                        upper_arm_l,
                        upper_arm_r,
                        forearm_l,
                        forearm_r,
                        hand_l,
                        hand_r,
                    });
                println!("got all bones");
            }
        }
    }
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

pub fn draw_server_position_system(
    mut gizmos: Gizmos,
    player_server_position: Single<&mut ServerPosition, With<LocalPlayer>>,
) {
    gizmos.sphere(Isometry3d::new(player_server_position.0, Quat::IDENTITY), 1.1, GREEN);
}
