use crate::{
    Pos,
    action::{Action, ActionProvider, AvailableActions},
    actor::Actor,
    map::fov::{FovRange, Opacity},
    state::State,
    ui::palette,
};
use hecs::{Entity, EntityBuilder};
use sdl2::pixels::Color;
use std::cmp::{max, min};

pub struct MobSpec {
    pub name: &'static str,
    pub ident: &'static str,
    pub color: Color,
    pub fov_range: u32,
}

pub const PIXIE: MobSpec = MobSpec {
    name: "pixie",
    ident: "pi",
    color: palette::FADED_PURPLE,
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
                    tile: state.tile_with_color(spec.ident, spec.color),
                    opacity: Opacity(0.5),
                    actions: AvailableActions::from(CuriousAI::default()),
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

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct CuriousAI {
    last_player_pos: Pos,
}

impl ActionProvider for CuriousAI {
    fn retain(&self) -> bool {
        true
    }

    fn available_actions(&mut self, entity: Entity, state: &State<'_>) -> Option<Vec<Action>> {
        if state.mapset.is_empty() {
            return None;
        }

        // required components
        let pos = *state.world.get::<&Pos>(entity).ok()?;
        let fov = state.world.get::<&FovRange>(entity).ok()?;
        let player_pos = *state.world.get::<&Pos>(state.e_player).ok()?;

        // potter around if we can't see the player
        if !fov.fast_has_los(pos, player_pos, state) {
            return RandomMoveAI.available_actions(entity, state);
        }

        let map = state.mapset.current();
        let prev = self.last_player_pos.fdist(pos);
        let current = pos.fdist(player_pos);
        self.last_player_pos = player_pos;

        // If the player is moving towards us: back off
        if current < prev && current < 2.5 {
            for p in map.neighbouring_tiles(pos) {
                let dist = p.fdist(player_pos);
                if map.tile_at(p).path_cost.is_some() && dist > current {
                    return Some(vec![Action::from(move |state: &mut State<'_>| {
                        *state.world.query_one_mut::<&mut Pos>(entity)? = p;

                        Ok(())
                    })]);
                }
            }
        }

        // Otherwise approach to a fixed distance
        for p in map.neighbouring_tiles(pos) {
            let dist = p.fdist(player_pos);
            if map.tile_at(p).path_cost.is_some() && dist <= current && dist > 1.5 {
                return Some(vec![Action::from(move |state: &mut State<'_>| {
                    *state.world.query_one_mut::<&mut Pos>(entity)? = p;

                    Ok(())
                })]);
            }
        }

        None
    }
}
