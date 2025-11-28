use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use shared::{components::{Grounded, Position, RestingHeight}, consts::SERVER_TICK_RATE, functions::spring_damper, messages::ServerTickMessage};

fn ground_check(
    rapier_context: &RapierContext,
    origin: Vec3,
    distance: f32,
) -> bool {
    if let Some(result) = rapier_context.cast_ray(origin, Vec3::NEG_Y, distance, true, QueryFilter::default()) {
        return true;
    }

    return false;
}

pub fn ground_check_system(
    rapier_context: ReadRapierContext,
    mut query: Query<(&Position, &mut shared::components::Velocity, &mut Grounded, &RestingHeight)>,
    mut server_tick_message: MessageReader<ServerTickMessage>,
) {
    for _tick in server_tick_message.read() {
        let rapier_context = rapier_context.single().unwrap();

        for (position, mut velocity, mut grounded, resting_height) in query.iter_mut() {
            if let Some(result) = rapier_context.cast_ray(position.0, Vec3::NEG_Y, resting_height.0, true, QueryFilter::default()) {
                grounded.0 = true;

                let x = resting_height.0 - result.1;
                let x_dot = velocity.0.y;

                let spring_force = spring_damper(1.0, 15.0, x, x_dot, SERVER_TICK_RATE);
                velocity.0.y += spring_force;

                continue;
            }

            grounded.0 = false;
        }
    }
}

pub fn sync_positions_system(
    mut query: Query<(&mut Transform, &Position)>
) {
    for (mut transform, position) in query.iter_mut() {
        transform.translation = position.0;
    }
}
