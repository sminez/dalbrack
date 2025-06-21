//! Input handling for SDL2 events
use crate::{
    action::{Action, quit, zoom_in, zoom_out},
    player::Player,
};
use sdl2::{event::Event, keyboard::Keycode, mouse::MouseButton};

pub type InputMapping = fn(Event) -> Option<Action>;

/// The default input mapping for the primary game state
pub fn map_event_in_game_state(event: &Event) -> Option<Action> {
    let action = match *event {
        Event::Quit { .. } => quit.into(),

        Event::KeyDown {
            keycode: Some(k),
            repeat: false,
            ..
        } => match k {
            Keycode::L | Keycode::Right => Player::try_move(1, 0),
            Keycode::H | Keycode::Left => Player::try_move(-1, 0),
            Keycode::K | Keycode::Up => Player::try_move(0, -1),
            Keycode::J | Keycode::Down => Player::try_move(0, 1),
            Keycode::Y => Player::try_move(-1, -1),
            Keycode::U => Player::try_move(1, -1),
            Keycode::B => Player::try_move(-1, 1),
            Keycode::N => Player::try_move(1, 1),

            Keycode::RightBracket => zoom_in.into(),
            Keycode::LeftBracket => zoom_out.into(),

            Keycode::Q | Keycode::Escape => quit.into(),

            _ => return None,
        },

        Event::MouseButtonDown {
            mouse_btn: MouseButton::Right,
            x,
            y,
            ..
        } => Player::mouse_move(x, y),

        _ => return None,
    };

    Some(action)
}
