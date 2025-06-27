use crate::{
    tileset::{Tile, TileSet},
    ui::palette,
};
use sdl2::pixels::Color;

pub const WALL: usize = 0;
pub const FLOOR: usize = 1;
// pub const DOOR: usize = 2;

#[derive(Debug, Copy, Clone)]
pub struct MapTile {
    /// The sprite to use for this tile
    pub t: Tile,
    pub bg: Option<Color>,
    /// Additional weight for pathfinding relative to floor tiles.
    /// None == blocked
    pub path_cost: Option<i32>,
    /// Additional cost to move through the cell
    pub move_weight: u8,
    /// Opacity 0.0..=1.0
    pub opacity: f32,
}

impl MapTile {
    pub fn new(
        ident: &str,
        color: Color,
        path_cost: Option<i32>,
        move_weight: u8,
        opacity: f32,
        ts: &TileSet<'_>,
    ) -> Self {
        let idx = ts.tile_index(ident).unwrap();

        Self {
            t: Tile::new_with_color(idx, color),
            bg: None,
            path_cost,
            move_weight,
            opacity,
        }
    }

    pub fn with_bg(mut self, color: Color) -> Self {
        self.bg = Some(color);
        self
    }

    pub fn forest_tiles(ts: &TileSet<'_>) -> Vec<Self> {
        vec![
            // default "wall"
            Self::new("club", palette::TREE_1, None, u8::MAX, 0.6, ts),
            // floor
            Self::new("dot", palette::EARTH, Some(1), 1, 0.0, ts),
            // other trees
            Self::new("club", palette::TREE_2, None, u8::MAX, 0.6, ts),
            Self::new("spade", palette::TREE_1, None, u8::MAX, 0.7, ts),
            Self::new("spade", palette::TREE_2, None, u8::MAX, 0.7, ts),
        ]
        .into_iter()
        .map(|c| c.with_bg(palette::FOREST_BG))
        .collect()
    }

    pub fn dungeon_tiles(ts: &TileSet<'_>) -> Vec<Self> {
        vec![
            Self::new("shade-dark", palette::GREY_13, None, u8::MAX, 1.0, ts),
            Self::new("dot", palette::GREY_15, Some(1), 1, 0.0, ts),
        ]
    }

    pub fn wall(ts: &TileSet<'_>) -> Self {
        Self::new("shade-dark", palette::GREY_13, None, u8::MAX, 1.0, ts)
    }

    pub fn floor(ts: &TileSet<'_>) -> Self {
        Self::new("dot", palette::EARTH, Some(1), 1, 0.0, ts)
    }

    //     pub fn door(ts: &TileSet<'_>, palette: &HashMap<String, Color>) -> Self {
    //         Self::new("=", "faded_orange", Some(2), 2, 0.1, ts, palette)
    //     }

    pub fn blocks_movement(&self) -> bool {
        self.path_cost.is_none()
    }
}
