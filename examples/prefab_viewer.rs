use risky_endevours::{data_files::parse_ibm437_prefab, tileset::TileSet, ui::Sdl2UI};
use sdl2::{event::Event, keyboard::Keycode};
use std::env::args;

const X: u32 = 1;
const Y: u32 = 1;

pub fn main() -> anyhow::Result<()> {
    let path = match args().nth(1) {
        Some(path) => path,
        None => "assets/prefabs/room.prefab".to_string(),
    };

    let mut ui = Sdl2UI::init(1280, 1000, 50, "Risky Endevours")?;

    render(&path, &mut ui)?;

    loop {
        match ui.wait_event() {
            Event::Quit { .. } => return Ok(()),

            Event::KeyDown {
                keycode: Some(k),
                repeat: false,
                ..
            } => match k {
                Keycode::Num1 => ui.ts = TileSet::df_classic()?,
                Keycode::Num2 => ui.ts = TileSet::df_buddy()?,
                Keycode::Num3 => ui.ts = TileSet::df_sb()?,
                Keycode::Num4 => ui.ts = TileSet::df_nordic()?,
                Keycode::Num5 => ui.ts = TileSet::df_rde()?,
                Keycode::Num6 => ui.ts = TileSet::df_yayo()?,
                Keycode::Num7 => ui.ts = TileSet::df_kruggsmash()?,

                Keycode::RightBracket => ui.dxy += 5,
                Keycode::LeftBracket => ui.dxy -= 5,

                Keycode::Space => ui.toggle_debug_bg(),
                Keycode::Q | Keycode::Escape => return Ok(()),

                _ => continue,
            },

            _ => continue,
        }

        render(&path, &mut ui)?;
    }
}

fn render(path: &str, ui: &mut Sdl2UI<'_>) -> anyhow::Result<()> {
    let grid = parse_ibm437_prefab(path, &ui.ts, &ui.palette)?;
    ui.clear();
    ui.blit_grid(&grid, X, Y)?;
    ui.render()
}
