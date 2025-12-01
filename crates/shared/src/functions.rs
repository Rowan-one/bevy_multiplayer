use glam::{Vec2, Vec3};
use crate::{components::*, consts::*, resources::PlayerInput};
use avian3d::prelude::{Position, LinearVelocity};

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
    velocity: &mut LinearVelocity,
    dt: f32,
) {
    let a = Vec3::new(0., GRAVITY * dt, 0.);
    velocity.0 += a;
}

pub fn simulate_player(
    position: &mut Position,
    velocity: &mut LinearVelocity,
    grounded: &mut Grounded,
    input: &PlayerInput,
    dt: f32,
) {
    let target_speed_x = PLAYER_MOVE_SPEED * (input.right as i8 - input.left as i8) as f32;
    let target_speed_z = PLAYER_MOVE_SPEED * (input.down as i8 - input.up as i8) as f32;
    let speed_diff_x = target_speed_x - velocity.0.x;
    let speed_diff_z = target_speed_z - velocity.0.z;
    let x = speed_diff_x * PLAYER_ACCEL * dt;
    let z = speed_diff_z * PLAYER_ACCEL * dt;
    println!("new x vel: {}",x);

    velocity.0 += Vec3::new(x, 0., z);

    // jump
    if input.jump && grounded.0 {
        velocity.0.y += PLAYER_JUMP_POWER;
        grounded.0 = false;
    }

    // gravity
    if !grounded.0 {
        // apply_gravity(velocity, dt);
    }
}
