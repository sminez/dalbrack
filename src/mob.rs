use crate::{Pos, grid::Tile, state::State};
use hecs::Bundle;
use rand::{Rng, rng};
use std::cmp::{max, min};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RandomMoveAI;

impl RandomMoveAI {
    pub fn random_move(&self, pos: &mut Pos, xmax: i32, ymax: i32) {
        let mut r = rng();
        if !r.random_bool(0.7) {
            return;
        }

        // 012
        // 3 4
        // 567
        let dir = r.random_range(0..8);

        if [0, 3, 5].contains(&dir) {
            pos.x = max(0, pos.x - 1);
        } else if [2, 4, 7].contains(&dir) {
            pos.x = min(xmax, pos.x + 1);
        }

        if [0, 1, 2].contains(&dir) {
            pos.y = max(0, pos.y - 1);
        } else if [5, 6, 7].contains(&dir) {
            pos.y = min(ymax, pos.y + 1);
        }
    }
}

#[derive(Debug, Bundle, Clone, Copy, PartialEq, Eq)]
pub struct Mob {
    pub pos: Pos,
    pub tile: Tile,
    pub ai: RandomMoveAI,
}

impl Mob {
    pub fn new(ident: &str, color: &str, x: i32, y: i32, state: &State<'_>) -> Self {
        let mut tile = state.ts.tile(ident).unwrap();
        tile.color = *state.palette.get(color).unwrap();

        Self {
            pos: Pos::new(x, y),
            tile,
            ai: RandomMoveAI,
        }
    }
}
