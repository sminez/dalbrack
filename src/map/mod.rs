use crate::{Pos, map::map_tile::MapTile, state::State};
use sdl2::rect::Rect;
use std::{
    cmp::{max, min},
    collections::HashSet,
};

pub mod builders;
pub mod fov;
pub mod map_tile;

const WALL: usize = 0;
const FLOOR: usize = 1;
// const DOOR: usize = 2;

#[derive(Debug, Clone)]
pub struct Map {
    pub tiles: Vec<usize>,
    pub explored: HashSet<usize>,
    pub tile_defs: Vec<MapTile>,
    pub w: usize,
    pub h: usize,
}

impl Map {
    pub fn new(w: usize, h: usize, state: &State<'_>) -> Self {
        Self {
            tiles: vec![WALL; w * h],
            explored: HashSet::new(),
            tile_defs: vec![
                MapTile::wall(&state.ts, &state.palette),
                MapTile::floor(&state.ts, &state.palette),
                // MapTile::door(&state.ts, &state.palette),
            ],
            w,
            h,
        }
    }

    pub fn explore_all(&mut self) {
        self.explored = (0..self.tiles.len()).collect();
    }

    pub fn clear_explored(&mut self) {
        self.explored = Default::default();
    }

    #[inline]
    pub fn idx(&self, x: usize, y: usize) -> usize {
        y * self.w + x
    }

    #[inline]
    pub fn pos_idx(&self, Pos { x, y }: Pos) -> usize {
        y as usize * self.w + x as usize
    }

    pub fn tile_at(&self, pos: Pos) -> &MapTile {
        let idx = self.idx(pos.x as usize, pos.y as usize);
        let tile_idx = self.tiles[idx];

        &self.tile_defs[tile_idx]
    }

    pub fn carve_rect(&mut self, r: Rect) {
        for y in r.y..r.y + r.h {
            for x in r.x..r.x + r.w {
                let idx = self.idx(x as usize, y as usize);
                self.tiles[idx] = FLOOR;
            }
        }
    }

    pub fn carve_h_tunnel(&mut self, x1: i32, x2: i32, y: i32) {
        for x in min(x1, x2)..=max(x1, x2) {
            let idx = self.idx(x as usize, y as usize);
            self.tiles[idx] = FLOOR;
        }
    }

    pub fn carve_v_tunnel(&mut self, y1: i32, y2: i32, x: i32) {
        for y in min(y1, y2)..=max(y1, y2) {
            let idx = self.idx(x as usize, y as usize);
            self.tiles[idx] = FLOOR;
        }
    }
}
