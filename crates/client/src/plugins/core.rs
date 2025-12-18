use bevy::prelude::*;
use shared::{consts::*, messages::ClientTickMessage, resources::*};

pub struct ClientCorePlugin;
impl Plugin for ClientCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Settings>();
        app.init_resource::<ClientTick>();
        app.init_resource::<PrevFrameTime>();
        app.init_resource::<TickAccumulator>();

        app.add_systems(PreUpdate, client_tick_system);
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
