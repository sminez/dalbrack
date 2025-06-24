use crate::{
    Pos,
    action::{Action, ActionProvider, AvailableActions},
    actor::Actor,
    map::fov::Opacity,
    state::State,
};
use hecs::{Entity, EntityBuilder};
use std::cmp::{max, min};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RandomMoveAI;

impl ActionProvider for RandomMoveAI {
    fn retain(&self) -> bool {
        true
    }

    fn available_actions(&mut self, entity: Entity, state: &State<'_>) -> Option<Vec<Action>> {
        let map = state.current_map()?;
        let (xmax, ymax) = (map.w - 1, map.h - 1);

        let mut pos = state.world.get::<&Pos>(entity).ok()?.random_offset();
        pos.x = max(0, min(xmax as i32, pos.x));
        pos.y = max(0, min(ymax as i32, pos.y));
        map.tile_at(pos).path_cost?;

        Some(vec![Action::from(move |state: &mut State<'_>| {
            *state.world.query_one_mut::<&mut Pos>(entity)? = pos;

            Ok(())
        })])
    }
}

#[derive(Debug)]
pub struct Mob;

impl Mob {
    pub fn spawn(ident: &str, color: &str, x: i32, y: i32, state: &mut State<'_>) -> Entity {
        let mut tile = state.ts.tile(ident).unwrap();
        tile.color = *state.palette.get(color).unwrap();

        let mut builder = EntityBuilder::new();
        state.world.spawn(
            builder
                .add(Mob)
                .add_bundle(Actor {
                    pos: Pos::new(x, y),
                    tile,
                    opacity: Opacity(0.7),
                    actions: AvailableActions::from(RandomMoveAI),
                })
                .build(),
        )
    }
}
