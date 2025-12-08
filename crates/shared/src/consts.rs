//
pub const PLAYER_MOVE_SPEED: f32 = 5.0;
pub const PLAYER_JUMP_POWER: f32 = 5.0;
pub const PLAYER_ACCEL: f32 = 10.;
pub const PLAYER_DEACCEL: f32 = 1.;

pub const MAX_FRAME_TIME: f32 = 0.25;
pub const SERVER_TICK_RATE: f32 = 1./60.;
pub const CLIENT_TICK_RATE: f32 = 1./60.;
pub const TICKS_PER_SNAPSHOT: u8 = 5; // send a snapshot to client every n ticks
pub const GRAVITY: f32 = -9.81;
