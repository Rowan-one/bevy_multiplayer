use bevy_rapier3d::{parry::shape::*, plugin::RapierContext, prelude::*};
use glam::{Quat, Vec2, Vec3};
use crate::{components::*, consts::*, resources::PlayerInput};

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
    dt: f32,
) -> Vec3 {
    let a = Vec3::new(0., GRAVITY, 0.) * dt;
    velocity + a
}

pub fn integrate(
    rapier_context: &RapierContext,
    position: Vec3,
    mut velocity: Vec3,
    dt: f32,
) -> Vec3 {
    velocity *= dt;
    velocity = collide_and_slide_player(position, velocity, 0, rapier_context);

    position + velocity
}

pub fn collide_and_slide_player(
    position: Vec3,
    velocity: Vec3,
    depth: u8,
    rapier_context: &RapierContext,
) -> Vec3 {
    // check if exceeded max depth
    if depth >= MAX_COLLISION_BOUNCES { println!("reached max depth"); return Vec3::ZERO }

    let shape = Ball::new(1.0 - SKIN_WIDTH);
    let dist: f32 = velocity.length() + SKIN_WIDTH;

    if let Some(result) = rapier_context.cast_shape(
        position,
        Quat::default(),
        velocity,
        &shape,
        ShapeCastOptions::with_max_time_of_impact(1.0), 
        QueryFilter::only_fixed(),
    ) {
        let hit = result.1;
        let mut snap_to_surface: Vec3 = velocity.normalize() * (hit.time_of_impact - SKIN_WIDTH);
        if snap_to_surface.length_squared() <= f32::EPSILON {
            return Vec3::ZERO;
        }

        let mut leftover: Vec3 = velocity - snap_to_surface;

        if snap_to_surface.length() <= SKIN_WIDTH {
            println!("snap to surface smaller than skin width");
            snap_to_surface = Vec3::ZERO;
        }

        let mag: f32 = leftover.length();
        leftover = leftover.project_onto(hit.details.unwrap().normal1).normalize() * mag;

        return snap_to_surface + collide_and_slide_player(position + snap_to_surface, leftover, depth+1, rapier_context);
    }
    
    velocity
}

pub fn simulate_player(
    position: Vec3,
    mut velocity: Vec3,
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
        velocity.y += PLAYER_JUMP_POWER;
        grounded.0 = false;
    }

    velocity = apply_gravity(velocity, dt);
    // position = integrate(rapier_context, position, velocity, dt);

    (position, velocity)
}
