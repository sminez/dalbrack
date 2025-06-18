use crate::{Grid, Pos, map::map_tile::MapTile, state::State};
use sdl2::rect::Rect;
use std::{
    cmp::{max, min},
    collections::HashSet,
    ops::{Deref, DerefMut},
};

pub mod builders;
pub mod fov;
pub mod map_tile;

const WALL: usize = 0;
const FLOOR: usize = 1;
// const DOOR: usize = 2;

#[derive(Debug, Clone)]
pub struct Map {
    pub tiles: Grid<usize>,
    pub explored: HashSet<usize>,
    pub tile_defs: Vec<MapTile>,
}

impl Map {
    pub fn new(w: usize, h: usize, state: &State<'_>) -> Self {
        Self {
            tiles: Grid::new(w, h, WALL),
            explored: HashSet::new(),
            tile_defs: vec![
                MapTile::wall(&state.ts, &state.palette),
                MapTile::floor(&state.ts, &state.palette),
                // MapTile::door(&state.ts, &state.palette),
            ],
        }
    }

    pub fn explore_all(&mut self) {
        self.explored = (0..self.tiles.len()).collect();
    }

    pub fn clear_explored(&mut self) {
        self.explored = Default::default();
    }

    pub fn tile_at(&self, pos: Pos) -> &MapTile {
        &self.tile_defs[self.tiles[pos]]
    }

    pub fn carve_rect(&mut self, r: Rect) {
        for y in r.y..r.y + r.h {
            for x in r.x..r.x + r.w {
                self.tiles[Pos::new(x, y)] = FLOOR;
            }
        }
    }

    pub fn carve_h_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2)..=max(x1, x2) {
            self.tiles[Pos::new(x, y)] = FLOOR;
        }
    }

    pub fn carve_v_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            self.tiles[Pos::new(x, y)] = FLOOR;
        }
    }
}

impl Deref for Map {
    type Target = Grid<usize>;

    fn deref(&self) -> &Self::Target {
        &self.tiles
    }
}

impl DerefMut for Map {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.tiles
    }
}
