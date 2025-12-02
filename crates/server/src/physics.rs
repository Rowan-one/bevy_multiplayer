use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use shared::{components::{CustomVelocity, Grounded, RestingHeight}, consts::SERVER_TICK_RATE, functions::spring_damper, messages::ServerTickMessage};

pub struct ServerPhysicsPlugin;
impl Plugin for ServerPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default());
        app.add_plugins(RapierDebugRenderPlugin::default());
        app.insert_resource(TimestepMode::Fixed { dt: SERVER_TICK_RATE, substeps: 5 });
        
        app.add_systems(Update, (
            ground_check_system
                .after(crate::player::process_inputs_system)
                .before(networking::replication::send_snapshots_system),
            // sync_positions_system
            //     .after(process_inputs_system)
            //     .after(ground_check_system)
        ));
    }
}

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
    mut query: Query<(&Transform, &mut CustomVelocity, &mut Grounded, &RestingHeight)>,
    mut server_tick_message: MessageReader<ServerTickMessage>,
) {
    for _tick in server_tick_message.read() {
        let rapier_context = rapier_context.single().unwrap();

        for (position, mut velocity, mut grounded, resting_height) in query.iter_mut() {
            if let Some(result) = rapier_context.cast_ray(position.translation, Vec3::NEG_Y, resting_height.0, true, QueryFilter::exclude_kinematic()) {
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

// pub fn sync_positions_system(
//     mut query: Query<(&mut Transform, &Position)>
// ) {
//     for (mut transform, position) in query.iter_mut() {
//         transform.translation = position.0;
//     }
// }
