use dalbrack::{TITLE, data_files::parse_cp437_prefab, state::State, tileset::TileSet};
use sdl2::{event::Event, keyboard::Keycode, pixels::Color};
use std::env::args;

const X: i32 = 1;
const Y: i32 = 1;

pub fn main() -> anyhow::Result<()> {
    let path = match args().nth(1) {
        Some(path) => path,
        None => "data/prefabs/room.prefab".to_string(),
    };

    let mut state = State::init(1280, 1000, 50, TITLE)?;
    state.ui.set_bg(Color::BLACK);
    update(&path, &mut state)?;

    loop {
        match state.ui.wait_event() {
            Event::Quit { .. } => return Ok(()),

            Event::KeyDown {
                keycode: Some(k),
                repeat: false,
                ..
            } => match k {
                Keycode::Num1 => state.ts = TileSet::df_classic()?,
                Keycode::Num2 => state.ts = TileSet::df_buddy()?,
                Keycode::Num3 => state.ts = TileSet::df_sb()?,
                Keycode::Num4 => state.ts = TileSet::df_nordic()?,
                Keycode::Num5 => state.ts = TileSet::df_rde()?,
                Keycode::Num6 => state.ts = TileSet::df_yayo()?,
                Keycode::Num7 => state.ts = TileSet::df_kruggsmash()?,

                Keycode::RightBracket => state.ui.dxy += 5,
                Keycode::LeftBracket => state.ui.dxy -= 5,

                Keycode::Q | Keycode::Escape => return Ok(()),

                _ => continue,
            },

            _ => continue,
        }

        update(&path, &mut state)?;
    }
}

fn update(path: &str, state: &mut State<'_>) -> anyhow::Result<()> {
    let grid = parse_cp437_prefab(path, &state.ts, &state.palette)?;
    state.world.clear();
    grid.spawn_all_at(X, Y, &mut state.world);

    state.ui.clear();
    state.blit_all()?;
    state.ui.render()
}
