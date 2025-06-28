//! Modes that the game can be in
use crate::{
    action::{Action, AvailableActions, quit, toggle_explored, zoom_in, zoom_out},
    actor::Actor,
    map::{
        builders::{BuildMap, Forest},
        fov::LightSource,
    },
    player::Player,
    state::State,
    ui::palette,
};
use rand::Rng;
use sdl2::{
    event::Event,
    keyboard::{Keycode, Mod},
    mouse::MouseButton,
};

pub trait GameMode {
    fn init(&self, state: &mut State<'_>) -> anyhow::Result<()>;

    fn action_for_input_event(&self, event: &Event, state: &State<'_>) -> Option<Action>;
}

/// The main game screen where the player controls their character on a local map of the area
pub struct LocalMap;
impl GameMode for LocalMap {
    fn init(&self, state: &mut State<'_>) -> anyhow::Result<()> {
        state.update_fov()?;
        state.update_light_map()?;
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
                    state.clear_with_comp::<AvailableActions>()?;

                    let (pos, map) = Forest::default().new_map(60, 35, Default::default(), state);
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
