use bevy::prelude::*;
pub use {bomb::*, bomb_neighbor::*, coordinates::*, uncover::*};

mod bomb;
mod bomb_neighbor;
mod coordinates;
mod uncover;

#[derive(Component)]
pub struct BoardMarker;
