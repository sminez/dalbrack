use crate::tileset::{Tile, TileSet};
use sdl2::pixels::Color;
use std::collections::HashMap;

pub const WALL: usize = 0;
pub const FLOOR: usize = 1;
// pub const DOOR: usize = 2;

#[derive(Debug, Copy, Clone)]
pub struct MapTile {
    /// The sprite to use for this tile
    pub t: Tile,
    /// Additional weight for pathfinding relative to floor tiles.
    /// None == blocked
    pub path_cost: Option<i32>,
    /// Additional cost to move through the cell
    pub move_weight: u8,
    /// Opacity 0.0..=1.0
    pub opacity: f32,
}

impl MapTile {
    pub fn new(
        ident: &str,
        color: &str,
        path_cost: Option<i32>,
        move_weight: u8,
        opacity: f32,
        ts: &TileSet<'_>,
        palette: &HashMap<String, Color>,
    ) -> Self {
        let idx = ts.tile_index(ident).unwrap();
        let color = *palette.get(color).unwrap();

        Self {
            t: Tile::new_with_color(idx, color),
            path_cost,
            move_weight,
            opacity,
        }
    }

    pub fn forest_tiles(ts: &TileSet<'_>, palette: &HashMap<String, Color>) -> Vec<Self> {
        vec![
            Self::new("club", "tree", None, u8::MAX, 0.7, ts, palette),
            Self::new("dot", "earth", Some(1), 1, 0.0, ts, palette),
        ]
    }

    pub fn dungeon_tiles(ts: &TileSet<'_>, palette: &HashMap<String, Color>) -> Vec<Self> {
        vec![
            Self::new("shade-dark", "grey13", None, u8::MAX, 1.0, ts, palette),
            Self::new("dot", "grey15", Some(1), 1, 0.0, ts, palette),
        ]
    }

    pub fn wall(ts: &TileSet<'_>, palette: &HashMap<String, Color>) -> Self {
        Self::new("shade-dark", "grey13", None, u8::MAX, 1.0, ts, palette)
    }

    pub fn floor(ts: &TileSet<'_>, palette: &HashMap<String, Color>) -> Self {
        Self::new("dot", "earth", Some(1), 1, 0.0, ts, palette)
    }

    //     pub fn door(ts: &TileSet<'_>, palette: &HashMap<String, Color>) -> Self {
    //         Self::new("=", "faded_orange", Some(2), 2, 0.1, ts, palette)
    //     }

    pub fn blocks_movement(&self) -> bool {
        self.path_cost.is_none()
    }
}
