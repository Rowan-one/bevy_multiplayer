use bevy::prelude::*;
use bevy_rapier3d::prelude::*;
use shared::{components::*, consts::SERVER_TICK_RATE};

pub struct ServerPhysicsPlugin;
impl Plugin for ServerPhysicsPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(RapierPhysicsPlugin::<NoUserData>::default());
        app.add_plugins(RapierDebugRenderPlugin::default());
        app.insert_resource(TimestepMode::Fixed { dt: SERVER_TICK_RATE, substeps: 5 });
        
        app.add_systems(Update, (
            sync_transforms_system.before(networking::replication::send_snapshots_system),
        ));
    }
}

pub fn sync_transforms_system(
    mut query: Query<(&CustomPosition, &mut Transform)>,
) {
    for (pos, mut transform) in query.iter_mut() {
        transform.translation = pos.0;
    }
}
