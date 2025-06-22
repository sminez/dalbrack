use crate::{Pos, map::fov::Opacity, state::State, tileset::Tile};
use hecs::{Bundle, Entity};
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

        *pos = pos.random_offset();
        pos.x = max(0, min(xmax, pos.x));
        pos.y = max(0, min(ymax, pos.y));
    }
}

#[derive(Debug, Bundle, Clone, Copy)]
pub struct Mob {
    pub pos: Pos,
    pub tile: Tile,
    pub opacity: Opacity,
    pub ai: RandomMoveAI,
}

impl Mob {
    pub fn new(ident: &str, color: &str, x: i32, y: i32, state: &State<'_>) -> Self {
        let mut tile = state.ts.tile(ident).unwrap();
        tile.color = *state.palette.get(color).unwrap();

        Self {
            pos: Pos::new(x, y),
            tile,
            opacity: Opacity(0.9),
            ai: RandomMoveAI,
        }
    }

    pub fn spawn(ident: &str, color: &str, x: i32, y: i32, state: &mut State<'_>) -> Entity {
        let mob = Self::new(ident, color, x, y, state);
        state.world.spawn(mob)
    }
}
