//! Systems relating to the player controlled character
use crate::{Pos, map::Map, state::State};
use sdl2::pixels::Color;

pub struct Player;

impl Player {
    pub fn set_pos(p_new: Pos, state: &mut State<'_>) {
        for (_entity, (_player, pos)) in state.world.query_mut::<(&Player, &mut Pos)>() {
            *pos = p_new;
        }
    }

    pub fn try_move(dx: i32, dy: i32, state: &mut State<'_>) {
        let map = state.world.get::<&Map>(state.e_map).unwrap();

        for (_entity, (_player, pos)) in state.world.query::<(&Player, &mut Pos)>().iter() {
            let p_new = Pos::new(pos.x + dx, pos.y + dy);
            if map.tile_at(p_new).block_move {
                return;
            }
            *pos = p_new;
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct FovRange {
    pub range: u32,
    pub color: Color,
}
