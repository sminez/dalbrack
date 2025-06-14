use crate::{Pos, grid::Tile, ui::Sdl2UI};
use rand::{Rng, rng};
use std::cmp::{max, min};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mob {
    pub pos: Pos,
    pub tile: Tile,
}

impl Mob {
    pub fn new(ident: &str, color: &str, x: i32, y: i32, ui: &Sdl2UI) -> Self {
        let mut tile = ui.ts.tile(ident).unwrap();
        tile.color = *ui.palette.get(color).unwrap();

        Self {
            pos: Pos::new(x, y),
            tile,
        }
    }

    pub fn blit(&self, ui: &mut Sdl2UI) -> anyhow::Result<()> {
        ui.blit_tile(&self.tile, self.pos.x as u32, self.pos.y as u32)
    }

    pub fn random_move(&mut self, xmax: i32, ymax: i32) {
        let mut r = rng();
        if !r.random_bool(0.7) {
            return;
        }

        // 012
        // 3 4
        // 567
        let dir = r.random_range(0..8);

        if [0, 3, 5].contains(&dir) {
            self.pos.x = max(0, self.pos.x - 1);
        } else if [2, 4, 7].contains(&dir) {
            self.pos.x = min(xmax, self.pos.x + 1);
        }

        if [0, 1, 2].contains(&dir) {
            self.pos.y = max(0, self.pos.y - 1);
        } else if [5, 6, 7].contains(&dir) {
            self.pos.y = min(ymax, self.pos.y + 1);
        }
    }
}
