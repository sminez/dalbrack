//! Systems relating to the player controlled character
use crate::{Pos, action::Action, map::Map, state::State};

pub struct Player;

impl Player {
    pub fn set_pos(p_new: Pos, state: &mut State<'_>) {
        *state
            .world
            .query_one_mut::<&mut Pos>(state.e_player)
            .unwrap() = p_new;
    }

    pub fn try_move(dx: i32, dy: i32) -> Action {
        Action::from(move |state: &mut State<'_>| {
            let p_new = {
                let map = state.world.get::<&Map>(state.e_map).unwrap();
                let pos = *state.world.get::<&Pos>(state.e_player).unwrap();

                let p_new = Pos::new(pos.x + dx, pos.y + dy);
                if map.tile_at(p_new).path_cost.is_none() {
                    return Ok(());
                }

                p_new
            };

            *state.world.get::<&mut Pos>(state.e_player).unwrap() = p_new;

            Ok(())
        })
    }

    pub fn try_move_pos(pos: Pos) -> Action {
        Action::from(move |state: &mut State<'_>| {
            {
                let map = state.world.get::<&Map>(state.e_map).unwrap();
                if map.tile_at(pos).path_cost.is_none() {
                    return Ok(());
                }
            }

            *state.world.get::<&mut Pos>(state.e_player).unwrap() = pos;

            Ok(())
        })
    }

    pub fn mouse_move(mouse_x: i32, mouse_y: i32) -> Action {
        Action::from(move |state: &mut State<'_>| {
            let target = Pos::new(mouse_x / state.ui.dxy as i32, mouse_y / state.ui.dxy as i32);
            let from = *state.world.query_one_mut::<&Pos>(state.e_player).unwrap();
            let map = state.world.query_one_mut::<&mut Map>(state.e_map).unwrap();
            let path = map.a_star_in_player_explored(from, target);

            state
                .action_queue
                .extend(path.into_iter().map(Player::try_move_pos));

            Ok(())
        })
    }
}
