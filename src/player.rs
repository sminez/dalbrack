//! Systems relating to the player controlled character
use crate::{Pos, map::Map, state::State};

pub struct Player;

impl Player {
    pub fn set_pos(p_new: Pos, state: &mut State<'_>) {
        *state
            .world
            .query_one_mut::<&mut Pos>(state.e_player)
            .unwrap() = p_new;
    }

    pub fn try_move(dx: i32, dy: i32, state: &mut State<'_>) {
        let p_new = {
            let map = state.world.get::<&Map>(state.e_map).unwrap();
            let pos = *state.world.get::<&Pos>(state.e_player).unwrap();

            let p_new = Pos::new(pos.x + dx, pos.y + dy);
            if map.tile_at(p_new).path_cost.is_none() {
                return;
            }

            p_new
        };

        *state.world.get::<&mut Pos>(state.e_player).unwrap() = p_new;
    }

    pub fn try_move_pos(pos: Pos, state: &mut State<'_>) {
        {
            let map = state.world.get::<&Map>(state.e_map).unwrap();
            if map.tile_at(pos).path_cost.is_none() {
                return;
            }
        }

        *state.world.get::<&mut Pos>(state.e_player).unwrap() = pos;
    }
}
