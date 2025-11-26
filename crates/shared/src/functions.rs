use glam::Vec2;
use crate::{components::{Position, Velocity}, consts::PLAYER_MOVE_SPEED, resources::PlayerInput};

pub fn integrate(
    position: &mut Position,
    velocity: &mut Velocity,
    dt: f32,
) {
    position.0 += velocity.0 * dt;
}

pub fn update_player_velocity(
    velocity: &mut Velocity,
    input: &PlayerInput,
) {
    let x = (input.right as i8 - input.left as i8) as f32;
    let y = (input.down as i8 - input.up as i8) as f32;
    let direction = Vec2::new(x, y).normalize_or_zero();
    velocity.0.x = direction.x * PLAYER_MOVE_SPEED;
    velocity.0.z = direction.y * PLAYER_MOVE_SPEED;
}
