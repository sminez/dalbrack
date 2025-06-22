//! Input handling for SDL2 events
use crate::{
    Pos,
    action::{Action, quit, zoom_in, zoom_out},
    actor::Actor,
    state::State,
};
use sdl2::{event::Event, keyboard::Keycode, mouse::MouseButton};

pub type InputMapping = fn(Event) -> Option<Action>;

/// The default input mapping for the primary game state
pub fn map_event_in_game_state(event: &Event, state: &State<'_>) -> Option<Action> {
    match *event {
        Event::Quit { .. } => Some(quit.into()),

        Event::KeyDown {
            keycode: Some(k),
            repeat: false,
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

            _ => None,
        },

        Event::MouseButtonDown {
            mouse_btn: MouseButton::Right,
            x,
            y,
            ..
        } => {
            let target = Pos::new(x / state.ui.dxy as i32, y / state.ui.dxy as i32);
            Actor::path_to_in_player_explored(target, state.e_player, state)
        }

        _ => None,
    }
}
