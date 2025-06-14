use std::ops::Deref;

use sdl2::pixels::Color;

#[derive(Default, Debug)]
pub struct Grid {
    pub cells: Vec<Cell>,
    pub w: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Tile {
    /// index into a tileset
    pub idx: usize,
    pub color: Color,
}

impl Default for Tile {
    fn default() -> Self {
        Self::new(0)
    }
}

impl Tile {
    pub fn new(idx: usize) -> Self {
        Self {
            idx,
            color: Color::WHITE,
        }
    }

    pub fn new_with_color(idx: usize, color: Color) -> Self {
        Self { idx, color }
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Cell {
    /// Bottom -> top stack of tiles
    pub tiles: Vec<Tile>,
}

impl Deref for Cell {
    type Target = Vec<Tile>;

    fn deref(&self) -> &Self::Target {
        &self.tiles
    }
}

impl Cell {
    pub fn new(idx: usize) -> Self {
        Self {
            tiles: vec![Tile::new(idx)],
        }
    }

    pub fn new_with_color(idx: usize, color: Color) -> Self {
        Self {
            tiles: vec![Tile::new_with_color(idx, color)],
        }
    }

    pub fn push(&mut self, t: Tile) {
        self.tiles.push(t);
    }
}
