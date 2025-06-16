use crate::tileset::{Tile, TileSet};
use sdl2::pixels::Color;
use std::collections::HashMap;

#[derive(Debug, Copy, Clone)]
pub struct MapTile {
    pub t: Tile,
    pub path_cost: Option<u8>,
    pub block_move: bool,
    pub block_sight: bool,
}

impl MapTile {
    pub fn new(
        ident: &str,
        color: &str,
        path_cost: Option<u8>,
        block_move: bool,
        block_sight: bool,
        ts: &TileSet<'_>,
        palette: &HashMap<String, Color>,
    ) -> Self {
        let idx = ts.tile_index(ident).unwrap();
        let color = *palette.get(color).unwrap();

        Self {
            t: Tile::new_with_color(idx, color),
            path_cost,
            block_move,
            block_sight,
        }
    }

    pub fn wall(ts: &TileSet<'_>, palette: &HashMap<String, Color>) -> Self {
        Self::new("shade-dark", "grey14", None, true, true, ts, palette)
    }

    pub fn floor(ts: &TileSet<'_>, palette: &HashMap<String, Color>) -> Self {
        Self::new("dot", "grey12", Some(1), false, false, ts, palette)
    }

    //     pub fn door(ts: &TileSet<'_>, palette: &HashMap<String, Color>) -> Self {
    //         Self::new("=", "faded_orange", Some(2), true, true, ts, palette)
    //     }
}
