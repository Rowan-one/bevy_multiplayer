use glam::{Vec2, Vec3};
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
    position: Vec3,
    velocity: Vec3,
    dt: f32,
) -> Vec3 {
    position + velocity * dt
}

pub fn simulate_player(
    mut position: Vec3,
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

    //velocity = apply_gravity(velocity, dt);
    position = integrate(position, velocity, dt);

    (position, velocity)
}
