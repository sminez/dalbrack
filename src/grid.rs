use sdl2::pixels::Color;

#[derive(Default, Debug)]
pub struct Grid {
    pub tiles: Vec<Tile>,
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
