use std::collections::HashMap;

use bevy::prelude::*;
use bevy_renet::renet::{RenetClient, RenetServer};
use bevy_replicon::prelude::*;
use serde::{Serialize, Deserialize};

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
) {
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

pub fn receive_snapshots_system(
    mut commands: Commands,
    mut client: ResMut<RenetClient>,
    id_map: Res<NetIdMap>,
) {
    while let Some(message) = client.receive_message(ServerChannel::Replication) {
        // decode message
        let config = bincode::config::standard();
        let (snapshot, _): (Snapshot, usize) = bincode::serde::decode_from_slice(&message, config).unwrap();

        // apply snapshot transforms to client entities
        for entity_snap in snapshot.entities.iter() {
            if let Some(entity) = id_map.0.get(&entity_snap.id.0) {
                commands.entity(*entity).insert(Transform::from_translation(entity_snap.position));
            }
        }
    }
}