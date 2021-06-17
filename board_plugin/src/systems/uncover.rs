use crate::components::{Coordinates, Uncover, Bomb, BombNeighbor};
use crate::events::TileTriggerEvent;
use crate::{Board};
use bevy::log;
use bevy::prelude::*;

pub fn uncover_tiles(
    mut commands: Commands,
    mut board: ResMut<Board>,
    children: Query<(Entity, &Parent), With<Uncover>>,
    parents: Query<(&Coordinates, Option<&Bomb>, Option<&BombNeighbor>)>,
) {
    // We iterate through tile covers to uncover
    for (entity, parent) in children.iter() {
        // We retrieve entity commands
        commands
            .entity(entity)
            // We remove the component
            .remove::<Uncover>()
            // we destroy the entity
            .despawn_recursive();
        let (coords, bomb, bomb_counter) = match parents.get(parent.0) {
            Ok(v) => v,
            Err(e) => {
                log::error!("{}", e);
                continue;
            }
        };
        // We remove the entity from the board map
        match board.try_uncover_tile(coords) {
            None => log::warn!("Tried to uncover an already uncovered tile"),
            Some(e) => log::debug!("Uncovered tile {} (entity: {:?})", coords, e),
        }
        if bomb.is_some() {
            log::info!("Boom !");
            // TODO: Add explosion event
        }
        // If the tile is empty..
        else if bomb_counter.is_none() {
            // .. We propagate the uncovering by adding the `Uncover` component to adjacent tiles
            // which will then be removed next frame
            for entity in board.adjacent_covered_tiles(coords) {
                commands.entity(entity).insert(Uncover {});
            }
        }
    }
}

pub fn trigger_event_handler(
    mut commands: Commands,
    board: Res<Board>,
    mut tile_trigger_evr: EventReader<TileTriggerEvent>,
) {
    for trigger_event in tile_trigger_evr.iter() {
        if let Some(entity) = board.covered_tile(&trigger_event.0) {
            commands.entity(*entity).insert(Uncover {});
        }
    }
}