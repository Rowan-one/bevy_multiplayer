use bevy::prelude::*;

pub struct ClientPlayerPlugin;
impl Plugin for ClientPlayerPlugin {
    fn build(&self, app: &mut App) {
        
    }
}

#[derive(Debug, Default, Message)]
pub struct ClientTickMessage;

pub fn client_tick_system(

) {

}

pub fn local_player_movement_system(

) {

}

pub fn server_reconciliation_system(

) {

}
