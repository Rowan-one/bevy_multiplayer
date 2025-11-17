use std::collections::{HashMap, VecDeque};

use bevy::prelude::*;
use bevy_renet::renet::{RenetClient, RenetServer};
use bevy_replicon::prelude::*;
use serde::{Serialize, Deserialize};

use shared::messages::*;
use crate::protocol::ServerChannel;

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
pub struct SnapshotBuffer(pub HashMap<u64, VecDeque<(f32, Vec3)>>);
    
#[derive(Debug, Serialize, Deserialize)]
pub struct EntitySnap {
    pub id: NetId,
    pub position: Vec3
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Snapshot {
    pub tick: u64,
    pub entities: Vec<EntitySnap>,
}

pub fn send_snapshots_system(
    query: Query<(&Transform, &NetId), With<Replicated>>,
    mut server: ResMut<RenetServer>,
    mut server_tick_message: MessageReader<ServerTickMessage>,
) {
    for _message in server_tick_message.read() {
        // create new snapshot
        let mut snapshot = Snapshot::default();

        // populate snapshot
        for (&transform, &id) in query.iter() {
            let snap = EntitySnap { id, position: transform.translation };
            snapshot.entities.push(snap);
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
                .entry(entity_snap.id.0)
                .or_default();

            // append new snap to back of queue
            buf.push_back((
                elapsed_time,
                entity_snap.position,
            ));

            // trim length of buffer
            while buf.len() > 16 {
                buf.pop_front();
            }
        }
    }
}
