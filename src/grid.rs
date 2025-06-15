use crate::Pos;
use hecs::World;
use sdl2::pixels::Color;

#[derive(Default, Debug)]
pub struct Grid {
    pub tiles: Vec<Tile>,
    pub w: usize,
}

impl Grid {
    pub fn spawn_all_at(&self, dx: i32, dy: i32, world: &mut World) {
        for (col, line) in self.tiles.chunks(self.w).enumerate() {
            for (row, tile) in line.iter().enumerate() {
                world.spawn((Pos::new(dx + row as i32, dy + col as i32), *tile));
            }
        }
    }
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
