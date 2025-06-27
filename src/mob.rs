use crate::{
    Pos,
    action::{Action, ActionProvider, AvailableActions},
    actor::Actor,
    map::fov::{FovRange, Opacity},
    state::State,
};
use hecs::{Entity, EntityBuilder};
use std::cmp::{max, min};

pub struct MobSpec {
    pub name: &'static str,
    pub ident: &'static str,
    pub color: &'static str,
    pub fov_range: u32,
}

pub const PIXIE: MobSpec = MobSpec {
    name: "pixie",
    ident: "p",
    color: "fadedPurple",
    fov_range: 6,
};

/// Mobs cover all sentient creatures other than the player.
#[derive(Debug)]
pub struct Mob;

impl Mob {
    pub fn spawn_spec(spec: MobSpec, x: i32, y: i32, state: &mut State<'_>) -> Entity {
        state.world.spawn(
            EntityBuilder::new()
                .add(Mob)
                .add(FovRange(spec.fov_range))
                .add_bundle(Actor {
                    pos: Pos::new(x, y),
                    tile: state.tile_with_named_color(spec.ident, spec.color),
                    opacity: Opacity(0.5),
                    actions: AvailableActions::from(RandomMoveAI),
                })
                .build(),
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RandomMoveAI;

impl ActionProvider for RandomMoveAI {
    fn retain(&self) -> bool {
        true
    }

    fn available_actions(&mut self, entity: Entity, state: &State<'_>) -> Option<Vec<Action>> {
        if state.mapset.is_empty() {
            return None;
        }
        let map = state.mapset.current();
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
