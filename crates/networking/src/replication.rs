use bevy::prelude::*;
use serde::{Serialize, Deserialize};

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

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct NetworkedEntities {
    pub entities: Vec<Entity>,
    pub translations: Vec<[f32; 3]>
}