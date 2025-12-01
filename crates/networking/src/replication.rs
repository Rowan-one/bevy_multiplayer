use std::collections::{HashMap, VecDeque};

use bevy::prelude::*;
use bevy_renet::renet::{RenetClient, RenetServer};
use bevy_replicon::prelude::*;
use avian3d::prelude::*;
use serde::{Serialize, Deserialize};

use shared::{consts::TICKS_PER_SNAPSHOT, messages::*, resources::InputStateMap, structs::EntitySnap};
use crate::{protocol::ServerChannel, server::OwnedByClient};

#[derive(Component, Debug, Clone, Copy, Hash, Serialize, Deserialize)]
pub struct NetId(pub u64);

#[derive(Debug, Default, Resource)]
pub struct NetIdGen(pub u64);
impl NetIdGen {
    pub fn next(&mut self) -> NetId {
        let id = self.0;
        self.0 += 1;
        NetId(id)
    }
}

#[derive(Debug, Default, Resource)]
pub struct NetIdMap(pub HashMap<u64, Entity>);

#[derive(Debug, Default, Resource)]
pub struct SnapshotBuffer(pub HashMap<u64, VecDeque<EntitySnap>>);

#[derive(Debug, Default, Message)]
pub struct SnapshotReceiveMessage();

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Snapshot {
    pub tick: u64,
    pub entities: Vec<EntitySnap>,
}

pub fn send_snapshots_system(
    time: Res<Time>,
    query: Query<(&Transform, &LinearVelocity, &NetId, &OwnedByClient), With<Replicated>>,
    input_state_map: Res<InputStateMap>,
    mut server: ResMut<RenetServer>,
    mut server_tick_message: MessageReader<ServerTickMessage>,
) {
    for tick in server_tick_message.read() {
        // only send snapshots every nth tick
        if tick.tick % TICKS_PER_SNAPSHOT as u64 != 0 { continue; }

        // create new snapshot
        let mut snapshot = Snapshot::default();

        // populate snapshot
        for (&transform, velocity, &id, &client) in query.iter() {

            // get client's input state
            if let Some(input_state) = input_state_map.0.get(&client.id) {

                let snap = EntitySnap {
                    net_id: id.0,
                    position: transform.translation,
                    velocity: velocity.0,
                    last_processed_seq: input_state.last_processed_input.seq,
                    timestamp: time.elapsed_secs(),
                };

                snapshot.entities.push(snap);
            }
        }

        let config = bincode::config::standard();
        let message = bincode::serde::encode_to_vec(snapshot, config).unwrap();

        // broadcast snapshot to all clients
        server.broadcast_message(ServerChannel::Replication, message);
    }
}

pub fn receive_snapshots_system(
    time: Res<Time>,
    mut client: ResMut<RenetClient>,
    mut snapshot_buffer: ResMut<SnapshotBuffer>,
    mut snapshot_receive_message: MessageWriter<SnapshotReceiveMessage>,
) {
    while let Some(message) = client.receive_message(ServerChannel::Replication) {
        // decode message
        let config = bincode::config::standard();
        let (snapshot, _): (Snapshot, usize) = bincode::serde::decode_from_slice(&message, config).unwrap();

        // add all entity snapshots to buffer resource
        for entity_snap in snapshot.entities.iter() {
            let elapsed_time = time.elapsed_secs();

            // get existing entity buffer or create new one
            let buf = snapshot_buffer.0
                .entry(entity_snap.net_id)
                .or_default();

            // append new snap to back of queue
            buf.push_back(EntitySnap {
                net_id: entity_snap.net_id,
                timestamp: elapsed_time, // NOTE: used elapsed time from this function before using
                                         // EntitySnap, might want to use entity_snap.timestamp
                position: entity_snap.position,
                velocity: entity_snap.velocity,
                last_processed_seq: entity_snap.last_processed_seq,
            });

            // trim length of buffer
            while buf.len() > 16 {
                buf.pop_front();
            }

            // send snapshot receive message
            snapshot_receive_message.write(SnapshotReceiveMessage::default());
        }
    }
}
