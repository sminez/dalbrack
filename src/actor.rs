use crate::{
    Pos,
    action::{Action, ActionProvider, AvailableActions},
    map::fov::{Fov, Opacity},
    state::State,
    tileset::Tile,
};
use hecs::{Bundle, Entity};

#[derive(Debug, Bundle)]
pub struct Actor {
    pub pos: Pos,
    pub tile: Tile,
    pub opacity: Opacity,
    pub actions: AvailableActions,
}

impl Actor {
    pub fn wait(entity: Entity, state: &State<'_>) -> Option<Action> {
        state
            .world
            .get::<&mut AvailableActions>(entity)
            .unwrap()
            .push(Wait);

        None
    }

    pub fn try_move(dx: i32, dy: i32, entity: Entity, state: &State<'_>) -> Option<Action> {
        let pos = *state.world.get::<&Pos>(entity).unwrap() + Pos::new(dx, dy);
        let map = state.mapset.current();
        if map.tile_at(pos).blocks_movement() {
            return None;
        }

        state
            .world
            .get::<&mut AvailableActions>(entity)
            .unwrap()
            .push(Move1(pos));

        None
    }

    pub fn path_to_in_player_explored(
        target: Pos,
        entity: Entity,
        state: &State<'_>,
    ) -> Option<Action> {
        let pos = *state.world.get::<&Pos>(entity).unwrap();
        let fp = FollowPath::try_new_a_star_in_player_explored(pos, target, state)?;
        state
            .world
            .get::<&mut AvailableActions>(entity)
            .unwrap()
            .push(fp);

        None
    }
}

#[derive(Debug)]
pub struct Wait;

impl ActionProvider for Wait {
    fn retain(&self) -> bool {
        false
    }

    fn available_actions(&mut self, _entity: Entity, _state: &State<'_>) -> Option<Vec<Action>> {
        Some(vec![Action::from(move |_state: &mut State<'_>| Ok(()))])
    }
}

/// Try to move a single tile from the current position
#[derive(Debug)]
pub struct Move1(pub Pos);

impl ActionProvider for Move1 {
    fn retain(&self) -> bool {
        false
    }

    fn available_actions(&mut self, entity: Entity, _state: &State<'_>) -> Option<Vec<Action>> {
        let pos = self.0;

        Some(vec![Action::from(move |state: &mut State<'_>| {
            *state.world.get::<&mut Pos>(entity).unwrap() = pos;
            if let Ok(mut fov) = state.world.get::<&mut Fov>(entity) {
                fov.dirty = true;
            };

            Ok(())
        })])
    }
}

#[derive(Debug)]
pub struct FollowPath {
    path: Vec<Pos>,
}

impl FollowPath {
    pub fn try_new_a_star(from: Pos, target: Pos, state: &State<'_>) -> Option<Self> {
        let mut path = state.mapset.current().a_star(from, target);

        if path.is_empty() {
            None
        } else {
            path.reverse();

            Some(Self { path })
        }
    }

    pub fn try_new_a_star_in_player_explored(
        from: Pos,
        target: Pos,
        state: &State<'_>,
    ) -> Option<Self> {
        let mut path = state
            .mapset
            .current()
            .a_star_in_player_explored(from, target);

        if path.is_empty() {
            None
        } else {
            path.reverse();

            Some(Self { path })
        }
    }
}

impl ActionProvider for FollowPath {
    fn retain(&self) -> bool {
        !self.path.is_empty()
    }

    fn available_actions(&mut self, entity: Entity, state: &State<'_>) -> Option<Vec<Action>> {
        let pos = self.path.pop()?;
        let map = state.mapset.current();
        if map.tile_at(pos).blocks_movement() {
            self.path.clear();
            return None;
        }

        Some(vec![Action::from(move |state: &mut State<'_>| {
            *state.world.get::<&mut Pos>(entity).unwrap() = pos;
            if let Ok(mut fov) = state.world.get::<&mut Fov>(entity) {
                fov.dirty = true;
            };

            Ok(())
        })])
    }
}
