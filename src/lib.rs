pub use bevy::prelude::*;
pub use bevy_renet::renet::*;
pub use bevy_renet::*;

pub mod server;
pub mod client;
pub mod shared;

pub const PROTOCOL_ID: u64 = 1000;