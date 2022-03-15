use crate::components::Coordinates;
use crate::tile_map::TileMap;
use crate::Bounds2;
use bevy::log;
use bevy::prelude::*;
use rand::random;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Board {
    pub tile_map: TileMap,
    pub bounds: Bounds2,
    pub tile_size: f32,
    pub covered_tiles: HashMap<Coordinates, Entity>,
    pub marked_tiles: Vec<Coordinates>,
    pub entity: Entity,
}

impl Board {
    /// Translates a mouse position to board coordinates
    pub fn mouse_position(&self, window: &Window, position: Vec2) -> Option<Coordinates> {
        let window_size = Vec2::new(window.width(), window.height());
        let position = position - window_size / 2.;

        if !self.bounds.in_bounds(position) {
            return None;
        }
        let coordinates = position - self.bounds.position;
        Some(Coordinates {
            x: (coordinates.x / self.tile_size) as u16,
            y: (coordinates.y / self.tile_size) as u16,
        })
    }

    /// Retrieves a covered tile entity
    pub fn tile_to_uncover(&self, coords: &Coordinates) -> Vec<Entity> {
        if self.marked_tiles.contains(coords) {
            bevy::log::info!("tile is marked");
            vec![]
        } else if self.covered_tiles.contains_key(coords) {
            bevy::log::info!("Single uncover");
            self.covered_tiles
                .get(coords)
                .map(|entity| vec![*entity])
                .unwrap()
        } else if self.marked_tile_is_safe(coords) {
            bevy::log::info!("Marked tile is safe");
            self.surrounding_covered_tiles(coords)
        } else {
            bevy::log::info!("Uncovered and unsafe for surrounding");
            vec![]
        }
    }

    pub fn surrounding_covered_tiles(&self, coords: &Coordinates) -> Vec<Entity> {
        [
            (-1i8, -1i8),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ]
        .iter()
        .map(|tuple| *coords + *tuple)
        .filter(|coords| self.covered_tiles.contains_key(coords))
        .filter(|coords| !self.marked_tiles.contains(coords))
        .map(|coords| *self.covered_tiles.get(&coords).unwrap())
        .collect()
    }

    pub fn marked_safe_count_at(&self, coords: &Coordinates) -> u8 {
        [
            (-1i8, -1i8),
            (0, -1),
            (1, -1),
            (-1, 0),
            (1, 0),
            (-1, 1),
            (0, 1),
            (1, 1),
        ]
        .iter()
        .map(|tuple| *coords + *tuple)
        .filter(|coords| self.marked_tiles.contains(coords))
        .count() as u8
    }

    pub fn marked_tile_is_safe(&self, coords: &Coordinates) -> bool {
        let bomb_count = self.tile_map.bomb_count_at(*coords);
        let marked_neighbors = self.marked_safe_count_at(coords);
        bevy::log::info!("Bomb count is {bomb_count} and marked bombs are {marked_neighbors}");
        marked_neighbors > 0 && bomb_count == marked_neighbors
    }

    /// Removes the `coords` from `marked_tiles`
    fn unmark_tile(&mut self, coords: &Coordinates) -> Option<Coordinates> {
        let pos = match self.marked_tiles.iter().position(|a| a == coords) {
            None => {
                log::error!("Failed to unmark tile at {}", coords);
                return None;
            }
            Some(p) => p,
        };
        Some(self.marked_tiles.remove(pos))
    }

    pub fn find_safe_covered_coord(&self) -> Option<Coordinates> {
        for _ in 0..10000 {
            let covered_tiles: Vec<Coordinates> = self.covered_tiles.keys().cloned().collect();
            let covered_tile = covered_tiles[random::<usize>() % covered_tiles.len()];
            if !self.tile_map.is_bomb_at(covered_tile) {
                return Some(covered_tile);
            }
        }
        None
    }

    /// We try to uncover a tile, returning the entity
    pub fn try_uncover_tile(&mut self, coords: &Coordinates) -> Option<Entity> {
        if self.marked_tiles.contains(coords) {
            self.unmark_tile(coords)?;
        }
        self.covered_tiles.remove(coords)
    }

    /// We try to mark or unmark a tile, returning the entity and if the tile is marked
    pub fn try_toggle_mark(&mut self, coords: &Coordinates) -> Option<(Entity, bool)> {
        let entity = *self.covered_tiles.get(coords)?;
        let mark = if self.marked_tiles.contains(coords) {
            self.unmark_tile(coords)?;
            false
        } else {
            self.marked_tiles.push(*coords);
            true
        };
        Some((entity, mark))
    }

    /// We retrieve the adjacent covered tile entities of `coord`
    pub fn adjacent_covered_tiles(&self, coord: Coordinates) -> Vec<Entity> {
        self.tile_map
            .safe_square_at(coord)
            .filter_map(|c| self.covered_tiles.get(&c))
            .copied()
            .collect()
    }

    /// Is the board complete
    #[inline]
    #[must_use]
    pub fn is_completed(&self) -> bool {
        self.tile_map.bomb_count() as usize == self.covered_tiles.len()
    }
}
