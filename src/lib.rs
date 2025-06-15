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
}
