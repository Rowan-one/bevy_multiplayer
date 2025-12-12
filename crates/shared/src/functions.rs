use glam::{Quat, Vec3};
use bevy_rapier3d::{parry::shape::Shape, prelude::*};
use crate::{components::*, consts::*, resources::PlayerInput};

pub fn project_and_scale(v: Vec3, n: Vec3) -> Vec3 {
    let mag: f32 = v.length();
    project_onto_plane(v, n).normalize_or_zero() * mag
}

pub fn project_onto_plane(
    v: Vec3,
    n: Vec3,
) -> Vec3 {
    v - n * v.dot(n)
}

pub fn spring_damper(
    k: f32, 
    c: f32, 
    x: f32, 
    x_dot: f32,
    dt: f32,
) -> f32 {
    let f = (k*x) + (c*-x_dot) * dt;
    f.max(0.0)
}

pub fn apply_gravity(
    velocity: Vec3,
    scale: f32,
    dt: f32,
) -> Vec3 {
    let a = Vec3::new(0., GRAVITY, 0.) * scale * dt;
    velocity + a
}

pub fn integrate(
    position: Vec3,
    velocity: Vec3,
    dt: f32,
) -> Vec3 {
    position + velocity
}

pub fn collide_and_slide(
    position: Vec3,
    velocity: Vec3,
    shape: &dyn Shape,
    depth: u8,
    gravity_pass: bool,
    vel_init: Vec3,
    rapier_context: &RapierContext,
) -> Vec3 {
    // check if exceeded max depth
    if depth >= MAX_COLLISION_BOUNCES { return Vec3::ZERO }

    let dist: f32 = velocity.length() + SKIN_WIDTH;

    if let Some(result) = rapier_context.cast_shape(position, Quat::IDENTITY, velocity.normalize(), shape, ShapeCastOptions::with_max_time_of_impact(dist), QueryFilter::only_fixed()) {
        let hit = result.1;
        let normal = hit.details.unwrap().normal1;

        let mut snap_to_surface: Vec3 = velocity.normalize_or_zero() * (hit.time_of_impact - SKIN_WIDTH);
        let mut leftover: Vec3 = velocity - snap_to_surface;
        let angle: f32 = Vec3::angle_between(Vec3::Y, normal).to_degrees();

        if snap_to_surface.length() <= SKIN_WIDTH {
            snap_to_surface = Vec3::ZERO;
        }

        // normal ground / slope
        if angle <= MAX_SLOPE_ANGLE {
            if gravity_pass {
                return snap_to_surface;
            }

            leftover = project_and_scale(leftover, normal);
        } else {
            let scale: f32 = 1.0 - Vec3::dot(
                Vec3::new(normal.x, 0., normal.z).normalize_or_zero(),
                -Vec3::new(vel_init.x, 0., vel_init.z).normalize_or_zero()
            );

            leftover = project_and_scale(leftover, normal) * scale;
        }

        return snap_to_surface + collide_and_slide(position + snap_to_surface, leftover, shape, depth+1, gravity_pass, vel_init, rapier_context);
    }
    
    velocity
}

pub fn simulate_player(
    position: Vec3,
    mut velocity: Vec3,
    gravity: &mut Gravity,
    grounded: &mut Grounded,
    input: &PlayerInput,
    dt: f32,
) -> (Vec3, Vec3) { // (position, velocity)
    let target_speed_x = PLAYER_MOVE_SPEED * (input.right as i8 - input.left as i8) as f32;
    let target_speed_z = PLAYER_MOVE_SPEED * (input.down as i8 - input.up as i8) as f32;
    let speed_diff_x = target_speed_x - velocity.x;
    let speed_diff_z = target_speed_z - velocity.z;
    let x = speed_diff_x * PLAYER_ACCEL * dt;
    let z = speed_diff_z * PLAYER_ACCEL * dt;

    velocity += Vec3::new(x, 0., z);

    // jump
    if input.jump && grounded.0 {
        gravity.vector.y += PLAYER_JUMP_POWER;
        grounded.0 = false;
    }

    // apply gravity
    gravity.vector = apply_gravity(gravity.vector, gravity.scale, dt);

    (position, velocity)
}
