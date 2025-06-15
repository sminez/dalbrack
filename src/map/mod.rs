use crate::{
    state::State,
    tileset::{Tile, TileSet},
};
use sdl2::{pixels::Color, rect::Rect};
use std::collections::HashMap;

const WALL: usize = 0;
const FLOOR: usize = 1;
// const DOOR: usize = 2;

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
        Self::new("shade-mid", "grey14", None, true, true, ts, palette)
    }

    pub fn floor(ts: &TileSet<'_>, palette: &HashMap<String, Color>) -> Self {
        Self::new(" ", "grey12", Some(1), false, false, ts, palette)
    }

    //     pub fn door(ts: &TileSet<'_>, palette: &HashMap<String, Color>) -> Self {
    //         Self::new("=", "faded_orange", Some(2), true, true, ts, palette)
    //     }
}

#[derive(Debug, Clone)]
pub struct Map {
    pub tiles: Vec<usize>,
    pub explored: Vec<usize>,
    pub tile_defs: Vec<MapTile>,
    pub w: usize,
}

impl Map {
    pub fn new(w: usize, h: usize, state: &State<'_>) -> Self {
        Self {
            tiles: vec![WALL; w * h],
            explored: (0..w * h).collect(),
            tile_defs: vec![
                MapTile::wall(&state.ts, &state.palette),
                MapTile::floor(&state.ts, &state.palette),
                // MapTile::door(&state.ts, &state.palette),
            ],
            w,
        }
    }

    pub fn carve_room(&mut self, r: Rect) {
        for y in r.y..r.y + r.h {
            for x in r.x..r.x + r.w {
                self.tiles[y as usize * self.w + x as usize] = FLOOR;
            }
        }
    }
}
