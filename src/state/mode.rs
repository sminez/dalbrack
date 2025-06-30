//! Modes that the game can be in
use crate::{
    action::{Action, quit, toggle_explored, zoom_in, zoom_out},
    actor::Actor,
    map::{
        builders::{BuildMap, Forest},
        fov::LightSource,
    },
    mob::Mob,
    player::Player,
    state::State,
    ui::{MAP_H, MAP_W, palette},
};
use rand::Rng;
use sdl2::{
    event::Event,
    keyboard::{Keycode, Mod},
    mouse::MouseButton,
};

pub trait GameMode {
    /// Run to initialise the game state before dropping into processing actions
    fn init(&self, state: &mut State<'_>) -> anyhow::Result<()>;

    /// Called after each queued action or player action before updating the UI
    fn after_action(&self, state: &mut State<'_>) -> anyhow::Result<()>;

    /// Called to update the UI and render
    fn update_ui(&self, state: &mut State<'_>) -> anyhow::Result<()>;

    /// Called per input event to obtain the next game action
    fn action_for_input_event(&self, event: &Event, state: &State<'_>) -> Option<Action>;
}

// Allow closures to be used as simple UI update functions that don't have any additional handling
// for actions
impl<F> GameMode for F
where
    F: Fn(&mut State<'_>) -> anyhow::Result<()>,
{
    fn init(&self, _: &mut State<'_>) -> anyhow::Result<()> {
        Ok(())
    }

    fn after_action(&self, _: &mut State<'_>) -> anyhow::Result<()> {
        Ok(())
    }

    fn update_ui(&self, state: &mut State<'_>) -> anyhow::Result<()> {
        (self)(state)
    }

    fn action_for_input_event(&self, _: &Event, _: &State<'_>) -> Option<Action> {
        None
    }
}

/// The main game screen where the player controls their character on a local map of the area
pub struct LocalMap;
impl GameMode for LocalMap {
    fn init(&self, state: &mut State<'_>) -> anyhow::Result<()> {
        state.update_fov()?;
        state.update_light_map()?;
        state.update_ui()
    }

    fn after_action(&self, state: &mut State<'_>) -> anyhow::Result<()> {
        state.update_fov()?;
        state.update_light_map()
    }

    fn update_ui(&self, state: &mut State<'_>) -> anyhow::Result<()> {
        state.update_ui()
    }

    fn action_for_input_event(&self, event: &Event, state: &State<'_>) -> Option<Action> {
        match *event {
            Event::Quit { .. } => Some(quit.into()),

            Event::KeyDown {
                keycode: Some(k),
                repeat: false,
                keymod: Mod::NOMOD,
                ..
            } => match k {
                Keycode::L | Keycode::Right => Actor::try_move(1, 0, state.e_player, state),
                Keycode::H | Keycode::Left => Actor::try_move(-1, 0, state.e_player, state),
                Keycode::K | Keycode::Up => Actor::try_move(0, -1, state.e_player, state),
                Keycode::J | Keycode::Down => Actor::try_move(0, 1, state.e_player, state),
                Keycode::Y => Actor::try_move(-1, -1, state.e_player, state),
                Keycode::U => Actor::try_move(1, -1, state.e_player, state),
                Keycode::B => Actor::try_move(-1, 1, state.e_player, state),
                Keycode::N => Actor::try_move(1, 1, state.e_player, state),

                Keycode::Z => Actor::wait(state.e_player, state),

                Keycode::RightBracket => Some(zoom_in.into()),
                Keycode::LeftBracket => Some(zoom_out.into()),

                Keycode::Q | Keycode::Escape => Some(quit.into()),

                // Debug actions
                Keycode::Space => Some(toggle_explored.into()),

                Keycode::C => Some(Action::from(move |state: &mut State<'_>| {
                    state.clear_with_comp::<LightSource>()
                })),

                Keycode::R => Some(Action::from(move |state: &mut State<'_>| {
                    state.clear_with_comp::<LightSource>()?;
                    state.clear_with_comp::<Mob>()?;

                    let (pos, map) = Forest::default().new_map(
                        MAP_W as usize,
                        MAP_H as usize,
                        Default::default(),
                        state,
                    );
                    state.set_map(map);
                    Player::warp(pos, state);

                    state.update_fov()?;
                    state.update_light_map()?;
                    state.update_ui()?;

                    Ok(())
                })),

                _ => None,
            },

            Event::KeyDown {
                keycode: Some(k),
                repeat: false,
                keymod: Mod::LSHIFTMOD,
                ..
            } => match k {
                Keycode::Right => Actor::try_move(1, 1, state.e_player, state),
                Keycode::Left => Actor::try_move(-1, -1, state.e_player, state),
                Keycode::Up => Actor::try_move(1, -1, state.e_player, state),
                Keycode::Down => Actor::try_move(-1, 1, state.e_player, state),

                _ => None,
            },

            Event::MouseButtonDown {
                mouse_btn: MouseButton::Right,
                x,
                y,
                ..
            } => Actor::path_to_in_player_explored(state.ui.map_click(x, y), state.e_player, state),

            // Debug actions
            Event::MouseButtonDown {
                mouse_btn: MouseButton::Left,
                x,
                y,
                ..
            } => Some(Action::from(move |state: &mut State<'_>| {
                let mut rng = rand::rng();
                let color = palette::FIRE_1;
                state.world.spawn((
                    state.ui.map_click(x, y),
                    state.tile_with_color("star", color),
                    LightSource {
                        range: rng.random_range(3..8),
                        color,
                    },
                ));

                Ok(())
            })),

            _ => None,
        }
    }
}
