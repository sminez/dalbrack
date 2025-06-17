pub mod data_files;
pub mod map;
pub mod mob;
pub mod player;
pub mod state;
pub mod tileset;
pub mod ui;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Pos {
    pub x: i32,
    pub y: i32,
}

impl Pos {
    pub const fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn dist(&self, other: Pos) -> u32 {
        (((self.x - other.x).pow(2) + (self.y - other.y).pow(2)) as f32)
            .sqrt()
            .ceil() as u32
    }
}
