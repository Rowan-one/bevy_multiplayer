use bevy::{color::palettes::css::{ORANGE, RED, YELLOW}, prelude::*};
use shared::{components::*, consts::*, enums::IKSolverType, functions::{quat_from_to, solve_two_bone_ik}, messages::ClientTickMessage, resources::*};

pub struct ClientCorePlugin;
impl Plugin for ClientCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Settings>();
        app.init_resource::<ClientTick>();
        app.init_resource::<PrevFrameTime>();
        app.init_resource::<TickAccumulator>();

        app.add_systems(PreUpdate, client_tick_system);
        app.add_systems(PostUpdate, apply_two_bone_ik_system);
    }
}

#[derive(Debug, Resource)]
pub struct Settings {
    pub sensitivity: f32,
}
impl Default for Settings {
    fn default() -> Self {
        Settings { sensitivity: 3.0 }
    }
}

#[derive(Debug, Default, Resource)]
pub struct ClientTick(pub u64); 

pub fn client_tick_system(
    time: Res<Time>,
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

    while accumulator.0 >= CLIENT_TICK_RATE {
        // send tick message
        tick_message_writer.write(ClientTickMessage {
            tick: client_tick.0,
            timestamp: time.elapsed_secs(),
        });

       accumulator.0 -= CLIENT_TICK_RATE;
       client_tick.0 += 1;
    }
}

const BONE_FWD_LOCAL: Vec3 = Vec3::Y;
pub fn apply_two_bone_ik_system(
    mut gizmos: Gizmos,
    mut ik_rigs: Query<&mut IKRig>,
    parents: Query<&ChildOf>,
    globals: Query<&GlobalTransform>,
    mut locals: Query<&mut Transform>,
) {
    fn world_to_local_rot(
        bone: Entity,
        desired_world_rot: Quat,
        parents: &Query<&ChildOf>,
        globals: &Query<&GlobalTransform>,
    ) -> Quat {
        let parent_world_rot = if let Ok(parent) = parents.get(bone) {
            globals.get(parent.0).map(|g| g.rotation()).unwrap_or(Quat::IDENTITY)
        } else {
            Quat::IDENTITY
        };

        parent_world_rot.inverse() * desired_world_rot
    }

    for rig in ik_rigs.iter_mut() {
        for chain in &rig.chains {
            // only solve 2 bone in this system
            if chain.solve_type == IKSolverType::Fabrik { continue; }

            let Some(upper) = chain.segments.get(0) else { continue; };
            let Some(mid) = chain.segments.get(1) else { continue; };
            let Some(end) = chain.segments.get(2) else { continue; };

            // get joint world positions
            let a = globals.get(upper.attached_object.unwrap()).unwrap().translation();
            let b = globals.get(mid.attached_object.unwrap()).unwrap().translation();
            let c = globals.get(end.attached_object.unwrap()).unwrap().translation();
            let t = globals.get(chain.target).unwrap().translation();

            let (b_final, c_final) = solve_two_bone_ik(a, b, c, t);

            // solve rotations
            let upper_gt = globals.get(upper.attached_object.unwrap()).unwrap();
            let fore_gt = globals.get(mid.attached_object.unwrap()).unwrap();

            let desired_upper_dir = (b_final - a).normalize();
            let desired_fore_dir = (c_final - b_final).normalize();

            let upper_fwd_world = upper_gt.rotation() * BONE_FWD_LOCAL;
            let fore_fwd_world = fore_gt.rotation() * BONE_FWD_LOCAL;

            let upper_delta = quat_from_to(upper_fwd_world, desired_upper_dir);
            let fore_delta = quat_from_to(fore_fwd_world, desired_fore_dir);

            let upper_world_desired = upper_delta * upper_gt.rotation();
            let fore_world_desired = fore_delta * fore_gt.rotation();

            let upper_local_desired = world_to_local_rot(upper.attached_object.unwrap(), upper_world_desired, &parents, &globals);
            let fore_local_desired = world_to_local_rot(mid.attached_object.unwrap(), fore_world_desired, &parents, &globals);

            locals.get_mut(upper.attached_object.unwrap()).unwrap().rotation = upper_local_desired;
            locals.get_mut(mid.attached_object.unwrap()).unwrap().rotation = fore_local_desired;
        }
    }
}
