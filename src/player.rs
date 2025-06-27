//! Systems relating to the player controlled character
use crate::{
    Pos,
    action::AvailableActions,
    actor::Actor,
    map::fov::{Fov, FovRange, Opacity},
    state::State,
    ui::palette,
};
use hecs::EntityBuilder;

#[derive(Debug)]
pub struct Player;

impl Player {
    pub fn new_base_bundle(pos: Pos, fov_range: FovRange, state: &State<'_>) -> EntityBuilder {
        let mut builder = Self::new_bundle_without_fov(pos, state);
        builder.add(fov_range);

        builder
    }

    pub fn new_bundle_without_fov(pos: Pos, state: &State<'_>) -> EntityBuilder {
        let mut builder = EntityBuilder::new();
        builder.add(Player).add_bundle(Actor {
            pos,
            tile: state.tile_with_color("@", palette::WHITE),
            opacity: Opacity(0.7),
            actions: AvailableActions::default(),
        });

        builder
    }

    pub fn warp(new_pos: Pos, state: &State<'_>) {
        let mut pos = state.world.get::<&mut Pos>(state.e_player).unwrap();
        *pos = new_pos;

        if let Ok(mut fov) = state.world.get::<&mut Fov>(state.e_player) {
            fov.dirty = true;
        };
    }
}
