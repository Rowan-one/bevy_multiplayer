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
    velocity: &mut Velocity,
    dt: f32,
) {
    let a = Vec3::new(0., GRAVITY * dt, 0.);
    velocity.0 += a;
}


pub fn integrate(
    position: &mut Position,
    velocity: &mut Velocity,
    dt: f32,
) {
    position.0 += velocity.0 * dt;
}

pub fn simulate_player(
    position: &mut Position,
    velocity: &mut Velocity,
    grounded: &mut Grounded,
    input: &PlayerInput,
    dt: f32,
) {
    let x = (input.right as i8 - input.left as i8) as f32;
    let y = (input.down as i8 - input.up as i8) as f32;
    let direction = Vec2::new(x, y).normalize_or_zero();
    velocity.0.x = direction.x * PLAYER_MOVE_SPEED;
    velocity.0.z = direction.y * PLAYER_MOVE_SPEED;

    // jump
    if input.jump && grounded.0 {
        velocity.0.y += PLAYER_JUMP_POWER;
        grounded.0 = false;
    }

    // gravity
    if !grounded.0 {
        apply_gravity(velocity, dt);
    }

    integrate(position, velocity, dt);
}
