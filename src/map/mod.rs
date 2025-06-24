use crate::{Grid, Pos, grid::a_star, map::map_tile::MapTile, state::State};
use fov::LightMap;
use sdl2::rect::Rect;
use std::{
    cmp::{max, min},
    collections::HashSet,
    ops::{Deref, DerefMut},
};

pub mod builders;
pub mod fov;
pub mod map_tile;
mod mapset;

pub use mapset::MapSet;

const WALL: usize = 0;
const FLOOR: usize = 1;
// const DOOR: usize = 2;

#[derive(Debug, Clone)]
pub struct Map {
    pub tiles: Grid<usize>,
    pub explored: HashSet<usize>,
    pub tile_defs: Vec<MapTile>,
    pub light_map: Option<LightMap>,
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
            light_map: None,
        }
    }

    pub fn explore_all(&mut self) {
        self.explored = (0..self.tiles.len()).collect();
    }

    pub fn clear_explored(&mut self) {
        self.explored = Default::default();
    }

    pub fn tile_at(&self, pos: Pos) -> &MapTile {
        if self.contains_pos(pos) {
            &self.tile_defs[self.tiles[pos]]
        } else {
            &self.tile_defs[WALL]
        }
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

    pub fn a_star_in_player_explored(&self, a: Pos, b: Pos) -> Vec<Pos> {
        a_star(a, b, &self.tiles, |p| {
            if self.explored.contains(&self.pos_idx(p)) {
                self.tile_defs[self.tiles[p]].path_cost
            } else {
                None
            }
        })
    }

    pub fn a_star(&self, a: Pos, b: Pos) -> Vec<Pos> {
        a_star(a, b, &self.tiles, |p| {
            self.tile_defs[self.tiles[p]].path_cost
        })
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
